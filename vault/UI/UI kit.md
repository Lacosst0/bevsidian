---
check: skip
icon: component
---
# UI kit

Bevsidian's interface is built from a small component library, `bevsidian_ui`, modelled on [shadcn/ui](https://ui.shadcn.com): its dark *neutral* palette, sizes and states, with one orange accent kept for links. Every component is a plain function returning a BSN scene (`bsn!`, or `view!` when it reads signals), so you use it with `@{...}` and patch it like any other scene. The previews below are the real components. #ui #bsn

## Buttons

Four variants (`Primary`, `Secondary`, `Outline`, `Ghost`) and eight sizes, four of them square for icons. Hover, selected and disabled colours come from one `Surface` component, so a button only says *which* state it is in, for example `Disabled(true)`.

```rust preview=kit-buttons
pub fn buttons() -> impl Scene {
    bsn! {
        Node { flex_wrap: FlexWrap::Wrap, column_gap: px(8), row_gap: px(8), align_items: AlignItems::Center }
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
            --
            @{icon_text_button(Variant::Secondary, Size::Sm, "copy", "Copy")}
            --
            @{text_button(Variant::Outline, Size::Sm, "Disabled")}
            Disabled(true)
        ]
    }
}
```

## Tabs

The Note / Graph switch in the top bar. The selected tab is bound to a signal with `Selected($...)`; click to try it:

```rust preview=kit-tabs
pub fn view_tabs() -> impl Scene {
    let current = signal(|| 0);
    let (note, graph) = (current.clone(), current.clone());
    view! {
        @{tab_list()}
        Children [
            @{tab("Note")}
            Selected($current == 0)
            on(move |_: On<Activate>| note.set(0))
            --
            @{tab("Graph")}
            Selected($current == 1)
            on(move |_: On<Activate>| graph.set(1))
        ]
    }
}
```

## Badges and keys

Tags are outline badges; a selected one (the active filter) fills in.

```rust preview=kit-badges
pub fn badges() -> impl Scene {
    let active = signal(|| true);
    let toggle = active.clone();
    view! {
        Node { flex_direction: FlexDirection::Column, row_gap: px(12) }
        Children [
            Node { column_gap: px(6), align_items: AlignItems::Center }
            Children [
                @{badge(BadgeVariant::Outline, "#ecs")}
                --
                @{badge(BadgeVariant::Secondary, "3")}
                --
                @{badge_button("#beginner")}
                Selected($active)
                on(move |_: On<Activate>| toggle.update(|on| *on = !*on))
            ]
            --
            @{shortcuts(&[("Ctrl O", "switch"), ("Esc", "close")])}
        ]
    }
}
```

## Forms

`field` lays out a label, an optional description and a control. `stepper` disables its buttons at the ends of its range, and `segmented` selects nothing when the value matches no option:

```rust preview=kit-form
pub fn settings_card() -> impl Scene {
    let (size, speed, fps) = (signal(|| 16), signal(|| 1), signal(|| true));
    let speeds = vec![(1, "1×".to_owned()), (2, "2×".to_owned()), (3, "3×".to_owned())];
    bsn! {
        @{card()}
        Node { width: px(380), max_width: Val::Percent(100.0) }
        Children [
            @{field("Text size", None, stepper(size, 13..=24, 1, |v| format!("{v} px")))}
            --
            @{field("Scroll speed", Some("Wheel and trackpad"), segmented(speeds, speed))}
            --
            @{field("Show FPS", None, switch(fps))}
        ]
    }
}
```

## ECS identity

Entities, components and systems always look the same: the shape says the kind (a filled tile for things that exist, an outlined tile for things that run, a bare icon for data), the icon says which one. `EntityCard` is a `SceneComponent` with props:

```rust preview=kit-entity
pub fn entity_card() -> impl Scene {
    let components = ["Health(50)", "Player", "Transform (0, 0, 0)"]
        .map(ComponentValue::parse)
        .to_vec();
    bsn! { @EntityCard { @title: "Entity 7", @components: components } }
}
```

| Kind | Icon | Mount |
|:--|:--|:--|
| Entity, scene | `box`, `layers` | filled tile |
| System, observer | `square-function`, `eye` | outlined tile |
| Component | `puzzle`, or its own (`heart`, `move`, `video`) | bare icon, value on the right |
| Marker component | `flag` | bare icon, no value |
| Resource, event, query | `database`, `zap`, `filter` | bare icon |

## Also in the kit

| Component | shadcn/ui counterpart | Used for |
|:--|:--|:--|
| `item`, `sub_item` | Sidebar menu button | notes, headings, links and tags in the sidebar |
| `disclosure`, `tree_children` | Collapsible, Sidebar menu sub | the note tree |
| `text_input` | Input | sidebar search, quick switcher |
| `dialog`, `scrim` | Dialog | settings, quick switcher |
| `popover`, `menu_item` | Context menu | right-click menu, selection toolbar |
| `toast` | Sonner | "Copied to clipboard" |
| `alert` | Alert | callouts, help text |

Colours, radii, spacing, shadows and layers are design tokens in `bevsidian_ui::tokens`, named like shadcn's: `background`, `card`, `accent`, `border`, `input`, `foreground`, `muted-foreground`, `primary`. When a new component is needed, start from the matching shadcn/ui component.

See also [[Scenes and BSN]] for how `@{...}` inheritance and patches work.
