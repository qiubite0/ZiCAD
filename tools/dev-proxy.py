# -*- coding: utf-8 -*-
"""
开发服务器：静态文件 (dist/web) + AI 平台转发 (/llmi?u=<urlencoded-endpoint>)。
浏览器跨域限制下，网页无法直连各家 LLM API；所有请求经此同源转发。
用法: python tools/dev-proxy.py  (默认 0.0.0.0:8771，静态目录 dist/web)
"""
import sys
import urllib.parse
import urllib.request
import urllib.error
import json
import os
import subprocess
import tempfile
import threading
from pathlib import Path
from http.server import ThreadingHTTPServer, SimpleHTTPRequestHandler

ROOT = Path(__file__).resolve().parents[1]
STATIC_DIR = sys.argv[1] if len(sys.argv) > 1 else str(ROOT / "dist/web")
PORT = int(sys.argv[2]) if len(sys.argv) > 2 else 8771
HOP_HEADERS = {"connection", "keep-alive", "transfer-encoding", "upgrade", "host"}
MAX_DXF_BYTES = 64 * 1024 * 1024
CONVERSIONS = threading.BoundedSemaphore(2)
CLI = Path(os.environ.get("CADCRAFT_CLI", str(ROOT / ("target/debug/cadcraft-cli.exe" if os.name == "nt" else "target/debug/cadcraft-cli"))))


def convert_dxf(payload, version):
    """Convert in an isolated temp directory; never interpolate uploads into shell commands."""
    if version not in {"2000", "2004", "2007", "2010", "2013", "2018"}:
        raise ValueError("Unsupported DWG version")
    if not payload or len(payload) > MAX_DXF_BYTES:
        raise ValueError("DXF must contain between 1 byte and 64 MB")
    if not CLI.is_file():
        raise RuntimeError("DWG converter unavailable; build with cargo build -p cadcraft-cli")
    with tempfile.TemporaryDirectory(prefix="cadcraft-dwg-") as folder:
        source, dest = Path(folder) / "input.dxf", Path(folder) / "output.dwg"
        source.write_bytes(payload)
        command = "saveas " + json.dumps({"path": str(dest), "version": version})
        result = subprocess.run([str(CLI), "run", str(source), "--cmd", command],
                                capture_output=True, timeout=120, creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
        if result.returncode or not dest.is_file():
            detail = (result.stderr or result.stdout).decode("utf-8", errors="replace")[-2000:]
            raise ValueError("DXF to DWG failed: " + detail)
        if dest.stat().st_size > MAX_DXF_BYTES * 4:
            raise ValueError("Converted DWG exceeds output limit")
        output = dest.read_bytes()
        if not output.startswith(b"AC10"):
            raise ValueError("Converter returned an invalid DWG")
        return output


class Handler(SimpleHTTPRequestHandler):
    # Windows 的 mimetypes 会查注册表解析未知扩展名（.wasm 常在此时出错断连），显式覆盖。
    extensions_map = {
        **SimpleHTTPRequestHandler.extensions_map,
        ".wasm": "application/wasm",
        ".dxf": "application/octet-stream",
        ".js": "text/javascript",
        ".html": "text/html",
    }

    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=STATIC_DIR, **kwargs)

    def end_headers(self):
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Headers", "*")
        self.send_header("Access-Control-Allow-Methods", "*")
        super().end_headers()

    def do_OPTIONS(self):
        self.send_response(204)
        self.end_headers()

    def _forward(self):
        query = urllib.parse.urlparse(self.path).query
        target = urllib.parse.parse_qs(query).get("u", [None])[0]
        if not target:
            self.send_error(400, "missing ?u=<endpoint>")
            return
        body = None
        length = int(self.headers.get("Content-Length") or 0)
        if length:
            body = self.rfile.read(length)
        req = urllib.request.Request(target, data=body, method=self.command)
        for key, value in self.headers.items():
            if key.lower() not in HOP_HEADERS:
                req.add_header(key, value)
        try:
            with urllib.request.urlopen(req, timeout=120) as resp:
                payload = resp.read()
                self.send_response(resp.status)
                for key, value in resp.headers.items():
                    if key.lower() not in HOP_HEADERS and key.lower() != "access-control-allow-origin":
                        self.send_header(key, value)
                self.send_header("Content-Length", str(len(payload)))
                self.end_headers()
                self.wfile.write(payload)
        except urllib.error.HTTPError as e:
            payload = e.read()
            self.send_response(e.code)
            self.send_header("Content-Type", e.headers.get("Content-Type", "application/json"))
            self.send_header("Content-Length", str(len(payload)))
            self.end_headers()
            self.wfile.write(payload)
        except Exception as e:  # noqa: BLE001
            self.send_error(502, f"proxy error: {e}")

    def do_GET(self):
        if self.path.startswith("/llmi?"):
            self._forward()
        else:
            super().do_GET()

    def do_POST(self):
        if urllib.parse.urlparse(self.path).path == "/convert/dxf-to-dwg":
            self._convert_dwg()
        elif self.path.startswith("/llmi?"):
            self._forward()
        else:
            self.send_error(405)

    def _convert_dwg(self):
        try:
            length = int(self.headers.get("Content-Length", "0"))
        except ValueError:
            self._conversion_error(400, "Invalid Content-Length")
            return
        if not 0 < length <= MAX_DXF_BYTES:
            self._conversion_error(413, "DXF size must be between 1 byte and 64 MB")
            return
        if not CONVERSIONS.acquire(blocking=False):
            self._conversion_error(503, "Conversion service busy; retry shortly")
            return
        try:
            self.connection.settimeout(120)
            payload = self.rfile.read(length)
            if len(payload) != length:
                raise ValueError("Incomplete DXF upload")
            version = urllib.parse.parse_qs(urllib.parse.urlparse(self.path).query).get("version", ["2018"])[0]
            output = convert_dxf(payload, version)
            self.send_response(200)
            self.send_header("Content-Type", "application/acad")
            self.send_header("Content-Disposition", 'attachment; filename="drawing.dwg"')
            self.send_header("Content-Length", str(len(output)))
            self.end_headers()
            self.wfile.write(output)
        except ValueError as error:
            self._conversion_error(422, str(error))
        except subprocess.TimeoutExpired:
            self._conversion_error(504, "DWG conversion timed out")
        except (RuntimeError, OSError) as error:
            self._conversion_error(503, str(error))
        finally:
            CONVERSIONS.release()

    def _conversion_error(self, status, message):
        payload = json.dumps({"error": message}, ensure_ascii=False).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)

    def log_message(self, fmt, *args):
        sys.stderr.write("%s %s\n" % (self.address_string(), fmt % args))


if __name__ == "__main__":
    print(f"serving {STATIC_DIR} on http://127.0.0.1:{PORT}  (AI proxy: /llmi?u=<endpoint>)")
    ThreadingHTTPServer(("0.0.0.0", PORT), Handler).serve_forever()
