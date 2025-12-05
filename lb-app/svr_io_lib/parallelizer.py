import os
from time import sleep
import socket
import multiprocessing as mp
import itertools
from collections import deque
import letterboxd_list.containers as lbc

from svr_io_lib.misc_utils import InterruptHandler
from svr_io_lib.senders import send_line



class Parallelizer:
    def __init__(self, cxn: socket.socket, lb_list: lbc.LetterboxdList, attrs: list, debug):
        """
        Handles the parallelization of the work. Divides the list URLs into batches,
        then processes these in parallel, sending each batch in order.
        """

        self.cpus    = os.cpu_count()
        self.attrs   = attrs
        self.conn    = cxn
        self.lb_list = lb_list._films   # to make sure the Curl object doesn't get passed around
        self.ranked  = lb_list.is_ranked

        self.interrupt_handler = InterruptHandler(cxn)
        
    

    def _set_batches(self, lb_list: lbc.LetterboxdList):
        
        batch_size   = max(lb_list.length // self.cpus, 1)
        ided_rows    = enumerate(lb_list)  # tag rows, for ordered send
        self.batches = [b for b in itertools.batched(ided_rows, batch_size)]


    def run_jobs(self):
        job_queue   = deque(maxlen=self.cpus)
        thread_pool = mp.Pool(processes=self.cpus)

        print("filling queue")
        for (row_id, url) in enumerate(self.lb_list):

            if row_id + 1 >= self.cpus:
                try:
                    csv_row = job_queue.popleft().get()
                except Exception as e0:

                    # join all threads
                    while len(job_queue):
                        job_queue.popleft().get()
                    raise e0                # let main thread handle the exception
                
                print(f"sending row: {csv_row}")
                sleep(0.5)
                send_line(self.conn, csv_row)

            print("add element")
            t_res = thread_pool.apply_async(self._get_csv_row, args=[url, row_id])
            job_queue.append(t_res)

        # empty the queue
        print("emptying queue")
        while len(job_queue):
            csv_row = job_queue.popleft().get()
            print(f"sending row: {csv_row}")
            send_line(self.conn, csv_row)



    def _get_csv_row(self, film_url: str, row_id: int):

        film = lbc.LetterboxdFilm(film_url)
        title = "\"" + film.title + "\""
        file_row = title + "," + film.year

        if len(self.attrs) > 0:
            file_row += "," + film.get_attrs_csv(self.attrs)

        if self.ranked:
            file_row = str(row_id+1) + "," + file_row

        return file_row


    def run_jobs_batch_send(self):
        """
        Takes the batches (defined by Parallelizer._set_batches()) and sends them
        out to `self.cpus` threads, processed asynchronously. Each batch is sent,
        in order, once completed.
        """

        tpool   = mp.Pool(processes=self.cpus)
        threads = []

        for b in self.batches:
            print("appending batch...")
            t_res = tpool.apply_async(self._get_batch_rows, [b])
            threads.append(t_res)

        print("waiting for threads...")
        for i, thread in enumerate(threads):
            print(f"waiting for thread {i}")
            rows = thread.get()
            for row in rows:
                # print(f"sending row: {row}")
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

            if self.ranked:
                file_row = str(row_id+1) + "," + file_row

            done_batch.append(file_row)
            
        return done_batch