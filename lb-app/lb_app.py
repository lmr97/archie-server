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

import json
import socket
import letterboxd_list.containers as lbl

# parameters for streaming data to server
ENDIANNESS = 'big'
SIZE_BYTES = 2

# Superset of TABBED_ATTRS list in letterboxdFilm.py
VALID_ATTRS = []
with open("./letterboxd_get_list/valid-lb-attrs.txt", "r", encoding="utf-8") as attr_file:
    VALID_ATTRS = attr_file.readlines()

VALID_ATTRS = [a.replace("\n", "") for a in VALID_ATTRS]


class ListTooLongError(lbl.RequestError):
    """
    Special error for when the Letterboxd list is 
    over 65,500 films. This is not only an ungodly 
    amount of films for a list, but also almost over 
    the 2-byte limit to the amount the server can process 
    (given its code, not necessarily the computer's 
    processing power).
    """

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

    if list_len > 65_500:
        raise ListTooLongError

    int_as_bytes = list_len.to_bytes(SIZE_BYTES, ENDIANNESS)
    conn.sendall(int_as_bytes)


def send_list_len_err(conn: socket.socket):
    """
    List length still needs to be sent regardless, but on an error
    condition, it can be a constant (0). 
    """
    list_len_err = 0
    int_as_bytes = list_len_err.to_bytes(SIZE_BYTES, ENDIANNESS)
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
    print(" -- Sending line --")
    byte_row = bytes(line, 'utf-8')

    # send row length
    # needs to be byte length of row, to account for multi-byte characters
    row_len  = len(byte_row)
    byte_len = row_len.to_bytes(SIZE_BYTES, ENDIANNESS)
    print(row_len)
    print(byte_len)
    conn.sendall(byte_len)

    # send row itself
    print(byte_row)
    conn.sendall(byte_row)


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
            send_list_len_err(conn)
            raise lbl.RequestError(f"Invalid attributes submitted: {attr}")

    try:
        lb_list    = lbl.LetterboxdList(lb_list_url)
    # let main() handle it
    except Exception as e:
        send_list_len_err(conn)
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

        file_row += "\n"
        send_line(conn, file_row)


def main():
    """See notes at top of file."""
    listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    listener.bind(('0.0.0.0', 3575))
    listener.listen(1)
    print(f"Listening on {listener.getsockname()}")

    while True:
        (conn, _) = listener.accept()
        try:
            req = conn.recv(2048).decode("utf-8")
            query = json.loads(req)

            # for Docker healthchecks
            if query == {"msg": "are you healthy?"}:
                print("yep, still healthy!")
                continue

            get_list_with_attrs(query, conn)
            send_line(conn, "done!")

        except ListTooLongError as res_too_long:
            print(res_too_long)
            send_line(conn, f"-- 403 FORBIDDEN -- {repr(res_too_long)}")
            send_list_len_err(conn)
        except lbl.RequestError as req_err:
            print(req_err)
            send_line(conn, f"-- 422 UNPROCESSABLE CONTENT -- {repr(req_err)}")
        except lbl.HTTPError as lb_serr:
            print(lb_serr)
            send_line(conn, f"-- 502 BAD GATEWAY -- {repr(lb_serr)}")
        except Exception as e:
            print(e)
            send_line(conn, f"-- 500 INTERNAL SERVER ERROR -- {repr(e)}")
        finally:
            send_line(conn, "done!")
            conn.close()    # sends EOF, so that Rust server can read data sent
            continue

        print("\n\033[0;32mRetrival complete, and data sent!\033[0m\n")

if __name__ == "__main__":
    main()
