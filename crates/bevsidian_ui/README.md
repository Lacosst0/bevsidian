# bevsidian_ui

Bevsidian's UI kit: design tokens and components modelled on
[shadcn/ui](https://ui.shadcn.com), written as BSN scenes for Bevy 0.20.

## Need a new component? Start at shadcn/ui

Before designing anything, find the closest component in the shadcn/ui docs
(<https://ui.shadcn.com/docs/components>). Its source lives in
[`apps/v4/registry/new-york-v4/ui/`](https://github.com/shadcn-ui/ui/tree/main/apps/v4/registry/new-york-v4/ui)
of `shadcn-ui/ui`. Copy its anatomy, sizes, spacing and states, translate its Tailwind
classes with the table below, and add it here as **one file** in `src/components/`.

| Tailwind (shadcn) | Here |
| --- | --- |
| `bg-background` / `bg-card` / `bg-accent` | `color::BACKGROUND` / `CARD` / `ACCENT` |
| `hover:bg-accent` on rows, ghost buttons | `Surface::GHOST` (`color::HOVER`, `SELECTED`) |
| `border` / `border-input` / `bg-input/30` | `color::BORDER` / `INPUT` / `INPUT_BG` |
| `text-foreground` / `text-muted-foreground` | `color::FOREGROUND` / `MUTED_FOREGROUND` |
| `bg-primary text-primary-foreground` | `color::PRIMARY` / `PRIMARY_FOREGROUND` |
| `rounded-sm` … `rounded-xl` | `radius::SM` … `radius::XXL` |
| `h-8` / `h-9` / `h-10` / `size-9` | `Size::Sm` / `Md` / `Lg` / `Icon` |
| `text-xs` / `text-sm` / `text-base` | `size::XS` / `SM` / `BASE` (13 / 14 px here) |
| `shadow-md` / `shadow-lg` | `shadow::popover()` / `shadow::dialog()` |
| lucide-react `<Icon />` | `icon("name", 16.0, color)` (add the name to `tools/icons.py`) |

The palette is shadcn's dark *neutral* theme. The only hue is `color::LINK` (orange), kept
for links, the current graph node and the running code line; warnings are amber.

## Conventions

- **One component per file**, one function per component returning `bsn!` (or `view!`
  when it reads a signal). Helpers stay private to their file.
- **Compose with `@{...}`, override by patching**: a caller adds `on(...)`, components or
  `Node { .. }` fields after the include instead of the component growing options.
- **State through [`Surface`](src/surface.rs)**: interactive parts set `Selected(..)` and
  `Disabled(..)`; one system paints backgrounds, borders (`SurfaceBorder`) and the `Tint`
  of the labels inside. Never write hover colours by hand.
- **`SceneComponent` + props** when a marker is useful for queries and props read better
  named: `@EntityCard { @title: .., @components: .. }`.
- **Edge cases are part of the component**: long labels are cut at the end (`tinted`),
  steppers disable at their limits, a segmented control with an unknown value selects
  nothing, empty optional parts (descriptions, values, shortcuts) spawn nothing.

## Components

| File | shadcn/ui | Functions |
| --- | --- | --- |
| `button.rs` | Button | `button`, `text_button`, `icon_button`, `icon_text_button` |
| `tabs.rs` | Tabs | `tab_list`, `tab` |
| `segmented.rs` | Toggle Group | `segmented` |
| `badge.rs` | Badge | `badge`, `badge_button` |
| `kbd.rs` | Kbd | `kbd`, `kbd_group`, `shortcuts` |
| `item.rs` | Sidebar menu button, Command item | `item`, `item_icon`, `item_label`, `sub_item`, `hash` |
| `tree.rs` | Sidebar menu sub, Collapsible | `disclosure`, `tree_children` |
| `input.rs` | Input | `text_input` |
| `field.rs` | Field | `field` |
| `switch.rs` | Switch | `switch` |
| `checkbox.rs` | Checkbox | `checkbox` |
| `stepper.rs` | (number input) | `stepper` |
| `dialog.rs` | Dialog | `scrim`, `dialog`, `dialog_header`, `dialog_footer` |
| `pagination.rs` | Pagination | `page_link` |
| `tooltip.rs` | Tooltip | `Tooltip` (a component: add it to any element) |
| `popover.rs` | Popover, Dropdown / Context menu | `popover`, `menu_item`, `menu_separator` |
| `toast.rs` | Sonner | `toast` |
| `alert.rs` | Alert | `alert` |
| `card.rs` | Card | `card`, `inset` |
| `separator.rs` | Separator | `separator`, `vertical_separator` |
| `typography.rs` | Typography | `label`, `tinted`, `title`, `muted`, `caption`, `group_label`, `empty`, `count` |
| `icon.rs` | (lucide) | `icon`, `icon_char`, `tinted_icon`, `icon_tile`, `note_icon` |
| `ecs.rs` | — | `EntityCard`, `component_icon`, `EcsKind` |

## ECS identity

Shape says the kind, the icon says the thing:

| Kind | Lucide | Mount |
| --- | --- | --- |
| Entity | `box` | filled tile |
| Scene | `layers` | filled tile |
| System | `square-function` | outlined tile |
| Observer | `eye` | outlined tile |
| Component | `puzzle` (or its own: `heart`, `move`, `video`, `shapes`, `palette`, …) | bare icon |
| Marker component | `flag` | bare icon, no value |
| Resource | `database` | bare icon |
| Event | `zap` | bare icon |
| Query | `filter` | bare icon |

The mockups these come from are in [`design/`](../../design).
