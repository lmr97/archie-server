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
        mngr             = mp.Manager()
        self.cpus        = os.cpu_count()
        self.attrs       = attrs
        self.conn        = cxn
        self.list_len    = lb_list.length
        self.list_ranked = lb_list.is_ranked
        self.done_rows   = mngr.list([""] * self.list_len)
        self.row_to_send = mngr.Value('i', 0)

        self._set_batches(lb_list)
        

    def _set_batches(self, lb_list: lbc.LetterboxdList):
        
        batch_size   = max(lb_list.length // self.cpus, 1)
        ided_rows    = enumerate(lb_list)  # tag rows, for ordered send
        self.batches = [b for b in itertools.batched(ided_rows, batch_size)]


    def run_jobs(self):

        tpool   = mp.Pool(processes=self.cpus)
        threads = []

        for b in self.batches:
            t_res = tpool.apply_async(self._send_batch_rows, [b])
            threads.append(t_res)

        for thread in threads:
            thread.get()


    def run_jobs_batch_send(self):
        """
        Takes the batches (defined by Parallelizer._set_batches()) and sends them
        out to `self.cpus` threads, processed asynchronously. Each batch is sent,
        in order, once completed.
        """

        tpool   = mp.Pool(processes=self.cpus)
        threads = []

        for b in self.batches:
            t_res = tpool.apply_async(self._get_batch_rows, [b])
            threads.append(t_res)

        for thread in threads:
            for row in thread.get():
                send_line(self.conn, row)


    def _get_batch_rows(self, batch: tuple) -> list[str]:
        """
        Processes a batch synchronously, returning a `list` of `str`s
        that are the CSV rows. It is not a `list` of `tuple`s, since 
        `row_id` is not included.
        """
        done_batch = []
        for (row_id, url) in batch:
            film = lbc.LetterboxdFilm(url)
            title = "\"" + film.title + "\""
            file_row = title + "," + film.year

            if len(self.attrs) > 0:
                file_row += "," + film.get_attrs_csv(self.attrs)

            if self.list_ranked:
                file_row = str(row_id+1) + "," + file_row

            done_batch.append(file_row)
            
        return done_batch


    def _send_batch_rows(self, batch: tuple):
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

            self._send_row_in_sequence(row_id, file_row)


    def _send_row_in_sequence(
        self, 
        row_id: int, 
        file_row: str, 
    ):
        """
        Takes async-given rows, and sends them in order from a pool of completed rows.
        Once called, it will send rows in order from the pool until the next row is not available.
        
        For example, if the pool (a list) looked like this,

            ```
            [
                "row_content0", 
                "row_content1", 
                "row_content2", 
                "", 
                "row_content4",   # <-- with `row_to_send` at this index
                "row_content5",
                "row_content6",
                "row_content7",
                "",
                "row_content9"
                "",
                ...
            ]
            ```
        then the values sent by thisdoc
        """
        self.done_rows[row_id] = file_row

        next_row = self.done_rows[self.row_to_send.value]

        while next_row:
            send_line(self.conn, next_row)

            self.row_to_send.value += 1

            if self.row_to_send.value > self.list_len-1:
                return   # we're done

            next_row = self.done_rows[self.row_to_send.value]