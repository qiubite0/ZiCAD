# Exporting DXF drawings as DWG

Open a DXF drawing, then choose **File → Export DWG…**. The current drawing,
including edits, is exported. **File → Default Save Version** selects the target
DWG version (2000, 2004, 2007, 2010, 2013 or 2018). Exporting does not change the
current drawing's file name or clear unsaved changes.

The web build sends DXF bytes to the same-origin local service at
`POST /convert/dxf-to-dwg?version=2018`. The service uses CADCraft's native Rust
converter; drawings are not sent to a cloud service. Temporary conversion files
are removed after each request. The service accepts up to 64 MiB, runs at most
two conversions at a time, and times out after 120 seconds.

Build the converter once with `cargo build -p cadcraft-cli -j 2`, then run the
repository's `python tools/dev-proxy.py`. `CADCRAFT_CLI` may override
the converter executable path. Rebuild and restart the service after backend
changes. Web DWG import is still unsupported.

Conversion preserves the entities supported by CADCraft's DXF/DWG bridge; it
does not promise complete compatibility with third-party custom entities.

Validation:

- `cargo test -p cadcraft-ui-egui --test dwg_export --test ai_chat_window -j 2`
- From the repository root: `python tools/test_dwg_conversion.py`
- Rebuild the web app and verify File → Export DWG… downloads a file with the
  selected DWG version header.
