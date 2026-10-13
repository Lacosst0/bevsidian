"""Compiles the Rust code blocks of the vault's notes against Bevy, so the docs cannot rot.

    python3 tools/check_snippets.py [--dir DIR] [note name ...]

Writes a scratch crate in DIR (default `target/snippets`) with one module per code block and
runs `cargo check` on it. Every note's blocks are checked, or only the notes named.

Blocks are checked on their own, with `use bevy::prelude::*;` in scope, unless the info string
says otherwise:

- ```rust cont         continues the note's previous block: it shares that block's module, so
                       it can use the types and functions defined there (and may not redefine
                       them). The `example=NAME` blocks of a note always share one module.
- ```rust ignore       is not compiled (fragments, pseudo-code, other languages' output).
- ```rust fresh        starts a new module even though it has `example=NAME`.

A note with `check: skip` in its front matter is not compiled (excerpts of a bigger program).
A note about a crate that has no release for the current Bevy yet says `bevy: 0.19` (the version
its latest release uses); those notes are checked together in a second scratch crate.

A note that shows an ecosystem crate lists it in its front matter, so it is added to the
scratch crate: `crates: bevy_egui=0.43; serde={ version = "1", features = ["derive"] }`
(separated by `;`).
"""

import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
VAULT = ROOT / "vault"
BEVY = "0.20"

FENCE = re.compile(r"^```(\S*)[ \t]*(.*)$")


def front_matter(text):
    props = {}
    if text.startswith("---\n"):
        end = text.find("\n---", 4)
        if end > 0:
            for line in text[4:end].splitlines():
                key, _, value = line.partition(":")
                props[key.strip()] = value.strip()
    return props


def blocks(text):
    """(info, code, line) of the fenced blocks of a note."""
    out, lines, i = [], text.splitlines(), 0
    while i < len(lines):
        m = FENCE.match(lines[i])
        if m and m.group(1):
            lang, info, start, code = m.group(1), m.group(2), i + 1, []
            i += 1
            while i < len(lines) and not lines[i].startswith("```"):
                code.append(lines[i])
                i += 1
            out.append((lang, info, "\n".join(code), start))
        i += 1
    return out


def slug(name):
    return re.sub(r"\W+", "_", name.lower()).strip("_")


def collect(wanted):
    """The notes' compilable modules, grouped by the Bevy version they are written for."""
    groups = {}  # bevy version -> {"crates": {...}, "modules": [(name, mods)]}
    for path in sorted(VAULT.rglob("*.md")):
        name = path.stem
        text = path.read_text()
        props = front_matter(text)
        if props.get("check") == "skip":
            continue
        if wanted and name.lower() not in wanted:
            continue
        bevy = props.get("bevy", BEVY)
        group = groups.setdefault(bevy, {"crates": {}, "modules": []})
        for spec in filter(None, (c.strip() for c in props.get("crates", "").split(";"))):
            crate, _, version = spec.partition("=")
            version = version.strip()
            group["crates"][crate.strip()] = version if version.startswith("{") else version.strip('"')
        mods = []  # (first line, code)
        example_mod = None  # the module of the note's `example=NAME` blocks
        for lang, info, code, line in blocks(text):
            words = info.split()
            if lang != "rust" or "ignore" in words:
                continue
            example = any(w.startswith("example=") for w in words)
            if example and example_mod is not None and "fresh" not in words:
                mods[example_mod][1].append(code)
            elif "cont" in words and mods:
                mods[-1][1].append(code)
            else:
                mods.append((line, [code]))
                if example:
                    example_mod = len(mods) - 1
        if mods:
            group["modules"].append((name, mods))
    return groups


def write_crate(target, bevy, crates, modules):
    src = target / "src"
    src.mkdir(parents=True, exist_ok=True)
    deps = "\n".join(
        f"{c} = {v}" if v.startswith("{") else f'{c} = "{v}"' for c, v in sorted(crates.items())
    )
    features = (
        '["free_camera", "pan_camera", "pan_orbit_camera", "dev", "bevy_feathers"]'
        if bevy == BEVY
        else '["dev"]'
    )
    (target / "Cargo.toml").write_text(
        f'[package]\nname = "snippets"\nversion = "0.0.0"\nedition = "2024"\n\n'
        f'[dependencies]\nbevy = {{ version = "{bevy}", features = {features} }}\n{deps}\n\n'
        f'[profile.dev]\nopt-level = 0\ndebug = false\n'
        f'[profile.dev.package."*"]\nopt-level = 1\n'
    )
    for old in src.glob("*.rs"):
        old.unlink()
    lib = [
        "#![allow(warnings, clippy::all)]",
        "// The preview's stand-in for `info!`, used by blocks with `example=NAME`.",
        "#[derive(bevy::prelude::Resource, Default)]",
        "pub struct Log(pub Vec<String>);",
        "impl Log { pub fn say(&mut self, line: impl Into<String>) { self.0.push(line.into()); } }",
    ]
    for name, mods in modules:
        file = f"n_{slug(name)}"
        lib.append(f"mod {file};")
        body = []
        for index, (line, codes) in enumerate(mods):
            body.append(f"// {name}.md line {line}")
            body.append(f"mod b{index} {{\n    use bevy::prelude::*;\n    use crate::Log;\n")
            body.append("\n".join(codes))
            body.append("}\n")
        (src / f"{file}.rs").write_text("\n".join(body))
    (src / "lib.rs").write_text("\n".join(lib) + "\n")


def main():
    args = sys.argv[1:]
    target = ROOT / "target" / "snippets"
    if "--dir" in args:
        at = args.index("--dir")
        target = pathlib.Path(args[at + 1])
        del args[at : at + 2]
    wanted = {a.lower() for a in args}

    status = 0
    for bevy, group in sorted(collect(wanted).items()):
        # Notes for the current Bevy go in `target`, others in `target-<version>`.
        where = target if bevy == BEVY else target.with_name(f"{target.name}-{bevy}")
        write_crate(where, bevy, group["crates"], group["modules"])
        count = sum(len(m) for _, m in group["modules"])
        print(f"checking {count} blocks of {len(group['modules'])} notes against Bevy {bevy} in {where}", flush=True)
        result = subprocess.run(
            ["cargo", "check", "--manifest-path", str(where / "Cargo.toml"), "--message-format=short"],
            cwd=where,
        )
        status = status or result.returncode
    sys.exit(status)


if __name__ == "__main__":
    main()
