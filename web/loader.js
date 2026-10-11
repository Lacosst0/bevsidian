// Development loader: serves the wasm-bindgen output next to this page.
// The release bundle (tools/bundle.py) replaces it with an inline, self-contained loader.
import init from "./bevsidian.js";
import { boot } from "./boot.js";

boot(() => init({ module_or_path: new URL("./bevsidian_bg.wasm", import.meta.url) }));
