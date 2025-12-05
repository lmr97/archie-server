import signal
import socket

DEBUG_PRINT = False

class InterruptHandler:
    """
    You'll never believe this, but this class handles OS signals.
    """
    def __init__(self, main_listener: socket.socket):

        self.kill_now = False
        # keep a copy of the main connection to close on exit
        self._main_listener = main_listener

        # define listeners for graceful exit if in main thread of interpreter;
        # otherwise just hold the listener and a shutdown boolean
        if __name__ == "__main__":
            signal.signal(signal.SIGINT,  self._sig_exit_handler)
            signal.signal(signal.SIGTERM, self._sig_exit_handler)
            signal.signal(signal.SIGKILL, self._sig_exit_handler)

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


def to_capital_header(attr: str) -> str:
    """
    Capitalizes the first letter of each word in a given string,
    and replaces the `-` with a space.
    """
    words    = attr.split("-")
    init_cap = [w.capitalize() for w in words]
    uc_attr  = " ".join(init_cap)
    return uc_attr