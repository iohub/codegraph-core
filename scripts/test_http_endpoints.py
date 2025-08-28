#!/usr/bin/env python3
import os
import sys
import json
import time
import signal
import socket
import subprocess
import select
from pathlib import Path

try:
    import requests
except ImportError:
    print("Python package 'requests' is required. Installing locally...")
    subprocess.check_call([sys.executable, "-m", "pip", "install", "--user", "requests"])
    import requests

REPO_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_ADDR = os.environ.get("CODEGRAPH_HTTP_ADDR", "127.0.0.1:8081")
BASE_URL = f"http://{DEFAULT_ADDR}"


def wait_for_port(host: str, port: int, timeout: float = 30.0) -> None:
    deadline = time.time() + timeout
    last_err = None
    while time.time() < deadline:
        try:
            with socket.create_connection((host, port), timeout=1.0):
                return
        except OSError as e:
            last_err = e
            time.sleep(0.2)
    raise TimeoutError(f"Timed out waiting for {host}:{port} to accept connections: {last_err}")


def wait_for_health(timeout: float = 30.0) -> None:
    deadline = time.time() + timeout
    while time.time() < deadline:
        try:
            r = requests.get(f"{BASE_URL}/health", timeout=2.0)
            if r.status_code == 200:
                j = r.json()
                if j.get("success") is True:
                    return
        except Exception:
            pass
        time.sleep(0.2)
    raise TimeoutError("Timed out waiting for /health to become ready")


def start_server() -> subprocess.Popen:
    env = os.environ.copy()
    cmd = [
        "cargo", "run", "--quiet", "--",
        "server", "--address", DEFAULT_ADDR,
    ]
    print("Starting server:", " ".join(cmd))
    proc = subprocess.Popen(
        cmd,
        cwd=str(REPO_ROOT),
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
        env=env,
    )
    return proc


def stop_server(proc: subprocess.Popen) -> None:
    if proc.poll() is not None:
        return
    try:
        # Try graceful first
        proc.send_signal(signal.SIGINT)
        for _ in range(50):
            if proc.poll() is not None:
                return
            time.sleep(0.1)
        # Force kill
        proc.kill()
    except Exception:
        pass


def read_proc_output_nonblocking(proc: subprocess.Popen, limit_lines: int = 200) -> str:
    """Read process output without blocking, using select for non-blocking I/O"""
    content = []
    if proc.stdout is None:
        return ""
    
    try:
        # Use select to check if there's data available (non-blocking)
        ready, _, _ = select.select([proc.stdout], [], [], 0.1)
        if not ready:
            return ""
        
        for _ in range(limit_lines):
            # Check if there's data available before reading
            if select.select([proc.stdout], [], [], 0.0)[0]:
                line = proc.stdout.readline()
                if not line:
                    break
                content.append(line.rstrip())
            else:
                break
    except Exception:
        pass
    
    return "\n".join(content)


def assert_true(condition: bool, message: str):
    if not condition:
        raise AssertionError(message)


def run_tests():
    host, port_str = DEFAULT_ADDR.split(":")
    port = int(port_str)

    # Build first to fail fast if compilation issues
    print("Building project (cargo build)...")
    subprocess.check_call(["cargo", "build", "--quiet"], cwd=str(REPO_ROOT))

    proc = start_server()
    try:
        print("Waiting for TCP port...")
        wait_for_port(host, port, timeout=30)
        print("Waiting for /health...")
        wait_for_health(timeout=30)
        print("Server is healthy.")

        # 1) build_graph
        project_dir = str(REPO_ROOT)
        build_payload = {
            "project_dir": project_dir,
            "force_rebuild": True,
            "exclude_patterns": ['target', '.git', '.venv'],
        }
        print(f"POST /build_graph {build_payload}")
        print("Note: This may take several minutes for large projects...")
        
        # Make the request with a long timeout
        r = requests.post(f"{BASE_URL}/build_graph", json=build_payload, timeout=600)
        
        # Read any available output after the request completes
        output = read_proc_output_nonblocking(proc)
        if output:
            print("Server output during build_graph:")
            print(output)
        
        assert_true(r.status_code == 200, f"/build_graph HTTP {r.status_code}: {r.text}")
        j = r.json()
        assert_true(j.get("success") is True, f"/build_graph success=false: {j}")
        data = j.get("data", {})
        assert_true("project_id" in data, "Missing project_id in build_graph response")
        print("/build_graph OK")

        # Choose a Rust file that certainly exists and has functions
        filepath = str(REPO_ROOT / "src/http/handlers/mod.rs")

        # 2) query_call_graph
        query_payload = {
            "filepath": filepath,
            # Leave function_name None to get all functions in file
            "max_depth": 2,
        }
        print(f"POST /query_call_graph {query_payload}")
        r = requests.post(f"{BASE_URL}/query_call_graph", json=query_payload, timeout=60)
        
        # Read any available output
        output = read_proc_output_nonblocking(proc)
        if output:
            print("Server output during query_call_graph:")
            print(output)
            
        assert_true(r.status_code == 200, f"/query_call_graph HTTP {r.status_code}: {r.text}")
        j = r.json()
        assert_true(j.get("success") is True, f"/query_call_graph success=false: {j}")
        data = j.get("data", {})
        functions = data.get("functions", [])
        print(f"Found {len(functions)} functions in call graph")

        # 3) query_code_snippet (use first function name if available)
        function_name = functions[0]["name"] if functions else None
        snippet_payload = {
            "filepath": filepath,
            "function_name": function_name,
            "include_context": True,
            "context_lines": 2,
        }
        print("POST /query_code_snippet ...")
        r = requests.post(f"{BASE_URL}/query_code_snippet", json=snippet_payload, timeout=60)
        
        # Read any available output
        output = read_proc_output_nonblocking(proc)
        if output:
            print("Server output during query_code_snippet:")
            print(output)
            
        assert_true(r.status_code == 200, f"/query_code_snippet HTTP {r.status_code}: {r.text}")
        j = r.json()
        assert_true(j.get("success") is True, f"/query_code_snippet success=false: {j}")
        data = j.get("data", {})
        assert_true("code_snippet" in data, "Missing code_snippet in snippet response")
        print("/query_code_snippet OK")

        # 4) query_code_skeleton
        skeleton_payload = {
            "filepath": filepath,
        }
        print("POST /query_code_skeleton ...")
        r = requests.post(f"{BASE_URL}/query_code_skeleton", json=skeleton_payload, timeout=60)
        
        # Read any available output
        output = read_proc_output_nonblocking(proc)
        if output:
            print("Server output during query_code_skeleton:")
            print(output)
            
        assert_true(r.status_code == 200, f"/query_code_skeleton HTTP {r.status_code}: {r.text}")
        j = r.json()
        assert_true(j.get("success") is True, f"/query_code_skeleton success=false: {j}")
        print("/query_code_skeleton OK")

        print("All endpoint tests passed.")
    finally:
        print("Shutting down server...")
        stop_server(proc)
        # Drain remaining output
        tail = read_proc_output_nonblocking(proc, limit_lines=1000)
        if tail:
            print("Final server output:")
            print(tail)


if __name__ == "__main__":
    try:
        run_tests()
    except Exception as e:
        print(f"TEST FAILED: {e}")
        sys.exit(1) 