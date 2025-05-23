"""
A simple health check for the Python app's Docker container.
"""
import sys
import socket

def is_healthy() -> bool:
    """
    Performs the healthcheck.
    """
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)

    try:
        sock.connect(("0.0.0.0", 3575))

        ping_msg  = bytes('{"msg": "are you healthy?"}', 'utf-8')
        sock.sendall(ping_msg)
        sock.recv(2)            # recieve and discard message length bytes (it's gonna be 19)
        response  = sock.recv(19).decode("utf-8")
        sock.recv(7)            # get done signal with length bytes (it's gonna be 2+5 == 7)

        if response != "yep, still healthy!":
            print("[ \033[0;31mHEALTHCHECK FAILED\033[0m ]: Python app running, but sent "
                f"unexpected response to healthcheck ping: \"{response}\"",
                file=sys.stderr)
            sock.close()  # redundant, but won't throw error
            return False

        sock.close()
        return True

    except ConnectionRefusedError:
        print("[ \033[0;31mHEALTHCHECK FAILED\033[0m ]: Python app no longer running!",
            file=sys.stderr)
        return False


def main():
    """
    Just handles exiting with the right exit code.
    """
    if not is_healthy():
        sys.exit(1)

    sys.exit(0)

if __name__ == "__main__":
    main()