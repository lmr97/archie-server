"""
Tests the I/O to the server and exception handling. Also tests the 
healthcheck run by Docker on the app. 

Note: I can't really test unexpected errors (or 502 response-type 
errors; the former because I can't expect them, the latter because 
that requires letterboxd.com, specifically, to respond with a 5xx status,
and I can't make that happen on demand. I also can't test the signal 
handlers here, since Python executes all signal handlers in the main thread
of the interpreter, meaning I cannot simply send a signal to a Python thread
to terminate it without terminating the whole interpreter. So coverage will 
be incomplete according to this test, but this functionality is easy to test
from the command line. 
"""
import csv
import os
import json
import pytest
import pandas as pd
import socket
import threading
from letterboxd_list import VALID_ATTRS
import lb_app
import healthcheck

# define socket object to emulate server data and catch bytes streamed out
PY_SOCK    = os.getenv("PY_CONT_SOCK", "127.0.0.1:3575")
(IP, PORT) = PY_SOCK.split(":")
PORT       = int(PORT)
ADDRESS    = (IP, PORT)

TEST_LIST     = pd.read_csv("letterboxd_get_list/letterboxd_list/tests/random-list-test.csv")
MIN_RAND_LIST = TEST_LIST[["Title", "Year"]]\
    .to_csv(index=False, quoting=csv.QUOTE_NONNUMERIC)\
    .split("\n")  # `list` CSV formatted `str`s

# app can be stopped by sending a specific string to the bound port
APP_THREAD_1 = threading.Thread(target=lb_app.main)
APP_THREAD_1.start()


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


def receive_list(connection: socket.socket, correct_len: int) -> list:
    """
    Mimic server's algorithm for receiving a list, with the exception 
    of using a while loop instead of a set-length repeater (that is 
    a requirement of using SSEs in the Axum framework). Length check
    assert statement takes place here for convenience.
    """
    received_len = receive_list_len(connection)
    assert correct_len == received_len

    lb_list      = []
    current_row  = receive_decode_row(connection)
    while current_row != "done!":
        # adding newline for easier comparison against touchstone file read in later
        lb_list.append(current_row+"\n")
        current_row  = receive_decode_row(connection)

    return lb_list


def test_send_line():
    """
    Make sure basic line-sending stream functionality is good.
    Uses a separate port from $PY_CONT_SOCK.
    """
    test_str  = "echo-echo-echo"
    test_sock = ('127.0.0.1', 3020)
    receiver  = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sender    = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    receiver.bind(test_sock)
    receiver.listen(1)
    sender.connect(test_sock)

    lb_app.send_line(sender, test_str)
    (conn, _)    = receiver.accept()
    received_str = receive_decode_row(conn)
    conn.close()

    assert test_str == received_str


def test_check_health_response():
    """
    Makes sure the part of the app that responds to the Docker 
    container healthcheck behaves as expected.
    """
    conn      = send_str('{"msg": "are you healthy?"}')
    response  = receive_decode_row(conn)
    sig_done  = receive_decode_row(conn)    # extra row for "done!" signal
    conn.close()

    assert "yep, still healthy!" == response
    assert sig_done == "done!"



def test_healthy():
    """
    Test that the actual healthcheck script responds as it should 
    when the app is running.
    """
    assert healthcheck.is_healthy()
    with pytest.raises(SystemExit) as se:
        healthcheck.main()

    # ensure the whole script exited with code 0
    assert se.value.code == 0


def test_rej_overlong_list():
    """
    See if the app will reject lists over 10k films long, with a 403 error.
    """
    # this list is over 10k films long!
    too_long_list = {
        "list_name": "monster-mega-list-2-actual-watch-list-1",
        "author_user": "maxwren",
        "attrs": ["director", "assistant-director"]
    }
    req_str       = json.dumps(too_long_list)
    conn          = send_str(req_str)

    # check that list length received is 0
    recvd_length  = receive_list_len(conn)
    assert recvd_length == 0

    received_str  = receive_decode_row(conn)
    done_signal   = receive_decode_row(conn)    # extra row for "done!" signal
    conn.close()

    assert "-- 403 FORBIDDEN --" in received_str
    assert done_signal == "done!"


def test_rej_bad_attr():
    """
    Testing whether the app rejects a request with an invalid attribute, 
    even if the request is properly formed.

    Genuinely malformed requests are rejected by the server before they 
    even get to the Python app, so they are tested elsewhere.
    """
    # real list, fake attribute
    bad_attr = {
        "list_name": "truly-random-films",
        "author_user": "dialectica972",
        "attrs": ["director", "bingus"]
    }
    request_str   = json.dumps(bad_attr)
    conn          = send_str(request_str)

    # check that list length received is 0
    recvd_length  = receive_list_len(conn)
    assert recvd_length == 0

    received_str  = receive_decode_row(conn)
    done_signal   = receive_decode_row(conn)  # extra row of "done!" sent by app
    conn.close()

    assert "-- 422 UNPROCESSABLE CONTENT --" in received_str
    assert done_signal == "done!"


def test_rej_bad_list():
    """
    Testing whether the app rejects a request with an nonexistent list, 
    even if the request is properly formed.

    Genuinely malformed requests are rejected by the server before they 
    even get to the Python app, so they are tested elsewhere.
    """
    # this list doesn't exist
    bad_list = {
        "list_name": "dfgjsdflkgdf",
        "author_user": "sdfkjshd",
        "attrs": ["director", "watches"]
    }
    request_str   = json.dumps(bad_list)
    conn          = send_str(request_str)

    # check that list length received is 0
    recvd_length  = receive_list_len(conn)
    assert recvd_length == 0

    received_str  = receive_decode_row(conn)
    done_signal   = receive_decode_row(conn)  # extra row of "done!" sent
    conn.close()

    assert "-- 422 UNPROCESSABLE CONTENT --" in received_str
    assert done_signal == "done!"


def test_long_rows():
    """
    Test a short list, but with long rows. The request will contain all 
    attributes, except for statistics rows (those change frequently).
    """
    vattrs_no_stats = VALID_ATTRS.copy()   # keep module-level global in tact in this scope
    vattrs_no_stats.remove("watches")
    vattrs_no_stats.remove("likes")
    vattrs_no_stats.remove("avg-rating")

    megarow_list = {
        "list_name": "test-list-all-attributes",
        "author_user": "dialectica972",
        "attrs": vattrs_no_stats
    }

    request_str  = json.dumps(megarow_list)
    connection   = send_str(request_str)
    lb_list      = receive_list(connection, correct_len=3)
    
    correct_list = []
    with open("short-list-all-attrs-no-stats.csv", "r", encoding="utf-8") as list_reader:
        correct_list = list_reader.readlines()

    films_in_list = [
        "Avengers: Endgame",
        "Top Gun: Maverick",
        "Avatar: The Way of Water"
        ]
    columns = correct_list[0].split(",")
    # checking row by row to show where any issues are
    for i, test_row in enumerate(lb_list):

        true_row = correct_list[i]

        # Sometimes the rows are not exactly identical only because the order of some names
        # in attributes may change on Letterboxd.com itself. This means that the test values
        # are fetched from different database than the touchstone data. This block is a
        # "closer look" to verify whether the content differs, or simply the order.
        try:
            assert true_row == test_row
        except AssertionError:
            # divide rows into cells, and check each cell for a difference in content.
            true_row = true_row.replace("\n", "")
            test_row = test_row.replace("\n", "")
            for j, (true_val, test_val) in enumerate(zip(true_row.split(","), test_row.split(","))):
                true_no_quotes = true_val.replace("\"","")
                test_no_quotes = test_val.replace("\"","")

                # if this fails, then there IS a meaningful difference between cell values.
                assert set(true_no_quotes.split("; ")) == set(test_no_quotes.split("; ")), \
                    f"film \"{films_in_list[i-1]}\" failed assertion on field {columns[j]}"


def test_list_order():
    """
    Request no attributes, get titles and years only. This makes the retrived 
    list easier to compare to a test list.
    """
    min_list_req = {
        "list_name": "truly-random-films",
        "author_user": "dialectica972",
        "attrs": ["none"]
    }

    request_str   = json.dumps(min_list_req)
    connection    = send_str(request_str)
    lb_list_recvd = receive_list(connection, correct_len=49)

    # checking row by row to show where any issues are
    for (idx, (true_row, test_row)) in enumerate(zip(MIN_RAND_LIST, lb_list_recvd)):

        # first row is simply the header
        if idx == 0:
            continue
        
        # receive_list() appends newline on all rows
        assert true_row+"\n" == test_row, \
            f"test failed for row {idx+1}. Was {test_row}, should have been {true_row}."
        

def test_null_attributes():
    minirow_list = {
        "list_name": "test-list-all-attributes",
        "author_user": "dialectica972",
        "attrs": ["none"]
    }

    request_str  = json.dumps(minirow_list)
    connection   = send_str(request_str)
    lb_list      = receive_list(connection, correct_len=3)

    correct_list = []
    with open("short-list-no-attrs.csv", "r", encoding="utf-8") as list_reader:
        correct_list = list_reader.readlines()

    # checking row by row to show where any issues are
    for i, test_row in enumerate(lb_list):
        assert correct_list[i] == test_row


def test_shutdown_via_msg():
    """
    The app can shut down if sent the JSON string below, if signal-sending
    is ever cumbersome. This tests that functionality.
    """
    conn = send_str('{"msg": "shutdown"}')
    conn.close()
    APP_THREAD_1.join()
    assert not APP_THREAD_1.is_alive()


def test_unhealthy():
    """
    Test that the actual healthcheck script responds as it should 
    when the app is NOT running.
    """
    assert not healthcheck.is_healthy()

    with pytest.raises(SystemExit) as se:
        healthcheck.main()

    # ensure the whole script exited with code 1
    assert se.value.code == 1