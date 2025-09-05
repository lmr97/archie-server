"""
The server translates HTTP GET request query into JSON bytes,
and sends them here. since this program is directly
# accessible to the network, it only uses the data format
I will give it from the server, and send back text as a
CSV stream of text.
Error handling and bundling the data as an HTTP response
is also done at the server.

The model is as follows:

                  CSV text <- (this)
                     |          ^
           HTTP      v          |
  Client  <---->   Server  ->  JSON

This script sends the CSV line by line as a byte stream,
first sending 2 bytes to denote the total list size, then
for every list row, sending 2 bytes to indicate the row size
(in bytes), and then sending the row data.

See custom_backend::lb_app_io.rs for how this data is transmitted
to the client.
"""

import os
import sys
import json
import socket

from letterboxd_list import VALID_ATTRS
import letterboxd_list.containers as lbc
from svr_io_lib.misc_utils import to_capital_header, InterruptHandler
from svr_io_lib.senders import send_line, send_list_len
from svr_io_lib.parallelizer import Parallelizer


def validate_query(query: dict) -> (list, lbc.LetterboxdList):

    attrs = query['attrs']   # for ease of access

    # catch 'none' attribute requests
    if attrs[0] == 'none':
        attrs = []
    
    lb_list_url = f"https://letterboxd.com/{query['author_user']}/list/{query['list_name']}/"

    # validate attrs. must occur here for a similar reason.
    for attr in attrs:
        if attr not in VALID_ATTRS:
            raise lbc.RequestError(f"Invalid attributes submitted: {attr}")

    # let main() handle the exception, whatever it is
    return (attrs, lbc.LetterboxdList(lb_list_url, max_length=10_000))


def send_header(conn: socket.socket, attrs: list, list_is_ranked: bool):

    header = "Title,Year"
    attrs.sort()                    # alphabetize
    for attr in attrs:
        header  += "," + to_capital_header(attr)

    if list_is_ranked:
        header = "Rank," + header

    send_line(conn, header)


def send_list_with_attrs(conn: socket.socket, query: dict):
    """
    The central function for the app. (main is just error handling)
    """

    (attrs, lb_list) = validate_query(query)

    send_list_len(conn, lb_list.length)
    send_header(conn, attrs, lb_list.is_ranked)

    # delegate the parallel processing to a Parallelizer object
    Parallelizer(conn, lb_list, attrs).run_jobs()
    print("job finished")



def main():
    """See notes at top of file."""

    print("\n\033[0;32m--- Letterboxd app is starting up! ---\033[0m\n")
    
    # it is vital that the IP address be 0.0.0.0, not 127.0.0.1, 
    # so when the app is running in a container, it is bound to a
    # port that is listening to incoming data from outside the 
    # container
    py_cont_sock = os.getenv("PY_CONT_SOCK", "0.0.0.0:3575")
    (ip, port)   = py_cont_sock.split(":")
    port         = int(port)        # listener.bind() needs tuple of str and int

    listener     = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    listener.bind((ip, port))
    listener.listen(3)

    shutdown_handler = InterruptHandler(listener)

    print(f"\033[0;32mListening on {py_cont_sock}...\033[0m\n")

    # using a global here so the app doesn't have to do I/O from the OS
    # every debug print statement. It is not updated anywhere else.
    global DEBUG_PRINT 
    dbg_env_var = os.getenv("PY_DEBUG")
    if dbg_env_var == "1": 
        DEBUG_PRINT = True

    while not shutdown_handler.kill_now:
        try:
            (conn, _) = listener.accept()
        # interrupts are most likely to be received while socket.accept() is hanging,
        # waiting for a new connection. This means the connection (fd) will be closed
        # while the main loop is awaiting connections, causing an OSError to be thrown
        # (specifically, Errno 9, "bad file descriptor").
        #
        # if somehow an error arose for another reason, we'll try the loop again.
        except OSError as ose:

            if "Bad file descriptor" in ose.strerror and shutdown_handler.kill_now:
                print(f"\033[0;32mTCP connection on {py_cont_sock} closed.\033[0m")
                break

            print(f"Unexpected error while awaiting new connections: {repr(ose)}"
                "\nReattempting awaiting new connections...",
                file=sys.stderr)
            continue

        try:
            # Receives JSON data of the following format:
            # {
            #     "list_name": string,
            #     "author_user": string,
            #     "attrs": array of strings
            # }
            req = conn.recv(700).decode("utf-8")
            query = json.loads(req)

            # for Docker healthchecks, no response sent
            if query == {"msg": "are you healthy?"}:
                send_line(conn, "yep, still healthy!")

            # this is for testing purposes
            elif query == {"msg": "shutdown"}:
                shutdown_handler.kill_now = True

            else:
                send_list_with_attrs(conn, query)

        except lbc.ListTooLongError as res_too_long:
            print(f"{repr(res_too_long)}", file=sys.stderr)
            send_list_len(conn, 0)
            send_line(conn, f"-- 403 FORBIDDEN -- {repr(res_too_long)}")

        except lbc.RequestError as req_err:
            print(f"{repr(req_err)}", file=sys.stderr)
            send_list_len(conn, 0)
            send_line(conn, f"-- 422 UNPROCESSABLE CONTENT -- {repr(req_err)}")

        except lbc.HTTPError as lb_serr:
            print(f"{repr(lb_serr)}", file=sys.stderr)
            send_list_len(conn, 0)
            send_line(conn, f"-- 502 BAD GATEWAY -- {repr(lb_serr)}")

        except Exception as e:
            print(f"{repr(e)}", file=sys.stderr)
            send_list_len(conn, 0)
            send_line(conn, f"-- 500 INTERNAL SERVER ERROR -- {repr(e)}")

        # all control paths lead to Rome (this code block)
        # "done!" signal is sent every single time, no matter what
        finally:
            if shutdown_handler.kill_now:
                print("\n\033[0;33mReceived a shutdown signal, exiting...\033[0m")
                listener.close()
                print(f"\033[0;32mTCP connection on {py_cont_sock} closed.\033[0m")
                return

            send_line(conn, "done!")
            conn.close()    # sends EOF, so that Rust server can read data sent
            continue

        print("\n\033[0;32mRetrival complete, and data sent!\033[0m\n")

if __name__ == "__main__":
    main()