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
import signal
from letterboxd_list import VALID_ATTRS
import letterboxd_list.containers as lbl

# parameters for streaming data to server
ENDIANNESS  = 'big'
SIZE_BYTES  = 2
DEBUG_PRINT = False


class InterruptHandler:
    """
    You'll never believe this, but this class handles OS signals.
    """
    def __init__(self, main_listener: socket.socket):

        self.kill_now = False
        # keep a copy of the main connection to close on exit
        self._main_listener = main_listener

        # define listeners for graceful exit, if in main thread of interpreter
        # otherwise just hold the listener and a shutdown boolean
        if __name__ == "__main__":
            signal.signal(signal.SIGINT, self._sig_exit_handler)
            signal.signal(signal.SIGTERM, self._sig_exit_handler)

    def _sig_exit_handler(self, signo, _stack_frame):
        """
        Called when SIGINT or SIGTERM is recieved by the program.
        """
        self.kill_now = True
        print(f"\n\033[0;33mReceived signal \"{signal.strsignal(signo)}\", exiting...\033[0m")
        self._main_listener.close()



def debug_print(msg: str):
    if DEBUG_PRINT:
        print(msg)


# get number of films
def send_list_len(list_len: int, conn: socket.socket):
    """
    Sends precisely 2 bytes that give the number of films in the list.

    It works by using a description <meta> element in the HTML header 
    that follows the form "A list of <number> films compiled on
    Letterboxd, including <film>..." etc. (The user-made description of the 
    list comes after this standard-issue desc., after "About this list:").
    """

    print(" -- Sending total length --")
    print(f"List length: {list_len}")

    int_as_bytes = list_len.to_bytes(SIZE_BYTES, ENDIANNESS)
    conn.sendall(int_as_bytes)


def send_line(conn: socket.socket, line: str):
    """
    First sends exactly 2 bytes, containing the number of chars to be written
    out to the stream (so the server knows how many to read), then 
    sends that data as bytes.
    
    This size-then-data protocol is implemented because Rust's 
    std::io::read_to_string(), my first choice, reads until EOF is received,
    and Python doesn't provide a good way to send an EOF signal without 
    closing a connection. So, Rust's std::io::read_exact() had to be used,
    for which a set size of buffer must be specified.

    This function is also used to send error messages, using the same protocol.
    """
    debug_print(" -- Sending line --")
    byte_row = bytes(line, 'utf-8')

    # send row length
    # needs to be byte length of row, to account for multi-byte characters
    row_len  = len(byte_row)
    byte_len = row_len.to_bytes(SIZE_BYTES, ENDIANNESS)
    debug_print(row_len)
    debug_print(byte_len)

    try:
        conn.sendall(byte_len)
    except OSError as ose:
        print(f"Could not send back above byte length, printing error here: {repr(ose)}",
            file=sys.stderr)
        return

    # send row itself
    debug_print(byte_row)
    try:
        conn.sendall(byte_row)
    except OSError as ose:
        print(f"Could not send back above row, printing error here: {repr(ose)}",
            file=sys.stderr)
        return


def to_capital_header(attr: str) -> str:
    """
    Capitalizes the first word in a given string,
    and replaces the `-` with a space.
    """
    words    = attr.split("-")
    init_cap = [w.capitalize() for w in words]
    uc_attr  = " ".join(init_cap)
    return uc_attr


def get_list_with_attrs(query: dict, conn: socket.socket) -> None:
    """Gets the requested attributed from the films on a Letterboxd list."""

    attrs = query['attrs']   # for ease of access
    lb_list_url = f"https://letterboxd.com/{query['author_user']}/list/{query['list_name']}/"

    # catch 'none' attribute requests
    if attrs[0] == 'none':
        attrs = []

    # validate attrs. must occur here for a similar reason.
    for attr in attrs:
        if attr not in VALID_ATTRS:
            raise lbl.RequestError(f"Invalid attributes submitted: {attr}")

    try:
        lb_list = lbl.LetterboxdList(lb_list_url, max_length=10_000)
    # let main() handle it, whatever it is
    except Exception as e:
        raise e

    send_list_len(lb_list.length, conn)

    # finalize header
    header = "Title,Year"
    for attr in attrs:
        header  += "," + to_capital_header(attr)

    if lb_list.is_ranked:
        header = "Rank," + header

    send_line(conn, header)

    list_rank = 1       # in case needed
    for url in lb_list:

        film     = lbl.LetterboxdFilm(url)
        title    = "\"" + film.title + "\""            # rudimentary sanitizing
        file_row = title+","+film.year

        if len(attrs) > 0:
            file_row  += "," + film.get_attrs_csv(attrs)

        if lb_list.is_ranked:
            file_row   = str(list_rank) + "," + file_row
            list_rank += 1

        # sends only row data, without newline (to be added at client)
        send_line(conn, file_row)



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
                get_list_with_attrs(query, conn)

        except lbl.ListTooLongError as res_too_long:
            print(f"{repr(res_too_long)}", file=sys.stderr)
            send_list_len(0, conn)
            send_line(conn, f"-- 403 FORBIDDEN -- {repr(res_too_long)}")

        except lbl.RequestError as req_err:
            print(f"{repr(req_err)}", file=sys.stderr)
            send_list_len(0, conn)
            send_line(conn, f"-- 422 UNPROCESSABLE CONTENT -- {repr(req_err)}")

        except lbl.HTTPError as lb_serr:
            print(f"{repr(lb_serr)}", file=sys.stderr)
            send_list_len(0, conn)
            send_line(conn, f"-- 502 BAD GATEWAY -- {repr(lb_serr)}")

        except Exception as e:
            print(f"{repr(e)}", file=sys.stderr)
            send_list_len(0, conn)
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