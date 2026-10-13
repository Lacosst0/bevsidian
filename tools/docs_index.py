"""Builds the Bevy docs index used by docs references (`[`Commands`]` in notes).

Reads the pages of a rustdoc build of the `bevy` crate and writes
`crates/bevsidian_core/data/bevy-docs.txt`: one line per item, `kind path` (e.g.
`struct ecs::system::Commands`), the page being `https://docs.rs/bevy/<version>/bevy/<path
with / for ::, the last segment as kind.Name>.html`. Items the prelude re-exports are listed
once, at their own module.

To rebuild it for a new Bevy version: in an empty crate depending on `bevy` (default
features, so every module is documented), run `cargo doc -p bevy --no-deps`, then
`python3 tools/docs_index.py <that crate>/target/doc/bevy`.
"""

import pathlib
import re
import sys

KINDS = "struct trait enum fn macro derive type constant attr union static primitive".split()
PAGE = re.compile(r"^(%s)\.(.+)\.html$" % "|".join(KINDS))

root = pathlib.Path(sys.argv[1])
items = set()
for page in root.rglob("*.html"):
    match = PAGE.match(page.name)
    if not match:
        continue
    modules = page.parent.relative_to(root).parts
    if "prelude" in modules:
        continue
    kind, name = match.groups()
    items.add((kind, "::".join(modules + (name,))))

out = pathlib.Path(__file__).resolve().parent.parent / "crates/bevsidian_core/data/bevy-docs.txt"
out.write_text("".join(f"{kind} {path}\n" for kind, path in sorted(items, key=lambda i: i[1])))
print(f"wrote {len(items)} items to {out}")
