import sys
import socket
from svr_io_lib.misc_utils import debug_print

# parameters for streaming data to server
ENDIANNESS  = 'big'
SIZE_BYTES  = 2

# get number of films
def send_list_len(conn: socket.socket, list_len: int):
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