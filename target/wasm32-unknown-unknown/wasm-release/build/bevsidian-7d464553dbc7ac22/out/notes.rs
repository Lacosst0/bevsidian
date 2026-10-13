/// `(path, markdown)` of every embedded note.
pub static NOTES: &[(&str, &str)] = &[
    ("ECS.md", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/vault/", "ECS.md"))),
    ("Mario.md", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/vault/", "Mario.md"))),
    ("Rendering.md", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/vault/", "Rendering.md"))),
    ("Scenes and BSN.md", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/vault/", "Scenes and BSN.md"))),
    ("UI kit.md", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/vault/", "UI kit.md"))),
    ("Welcome.md", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/vault/", "Welcome.md"))),
];
