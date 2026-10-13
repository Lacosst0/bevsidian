---
icon: component
---
# UI kit

Bevsidian's interface is built from a small component library, `bevsidian_ui`, modelled on [shadcn/ui](https://ui.shadcn.com). Every component is a plain function returning a BSN scene (`bsn!`, or `view!` when it reacts to signals), so you use it with `@{...}` and patch it like any other scene. The previews below are the real components. #ui #bsn

## Buttons

Four variants (`Primary`, `Secondary`, `Outline`, `Ghost`) and six sizes, three of them square for icons. Hover and selection colours come from one `Surface` component, so a button only declares *which* state it is in.

```rust preview=kit-buttons
fn buttons() -> impl Scene {
    bsn! {
        Node { flex_wrap: FlexWrap::Wrap, column_gap: px(8), row_gap: px(8) }
        Children [
            @{text_button(Variant::Primary, Size::Md, "Primary")}
            --
            @{text_button(Variant::Secondary, Size::Md, "Secondary")}
            --
            @{text_button(Variant::Outline, Size::Md, "Outline")}
            --
            @{text_button(Variant::Ghost, Size::Md, "Ghost")}
            --
            @{icon_button(Variant::Outline, Size::Icon, "search")}
        ]
    }
}
```

## Toggle group

The Note / Graph switch in the top bar. The selected item is bound to a signal with `Selected($...)`; click to try it:

```rust preview=kit-toggle-group
fn view_switch() -> impl Scene {
    let tab = signal(|| 0);
    let (note, graph) = (tab.clone(), tab.clone());
    view! {
        @{toggle_group()}
        Children [
            @{toggle("Note")}
            Selected($tab == 0)
            on(move |_: On<Activate>| note.set(0))
            --
            @{toggle("Graph")}
            Selected($tab == 1)
            on(move |_: On<Activate>| graph.set(1))
        ]
    }
}
```

## Badges and keys

```rust preview=kit-badges
fn badges() -> impl Scene {
    bsn! {
        Node { flex_direction: FlexDirection::Column, row_gap: px(10) }
        Children [
            Node { column_gap: px(6) }
            Children [
                @{badge()} Children [ @{label("#ecs", 12.0, FontWeight::MEDIUM, color::PRIMARY)} ]
                --
                @{badge_button()} Selected(true)
                Children [ @{label("#beginner", 12.0, FontWeight::MEDIUM, color::PRIMARY)} ]
            ]
            --
            @{shortcuts(&[("Ctrl O", "switch"), ("Esc", "close")])}
        ]
    }
}
```

## Forms

`field` lays out a label and a control; `stepper` and `switch` are reactive:

```rust preview=kit-form
fn settings_card() -> impl Scene {
    let (fps, size) = (signal(|| true), signal(|| 16));
    let (smaller, larger, text) = (size.clone(), size.clone(), size.clone());
    let text_size = stepper(
        move || format!("{} px", text.get()),
        move || smaller.update(|s| *s -= 1),
        move || larger.update(|s| *s += 1),
    );
    bsn! {
        @{card()}
        Node { width: px(300) }
        Children [
            @{title("Settings")}
            --
            @{field("Text size", text_size)}
            --
            @{field("Show FPS", switch(fps))}
            --
            @{separator()}
            --
            @{text_button(Variant::Primary, Size::Sm, "Done")}
            Node { align_self: AlignSelf::End }
        ]
    }
}
```

## Also in the kit

| Component | shadcn/ui counterpart | Used for |
|:--|:--|:--|
| `item` | Item, Command item | file list, outline and link rows, switcher results, menus |
| `section_label` | Sidebar group label | "FILES", "BACKLINKS" |
| `dialog`, `scrim` | Dialog | settings, quick switcher |
| `popover` | Popover, Dropdown menu | context menu, selection toolbar |
| `card`, `inset`, `tile` | Card | live previews and their entity cards |
| `progress` | Progress | health bars in previews |
| `toast` | Sonner | "Copied!" |
| `empty`, `note_box` | Empty, Alert | empty lists, help text |

Colours, radii, shadows and layers are design tokens in `bevsidian_ui::theme`, named like shadcn's: `background`, `card`, `secondary`, `foreground`, `muted-foreground`, `primary`, `border`.

See also [[Scenes and BSN]] for how `@{...}` inheritance and patches work.
