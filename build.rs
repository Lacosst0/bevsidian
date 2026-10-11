//! Embeds every markdown note under `vault/` into the binary, so the web build is a
//! single self-contained file.

use std::{env, fmt::Write as _, fs, path::Path};

fn collect(dir: &Path, root: &Path, out: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.flatten().map(|e| e.path()).collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect(&path, root, out);
        } else if path.extension().is_some_and(|e| e == "md") {
            let relative = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            out.push(relative);
        }
    }
}

fn main() {
    let manifest = env::var("CARGO_MANIFEST_DIR").unwrap();
    let vault = Path::new(&manifest).join("vault");
    println!("cargo:rerun-if-changed=vault");
    let mut notes = Vec::new();
    collect(&vault, &vault, &mut notes);
    let mut code = String::from(
        "/// `(path, markdown)` of every embedded note.\npub static NOTES: &[(&str, &str)] = &[\n",
    );
    for note in &notes {
        println!("cargo:rerun-if-changed=vault/{note}");
        let _ = writeln!(
            code,
            "    ({note:?}, include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/vault/\", {note:?}))),"
        );
    }
    code.push_str("];\n");
    let out = Path::new(&env::var("OUT_DIR").unwrap()).join("notes.rs");
    fs::write(out, code).unwrap();
}
