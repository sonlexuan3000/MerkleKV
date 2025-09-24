import argparse
import re
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

def open_conn(host, port):
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.connect((host, port))
    return s

# ---------- NEW TESTS: HASH ----------
def test_hash(main_host, main_port):
    s = open_conn(main_host, main_port)
    try:
        # Seed some keys
        assert send_cmd(s, "SET user:1 Alice") == "OK"
        assert send_cmd(s, "SET user:2 Bob") == "OK"
        assert send_cmd(s, "SET misc:color blue") == "OK"
        assert send_cmd(s, "SET sys:version 1.0") == "OK"

        # HASH full
        full1 = send_cmd(s, "HASH")
        assert re.match(r"^HASH\s+[0-9a-f]{64}$", full1), f"Bad HASH full: {full1!r}"

        # HASH theo prefix
        user1 = send_cmd(s, "HASH user:")
        assert re.match(r"^HASH\s+user:\s+[0-9a-f]{64}$", user1), f"Bad HASH user: {user1!r}"

        # Đổi 1 key trong user: => HASH user: phải đổi
        assert send_cmd(s, "SET user:2 Robert") == "OK"
        user2 = send_cmd(s, "HASH user:")
        assert re.match(r"^HASH\s+user:\s+[0-9a-f]{64}$", user2), f"Bad HASH user(2): {user2!r}"
        assert user1 != user2, "Expected user: hash to change after value update"

        # Full hash cũng có thể thay đổi
        full2 = send_cmd(s, "HASH")
        assert re.match(r"^HASH\s+[0-9a-f]{64}$", full2), f"Bad HASH full(2): {full2!r}"
        # Không bắt buộc phải khác, nhưng thường sẽ khác:
        # if full1 == full2: print("Note: full hash unchanged (unexpected but not fatal)")

        print("✓ HASH tests OK")
    finally:
        s.close()

# ---------- NEW TESTS: SYNC ----------
def test_sync(local_host, local_port, remote_host, remote_port, verify=False):
    # local: nơi ta gửi lệnh SYNC; remote: nguồn dữ liệu
    sl = open_conn(local_host, local_port)
    sr = open_conn(remote_host, remote_port)
    try:
        # Làm sạch
        assert send_cmd(sl, "FLUSHDB") == "OK"
        assert send_cmd(sr, "FLUSHDB") == "OK"

        # Seed remote một ít key
        assert send_cmd(sr, "SET r:1 Ralpha") == "OK"
        assert send_cmd(sr, "SET r:2 Rbeta") == "OK"
        assert send_cmd(sr, "SET user:1 U-Alice") == "OK"

        # Seed local có key riêng để kiểm chứng bị xóa sau SYNC
        assert send_cmd(sl, "SET onlylocal:1 X") == "OK"
        assert send_cmd(sl, "GET r:1") == "NOT_FOUND"

        # SYNC: local <- remote
        cmd = f"SYNC {remote_host} {remote_port}" + (" --verify" if verify else "")
        resp = send_cmd(sl, cmd)
        assert resp == "OK", f"SYNC resp={resp!r}"

        # Sau SYNC: local đã có key của remote
        assert send_cmd(sl, "GET r:1") == "VALUE Ralpha"
        assert send_cmd(sl, "GET r:2") == "VALUE Rbeta"
        assert send_cmd(sl, "GET user:1") == "VALUE U-Alice"

        # Key chỉ có ở local trước đó phải bị xóa
        assert send_cmd(sl, "GET onlylocal:1") == "NOT_FOUND"

        print("✓ SYNC tests OK")
    finally:
        sl.close()
        sr.close()

# ---------- NEW TESTS: REPLICATE ----------
def test_replicate(main_host, main_port):
    s = open_conn(main_host, main_port)
    try:
        st = send_cmd(s, "REPLICATE status")
        assert st.startswith("REPLICATION "), f"Bad status: {st!r}"

        # Disable -> status phải báo disabled
        assert send_cmd(s, "REPLICATE disable") == "OK"
        st2 = send_cmd(s, "REPLICATE status")
        assert "disabled" in st2.lower(), f"Expected disabled in: {st2!r}"

        # Enable -> status phải báo enabled
        assert send_cmd(s, "REPLICATE enable") == "OK"
        st3 = send_cmd(s, "REPLICATE status")
        assert "enabled" in st3.lower(), f"Expected enabled in: {st3!r}"

        print("✓ REPLICATE tests OK")
    finally:
        s.close()

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--host", default="127.0.0.1")
    ap.add_argument("--port", type=int, default=7379)
    ap.add_argument("--peer-host", default=None, help="Peer host để test SYNC")
    ap.add_argument("--peer-port", type=int, default=None, help="Peer port để test SYNC")
    ap.add_argument("--verify", action="store_true", help="Thêm --verify vào lệnh SYNC")
    args = ap.parse_args()

    # Kết nối node chính để chạy các test cũ
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    try:
        s.connect((args.host, args.port))
    except OSError as e:
        raise SystemExit(
            f"Cannot connect to {(args.host, args.port)}: {e}. "
            "Run server first (cargo run --release -- --config quickstart.toml)"
        )

    # ======= TEST CŨ (giữ nguyên) =======
    # 1) FLUSHDB -> OK
    resp = send_cmd(s, "FLUSHDB")
    assert resp == "OK", f"FLUSHDB resp={resp!r}"

    # 2) PING/ECHO
    resp = send_cmd(s, "PING hello")
    assert resp == "PONG: hello", f"PING resp={resp!r}"

    resp = send_cmd(s, "ECHO test")
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

    # 5) EXISTS (1-key cho chắc chắn tương thích parser)
    rx = send_cmd(s, "EXISTS a")
    assert rx.startswith("EXISTS "), f"EXISTS bad: {rx!r}"
    assert int(rx.split()[1]) in (0, 1), f"EXISTS value bad: {rx!r}"

    # 6) MEMORY
    m = send_cmd(s, "MEMORY")
    assert m.startswith("MEMORY "), f"MEMORY bad: {m!r}"
    mem_bytes = int(m.split()[1])
    assert mem_bytes >= 0

    # 7) DELETE + DBSIZE lại
    assert send_cmd(s, "DELETE a") in ("DELETED", "NOT_FOUND")
    resp = send_cmd(s, "DBSIZE")
    n2 = int(resp.split()[1])
    assert n2 >= 1, f"DBSIZE after delete should be >=1 (because 'b' still there), got {n2}"

    print("✓ Base tests passed.")
    s.close()

    # ======= NEW TESTS =======
    test_hash(args.host, args.port)

    if args.peer_host and args.peer_port:
        test_sync(args.host, args.port, args.peer_host, args.peer_port, verify=args.verify)
    else:
        print("✓ SYNC tests skipped (no --peer-host/--peer-port)")

    # REPLICATE test (không phụ thuộc broker nếu server chỉ toggle trạng thái)
    test_replicate(args.host, args.port)

    print("✓ All integration tests passed.")

if __name__ == "__main__":
    main()
