import sys
import subprocess
from threading import Thread

def main():
    if len(sys.argv) != 3:
        print("Usage: python team.py <hex_value> <port>")
        sys.exit(1)

    hex_value = sys.argv[1]
    port = int(sys.argv[2])

    # Run main.py with the given port and port + 1 in parallel

    def run_script(port_offset):
        subprocess.run(["python3", "main.py", hex_value, str(port + port_offset)])

    thread1 = Thread(target=run_script, args=(0,))
    thread2 = Thread(target=run_script, args=(1,))

    thread1.start()
    thread2.start()

    thread1.join()
    thread2.join()

if __name__ == "__main__":
    main()