#!/usr/bin/env python3
"""Bundle the web build into one self-contained index.html.

Inlines web/style.css, web/frameclock.js, the wasm-bindgen JS glue, web/boot.js and the
gzip-compressed, base64-encoded WebAssembly module, so the page works from any static host
or file://.

Usage: tools/bundle.py <wasm-bindgen out dir> <web dir> <output html>
"""

import base64
import gzip
import re
import sys
from pathlib import Path


def main() -> None:
    if len(sys.argv) != 4:
        sys.exit(__doc__)
    pkg, web, out = (Path(p) for p in sys.argv[1:])

    html = (web / "index.html").read_text(encoding="utf-8")
    css = (web / "style.css").read_text(encoding="utf-8")
    glue = (pkg / "bevsidian.js").read_text(encoding="utf-8")
    boot = (web / "boot.js").read_text(encoding="utf-8")
    frameclock = (web / "frameclock.js").read_text(encoding="utf-8")
    wasm = (pkg / "bevsidian_bg.wasm").read_bytes()

    packed = base64.b64encode(gzip.compress(wasm, compresslevel=9, mtime=0)).decode("ascii")

    # Turn the ES modules into one inline module script.
    boot = boot.replace("export async function boot", "async function boot")
    glue = re.sub(r"^export \{[^}]*\};?\s*$", "", glue, flags=re.M)
    glue = re.sub(r"^export default \w+;?\s*$", "", glue, flags=re.M)
    glue = glue.replace("export function ", "function ").replace("export class ", "class ")
    glue = glue.replace("export const ", "const ")

    loader = f"""
const WASM_GZIP_BASE64 = "{packed}";

// Decoded in place (no fetch), so it also works where data: URLs cannot be fetched.
async function wasmBytes() {{
  const text = atob(WASM_GZIP_BASE64);
  const bytes = new Uint8Array(text.length);
  for (let i = 0; i < text.length; i++) bytes[i] = text.charCodeAt(i);
  const stream = new Blob([bytes]).stream().pipeThrough(new DecompressionStream("gzip"));
  return new Response(stream).arrayBuffer();
}}

boot(async () => __wbg_init({{ module_or_path: await wasmBytes() }}));
"""
    script = "\n".join([glue, boot, loader]).replace("</script", "<\\/script")

    html = html.replace(
        '<link rel="stylesheet" href="style.css">', f"<style>\n{css}\n</style>"
    )
    html = html.replace('<script type="module" src="loader.js"></script>\n', "")
    # A classic script: it must patch the canvas and requestAnimationFrame before the wasm
    # module (deferred) starts.
    html = html.replace(
        '<script src="frameclock.js"></script>',
        "<script>\n" + frameclock.replace("</script", "<\\/script") + "\n</script>",
    )
    html = html.replace(
        "<!-- BEVSIDIAN_SCRIPT -->", f'<script type="module">\n{script}\n</script>'
    )
    if "__wbg_init" not in glue:
        sys.exit("unexpected wasm-bindgen output: __wbg_init not found")

    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(html, encoding="utf-8")
    print(
        f"wrote {out} ({out.stat().st_size / 1e6:.1f} MB; wasm {len(wasm) / 1e6:.1f} MB raw)"
    )


if __name__ == "__main__":
    main()
