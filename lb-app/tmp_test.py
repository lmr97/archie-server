"""
This script exists to informally test the basic functionality of 
the the Letterboxd app without running through the full test suite.
"""

import os
import socket
import json
from letterboxd_list import VALID_ATTRS

PY_SOCK    = os.getenv("PY_CONT_SOCK", "127.0.0.1:3575")
(IP, PORT) = PY_SOCK.split(":")
PORT       = int(PORT)
ADDRESS    = (IP, PORT)


def send_str(msg: str) -> socket.socket:
    """
    Send a given string to lb_app.py.
    """
    test_sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    test_sock.connect(ADDRESS)
    test_sock.sendall(bytes(msg, "utf-8"))
    return test_sock


def receive_list_len(connection: socket.socket) -> int:
    """
    Receive (and decode) the list length sent from lb_app.py.
    """
    byte_len = connection.recv(2)
    list_len = int.from_bytes(byte_len, byteorder='big', signed=False)
    return list_len


def receive_decode_row(connection: socket.socket) -> str:
    """
    Receives bytes and decodes them as a string the way the server does:
    use the first 2 bytes to determine the length of the data.
    """
    byte_len   = connection.recv(2)
    bytes_int  = int.from_bytes(byte_len, byteorder='big', signed=False)
    decoded    = connection.recv(bytes_int).decode("utf-8")
    return decoded



def main():
    vattrs_no_stats = VALID_ATTRS.copy()   # keep module-level global in tact in this scope
    vattrs_no_stats.remove("watches")
    vattrs_no_stats.remove("likes")
    vattrs_no_stats.remove("avg-rating")

    megarow_list = {
        "list_name": "truly-random-films",
        "author_user": "dialectica972",
        "attrs": vattrs_no_stats
    }

    request_str  = json.dumps(megarow_list)
    connection   = send_str(request_str)

    lb_list      = []
    current_row  = receive_decode_row(connection)
    while current_row != "done!":
        print("\n", current_row)
        # adding newline for easier comparison against touchstone file read in later
        lb_list.append(current_row+"\n")
        try:
            current_row  = receive_decode_row(connection)
        except Exception as e:
            print(e)
            connection.close()
            break

    print(len(lb_list))


if __name__ == "__main__":
    main()