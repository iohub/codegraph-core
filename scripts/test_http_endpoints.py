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

# Color codes for terminal output
class Colors:
    HEADER = '\033[95m'
    OKBLUE = '\033[94m'
    OKCYAN = '\033[96m'
    OKGREEN = '\033[92m'
    WARNING = '\033[93m'
    FAIL = '\033[91m'
    ENDC = '\033[0m'
    BOLD = '\033[1m'
    UNDERLINE = '\033[4m'

def print_header(text: str):
    """Print a formatted header"""
    print(f"\n{Colors.HEADER}{Colors.BOLD}{'='*60}")
    print(f"  {text}")
    print(f"{'='*60}{Colors.ENDC}")

def print_step(text: str):
    """Print a step indicator"""
    print(f"\n{Colors.OKBLUE}{Colors.BOLD}▶ {text}{Colors.ENDC}")

def print_success(text: str):
    """Print a success message"""
    print(f"{Colors.OKGREEN}{Colors.BOLD}✓ {text}{Colors.ENDC}")

def print_info(text: str):
    """Print an info message"""
    print(f"{Colors.OKCYAN}ℹ {text}{Colors.ENDC}")

def print_warning(text: str):
    """Print a warning message"""
    print(f"{Colors.WARNING}⚠ {text}{Colors.ENDC}")

def print_error(text: str):
    """Print an error message"""
    print(f"{Colors.FAIL}✗ {text}{Colors.ENDC}")

def print_progress(current: int, total: int, description: str = ""):
    """Print a progress bar"""
    bar_length = 40
    filled_length = int(bar_length * current // total)
    bar = '█' * filled_length + '░' * (bar_length - filled_length)
    percentage = current * 100 // total
    print(f"\r{Colors.OKCYAN}[{bar}] {percentage:3d}% {description}{Colors.ENDC}", end='', flush=True)
    if current == total:
        print()  # New line when complete

def print_json_pretty(data: dict, title: str = ""):
    """Print JSON data in a formatted way"""
    if title:
        print(f"\n{Colors.BOLD}{title}:{Colors.ENDC}")
    print(json.dumps(data, indent=2, ensure_ascii=False))

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
    print_info(f"Starting server: {' '.join(cmd)}")
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

    print_header("CodeGraph HTTP Endpoints Test Suite")
    print_info(f"Testing endpoints at: {BASE_URL}")
    print_info(f"Project root: {REPO_ROOT}")

    # Build first to fail fast if compilation issues
    print_step("Building project (cargo build)...")
    subprocess.check_call(["cargo", "build", "--quiet"], cwd=str(REPO_ROOT))
    print_success("Project built successfully")

    proc = start_server()
    try:
        print_step("Waiting for server to start...")
        print_progress(0, 3, "Waiting for TCP port...")
        wait_for_port(host, port, timeout=30)
        print_progress(1, 3, "TCP port ready")
        
        print_progress(1, 3, "Waiting for /health endpoint...")
        wait_for_health(timeout=30)
        print_progress(2, 3, "Health check passed")
        
        print_progress(3, 3, "Server is ready")
        print_success("Server is healthy and ready for testing")

        # 1) build_graph
        print_header("Test 1: Build Graph Endpoint")
        project_dir = str(REPO_ROOT)
        build_payload = {
            "project_dir": project_dir,
            "force_rebuild": True,
            "exclude_patterns": ['target', '.git', '.venv'],
        }
        print_info(f"POST /build_graph")
        print_json_pretty(build_payload, "Request Payload")
        print_warning("Note: This may take several minutes for large projects...")
        
        # Make the request with a long timeout
        r = requests.post(f"{BASE_URL}/build_graph", json=build_payload, timeout=600)
        
        # Read any available output after the request completes
        output = read_proc_output_nonblocking(proc)
        if output:
            print_info("Server output during build_graph:")
            print(f"{Colors.OKCYAN}{output}{Colors.ENDC}")
        
        assert_true(r.status_code == 200, f"/build_graph HTTP {r.status_code}: {r.text}")
        j = r.json()
        assert_true(j.get("success") is True, f"/build_graph success=false: {j}")
        data = j.get("data", {})
        assert_true("project_id" in data, "Missing project_id in build_graph response")
        print_success("/build_graph endpoint test passed")
        print_info(f"Project ID: {data.get('project_id', 'N/A')}")

        # Choose a Rust file that certainly exists and has functions
        filepath = str(REPO_ROOT / "src/codegraph/graph.rs")
        print_info(f"Using test file: {filepath}")

        # 2) query_call_graph
        print_header("Test 2: Query Call Graph Endpoint")
        query_payload = {
            "filepath": filepath,
            # Leave function_name None to get all functions in file
            "max_depth": 2,
        }
        print_info(f"POST /query_call_graph")
        print_json_pretty(query_payload, "Request Payload")
        
        r = requests.post(f"{BASE_URL}/query_call_graph", json=query_payload, timeout=60)
        
        # Read any available output
        output = read_proc_output_nonblocking(proc)
        if output:
            print_info("Server output during query_call_graph:")
            print(f"{Colors.OKCYAN}{output}{Colors.ENDC}")
            
        assert_true(r.status_code == 200, f"/query_call_graph HTTP {r.status_code}: {r.text}")
        j = r.json()
        assert_true(j.get("success") is True, f"/query_call_graph success=false: {j}")
        data = j.get("data", {})
        functions = data.get("functions", [])
        print_success("/query_call_graph endpoint test passed")
        print_info(f"Found {len(functions)} functions in call graph")

        # 3) query_code_snippet (use first function name if available)
        print_header("Test 3: Query Code Snippet Endpoint")
        function_name = functions[0]["name"] if functions else None
        snippet_payload = {
            "filepath": filepath,
            "function_name": function_name,
            "include_context": True,
            "context_lines": 2,
        }
        print_info(f"POST /query_code_snippet")
        print_json_pretty(snippet_payload, "Request Payload")
        
        r = requests.post(f"{BASE_URL}/query_code_snippet", json=snippet_payload, timeout=60)
        
        # Read any available output
        output = read_proc_output_nonblocking(proc)
        if output:
            print_info("Server output during query_code_snippet:")
            print(f"{Colors.OKCYAN}{output}{Colors.ENDC}")
            
        assert_true(r.status_code == 200, f"/query_code_snippet HTTP {r.status_code}: {r.text}")
        j = r.json()
        assert_true(j.get("success") is True, f"/query_code_snippet success=false: {j}")
        data = j.get("data", {})
        assert_true("code_snippet" in data, "Missing code_snippet in snippet response")
        print_success("/query_code_snippet endpoint test passed")

        # 4) query_code_skeleton
        print_header("Test 4: Query Code Skeleton Endpoint")
        skeleton_payload = {
            "filepaths": [filepath],
        }
        print_info(f"POST /query_code_skeleton")
        print_json_pretty(skeleton_payload, "Request Payload")
        
        r = requests.post(f"{BASE_URL}/query_code_skeleton", json=skeleton_payload, timeout=60)
        
        # Read any available output
        output = read_proc_output_nonblocking(proc)
        if output:
            print_info("Server output during query_code_skeleton:")
            print(f"{Colors.OKCYAN}{output}{Colors.ENDC}")
            
        assert_true(r.status_code == 200, f"/query_code_skeleton HTTP {r.status_code}: {r.text}")
        j = r.json()
        print_info("Skeleton text preview:")
        skeletons = j["data"].get("skeletons", [])
        assert_true(len(skeletons) > 0, "No skeletons returned from query_code_skeleton")
        skeleton_text = skeletons[0].get("skeleton_text", "")
        # Truncate if too long
        if len(skeleton_text) > 200:
            print(f"{Colors.OKCYAN}{skeleton_text[:500]}...{Colors.ENDC}")
        else:
            print(f"{Colors.OKCYAN}{skeleton_text}{Colors.ENDC}")
            
        assert_true(j.get("success") is True, f"/query_code_skeleton success=false: {j}")
        print_success("/query_code_skeleton endpoint test passed")
        
        print_header("Test Results Summary")
        print_success("All endpoint tests passed successfully! 🎉")
        print_info("✅ /build_graph")
        print_info("✅ /query_call_graph") 
        print_info("✅ /query_code_snippet")
        print_info("✅ /query_code_skeleton")
        
    finally:
        print_step("Shutting down server...")
        stop_server(proc)
        # Drain remaining output
        tail = read_proc_output_nonblocking(proc, limit_lines=1000)
        if tail:
            print_info("Final server output:")
            print(f"{Colors.OKCYAN}{tail}{Colors.ENDC}")


if __name__ == "__main__":
    try:
        run_tests()
    except Exception as e:
        print_error(f"TEST FAILED: {e}")
        sys.exit(1) 