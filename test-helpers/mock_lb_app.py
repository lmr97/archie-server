import json
import socket

# parameters for streaming data to server
ENDIANNESS = 'big'
SIZE_BYTES = 2

class RequestError(Exception):
    """
    to distinguish my syntax errors from genuine 
    request errors made by client
    """

class LbServerError(Exception):
    """
    For catching errors from letterboxd.com
    """

class ListTooLongError(RequestError):
    """
    Special error for when the Letterboxd list is 
    over 65,500 films. This is not only an ungodly 
    amount of films for a list, but also the 2-byte limit
    to the amount the server can process (given its code,
    not necessarily the computer's processing power).
    """

def send_list_len(list_len: int, conn: socket.socket):
    """
    Simplified version
    """

    int_as_bytes = list_len.to_bytes(SIZE_BYTES, ENDIANNESS)
    conn.sendall(int_as_bytes)


def send_line(conn: socket.socket, line: str):
    """
    This function is identical to the one in get_list.py; 
    see that function for notes.
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


def main():
    """Mocks up responses to """
    listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    listener.bind(('0.0.0.0', 3575))
    listener.listen(1)
    print("Starting mock Python app...")

    try:
        # only expecting 6 messages; this will allow the app to terminate
        # automatically and gracefully once the tests are complete.
        for i in range(7):

            (conn, _) = listener.accept()
            req = conn.recv(2048).decode("utf-8")
            query = json.loads(req)
            print(query)

            if (query['list_name'] == "this-hurts-you"):
                crashing_err = Exception("A crash of some sort")
                send_list_len(0, conn)
                send_line(conn, f"-- 500 INTERNAL SERVER ERROR -- {repr(crashing_err)}")
                send_line(conn, "done!")
                continue

            if (query["list_name"] == "server-down"):
                lb_serr = RequestError(
                    "Letterboxd server error occurred in fetching webpage. Response status: 500"
                    )
                send_list_len(0, conn)
                send_line(conn, f"-- 502 BAD GATEWAY -- {repr(lb_serr)}")
                send_line(conn, "done!")
                continue

            if (query["list_name"] == "list-no-exist"):
                req_err = RequestError("Error in fetching webpage. Response status: 404")
                send_list_len(0, conn)
                send_line(conn, f"-- 422 UNPROCESSABLE CONTENT -- {repr(req_err)}")
                send_line(conn, "done!")
                continue

            if ("bingus" in query["attrs"]):
                req_err = RequestError(
                    f"Invalid attributes submitted. All submitted attributes: {query["attrs"]}"
                    )
                send_list_len(0, conn)
                send_line(conn, f"-- 422 UNPROCESSABLE CONTENT -- {repr(req_err)}")
                send_line(conn, "done!")
                continue
            
            if (query["list_name"] == "list-too-long"):
                req_err = RequestError("ListTooLongError")
                send_list_len(0, conn)
                send_line(conn, f"-- 403 FORBIDDEN -- {repr(req_err)}")
                send_line(conn, "done!")
                continue

            if (query["attrs"][0] == 'none'):

                titles = ["2001: A Space Odyssey", "Blade Runner",
                    "The Players vs. Ángeles Caídos", "8½"]
                years  = ["1968", "1982", "1969", "1963"]

                total_len = len(titles)
                send_list_len(total_len, conn)
                send_line(conn, "Title,Year")

                for i in range(total_len):
                    print(   f"DEBUG: {titles[i]},{years[i]}")
                    send_line(conn, f"{titles[i]},{years[i]}")
                
                send_line(conn, "done!")
                continue

            if (len(query["attrs"]) > 20):           # proxy for "all attributes"
                continue

            conn.close()    # sends EOF, so that Rust server can read data sent
            continue
    except Exception as e:
        print(e)
        conn.close()

if (__name__ == "__main__"):
    main()
