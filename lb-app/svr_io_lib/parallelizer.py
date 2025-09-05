import os
import socket
import multiprocessing as mp
import itertools
import letterboxd_list.containers as lbc
from svr_io_lib.senders import send_line


class Parallelizer:
    def __init__(self, cxn: socket.socket, lb_list: lbc.LetterboxdList, attrs: list):
        """
        Handles the parallelization of the work. The LetterboxdList's rows are ID'd first, 
        so we know their order, and then batched and sent out to one thread per CPU. 
        The finished rows are placed in a managed (thread-safe) list (init as empty strings
        along the length of the list), at the correct index. Then they are send sequentially,
        following the list ordering.
        """
        cpus             = os.cpu_count()
        self.attrs       = attrs
        self.conn        = cxn
        self.list_len    = lb_list.length
        self.list_ranked = lb_list.is_ranked
        self.row_to_send = mp.Value('i', 0)
        self.tpool       = mp.Pool(processes=cpus)

        row_mngr         = mp.Manager()
        row_mngr.start()
        self.done_rows   = row_mngr.list([""] * self.list_len)

        self.set_batches(lb_list, cpus)
        

    def set_batches(self, lb_list: lbc.LetterboxdList, cpus: int):
        
        batch_size   = lb_list.length // self.cpus
        
        # don't multithread for tiny lists
        if self.cpus >= lb_list.length:
            batch_size = lb_list.length
        
        ided_rows    = enumerate(lb_list)  # tag rows, for ordered send
        self.batches = [b for b in itertools.batched(ided_rows, batch_size)]


    def run_jobs(self):
        threads = []
        for b in self.batches:
            self.tpool.apply_async(self.send_batch_rows, [b])

        for thread in threads:
            thread.get()


    def send_batch_rows(self, batch: tuple):
        """
        Since `itertools.batched` uses `tuple`s to make batches, and the 
        original iterable was a `list` of `(row_id, film)` `tuple`s itself,
        `batch` is a `tuple` of `tuple`s.
        """
        
        for (row_id, url) in batch:
            film = lbc.LetterboxdFilm(url)
            title = "\"" + film.title + "\""
            file_row = title + "," + film.year

            if len(self.attrs) > 0:
                file_row += "," + film.get_attrs_csv(self.attrs)

            if self.list_ranked:
                file_row = str(row_id+1) + "," + file_row

            self._send_row_in_sequence((row_id, file_row))


    def _send_row_in_sequence(self, ided_file_row: tuple):
        """
        Takes async-given rows, and sends them in order from a pool of completed rows.
        Will send rows in sequence until the next row is not available.
        """
        self.done_rows[ided_file_row[0]] = ided_file_row[1]

        next_row = self.done_rows[self.row_to_send]

        while next_row:
            send_line(next_row)
            self.row_to_send += 1
            next_row = self.done_rows[self.row_to_send]