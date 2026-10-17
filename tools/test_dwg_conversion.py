"""Local HTTP and real Rust converter regression tests (no cloud service)."""
import importlib.util
import json
from pathlib import Path
import tempfile
import threading
import unittest
import urllib.request
import urllib.error
import subprocess

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("dev_proxy", ROOT / "tools/dev-proxy.py")
proxy = importlib.util.module_from_spec(spec)
spec.loader.exec_module(proxy)


class ConversionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.server = proxy.ThreadingHTTPServer(("127.0.0.1", 0), proxy.Handler)
        cls.thread = threading.Thread(target=cls.server.serve_forever, daemon=True)
        cls.thread.start()
        cls.url = f"http://127.0.0.1:{cls.server.server_port}/convert/dxf-to-dwg"

    @classmethod
    def tearDownClass(cls):
        cls.server.shutdown()
        cls.server.server_close()
        cls.thread.join()

    def post(self, payload, suffix=""):
        return urllib.request.urlopen(urllib.request.Request(self.url + suffix, data=payload,
            headers={"Content-Type": "application/octet-stream"}), timeout=130)

    def test_real_dxf_to_dwg_and_read_back(self):
        payload = (ROOT / "apps/cadcraft-web/dev-host/mep-coordination.dxf").read_bytes()
        with self.post(payload, "?version=2018") as response:
            self.assertEqual(response.status, 200)
            output = response.read()
            self.assertEqual(output[:6], b"AC1032")
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / "roundtrip.dwg"
            path.write_bytes(output)
            result = subprocess.run([str(proxy.CLI), "info", str(path)], capture_output=True, timeout=60)
            self.assertEqual(result.returncode, 0, result.stderr.decode(errors="replace"))
            before = subprocess.run([str(proxy.CLI), "info", str(ROOT / "apps/cadcraft-web/dev-host/mep-coordination.dxf")],
                                    capture_output=True, timeout=60, check=True)
            original, converted = json.loads(before.stdout), json.loads(result.stdout)
            # CAD table names are case-insensitive; the DWG writer canonicalizes them.
            for summary in (original, converted):
                for layer in summary["layers"]:
                    layer["linetype"] = layer["linetype"].casefold()
            for field in ("entityCount", "counts", "extents", "layers", "layouts"):
                self.assertEqual(converted[field], original[field], field)

    def test_invalid_input_and_version(self):
        for data, suffix in [(b"not a DXF", ""), (b"anything", "?version=bad")]:
            with self.assertRaises(urllib.error.HTTPError) as error:
                self.post(data, suffix)
            self.assertEqual(error.exception.code, 422)

    def test_empty_upload_is_rejected(self):
        with self.assertRaises(urllib.error.HTTPError) as error:
            self.post(b"")
        self.assertEqual(error.exception.code, 413)


if __name__ == "__main__":
    unittest.main()
