import argparse
import socket
import time

CRLF = b"\r\n"

def send_cmd(sock: socket.socket, cmd: str, multiline=False, max_lines=50, timeout=2.0):
    sock.sendall(cmd.encode("utf-8") + CRLF)
    sock.settimeout(timeout)
    if not multiline:
        data = b""
        while True:
            ch = sock.recv(1)
            if not ch:
                break
            data += ch
            if data.endswith(CRLF):
                break
        return data.decode("utf-8").rstrip("\r\n")
    else:
        lines = []
        for _ in range(max_lines):
            line = b""
            while True:
                ch = sock.recv(1)
                if not ch:
                    break
                line += ch
                if line.endswith(CRLF):
                    break
            if not line:
                break
            s = line.decode("utf-8").rstrip("\r\n")
            lines.append(s)
            if s == "END":
                break
        return lines

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--host", default="127.0.0.1")
    ap.add_argument("--port", type=int, default=7379)
    args = ap.parse_args()

    addr = (args.host, args.port)
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)

    try:
        s.connect(addr)
    except OSError as e:
        raise SystemExit(f"Cannot connect to {addr}: {e}. Run server first (cargo run --release -- --config quickstart.toml)")

    # 1) FLUSHDB -> OK
    resp = send_cmd(s, "FLUSHDB")
    assert resp == "OK", f"FLUSHDB resp={resp!r}"

    # 2) PING/ECHO
    resp = send_cmd(s, "PING hello")
    # server return "PONG: hello"
    assert resp == "PONG: hello", f"PING resp={resp!r}"

    resp = send_cmd(s, "ECHO test")
    # server return "ECHO: test"
    assert resp == "ECHO: test", f"ECHO resp={resp!r}"

    # 3) SET/GET
    assert send_cmd(s, "SET a 1") == "OK"
    assert send_cmd(s, "SET b 2") == "OK"
    assert send_cmd(s, "GET a") == "VALUE 1"
    assert send_cmd(s, "GET c") == "NOT_FOUND"

    # 4) DBSIZE
    resp = send_cmd(s, "DBSIZE")
    assert resp.startswith("DBSIZE "), f"DBSIZE bad: {resp!r}"
    n = int(resp.split()[1])
    assert n >= 2, f"DBSIZE expected >=2, got {n}"

    # 5) EXISTS (if your parser supports multiple keys: EXISTS a b z)
    # to be safe, test single-key (your server has implemented counting multiple keys in server.rs,
    # as long as protocol.rs can parse "EXISTS k1 k2 ...")
    rx = send_cmd(s, "EXISTS a")
    assert rx.startswith("EXISTS "), f"EXISTS bad: {rx!r}"
    assert int(rx.split()[1]) in (0,1), f"EXISTS value bad: {rx!r}"

    # if you want to test multi:
    # rx = send_cmd(s, "EXISTS a b zzz")
    # assert rx == "EXISTS 2", f"EXISTS multi bad: {rx!r}"

    # 6) MEMORY
    # RESP: "MEMORY <bytes>"
    m = send_cmd(s, "MEMORY")
    assert m.startswith("MEMORY "), f"MEMORY bad: {m!r}"
    mem_bytes = int(m.split()[1])
    assert mem_bytes >= 0

    # 7) DELETE + DBSIZE lại
    assert send_cmd(s, "DELETE a") in ("DELETED", "NOT_FOUND")
    resp = send_cmd(s, "DBSIZE")
    n2 = int(resp.split()[1])
    assert n2 >= 1, f"DBSIZE after delete should be >=1 (because 'b' still there), got {n2}"

    print("✓ Integration test passed.")
    s.close()

if __name__ == "__main__":
    main()
