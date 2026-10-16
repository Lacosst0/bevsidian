//! The on-screen keyboard. On the web, focusing a text field by touch focuses a hidden textarea
//! so the phone shows its own keyboard, and what it types goes to the field (see [`native`]).
//! Elsewhere a Feathers [`VirtualKeyboard`] docks at the bottom; its keys type into the field
//! that had focus.

use bevy::{
    feathers::controls::{VirtualKeyPressed, VirtualKeyboard},
    input_focus::InputFocus,
    prelude::*,
    text::{EditableText, TextEdit},
};
use bevy_radioactive::view::{Watch, component, view};

use bevsidian_core::Nav;
use bevsidian_ui::prelude::*;

pub fn plugin(app: &mut App) {
    app.init_resource::<KeyboardTarget>()
        .add_systems(Update, track_keyboard_target);
    #[cfg(target_arch = "wasm32")]
    app.add_systems(Update, native::sync.after(track_keyboard_target));
}

type Keys = VirtualKeyboard<&'static str>;

/// The text field the on-screen keyboard types into.
#[derive(Resource, Default)]
struct KeyboardTarget(Option<Entity>);

/// Root of the on-screen keyboard.
#[derive(Component, Clone, Copy, Debug, Default)]
struct KeyboardPanel;

fn track_keyboard_target(
    focus: Res<InputFocus>,
    editable: Query<(), With<EditableText>>,
    panel: Query<(), With<KeyboardPanel>>,
    parents: Query<&ChildOf>,
    mut target: ResMut<KeyboardTarget>,
    mut nav: ResMut<Nav>,
) {
    if !focus.is_changed() {
        return;
    }
    match focus.get() {
        Some(entity) if editable.contains(entity) => target.0 = Some(entity),
        // Pressing a key focuses the key's button: keep typing into the field.
        Some(entity) if parents.iter_ancestors(entity).any(|e| panel.contains(e)) => {}
        _ => target.0 = None,
    }
    if nav.keyboard != target.0.is_some() {
        nav.keyboard = target.0.is_some();
    }
}

const ROWS: [&[&str]; 5] = [
    &["#", "*", "-", "`", "[", "]", "(", ")", ".", ","],
    &["q", "w", "e", "r", "t", "y", "u", "i", "o", "p"],
    &["a", "s", "d", "f", "g", "h", "j", "k", "l"],
    &["z", "x", "c", "v", "b", "n", "m", "del"],
    &["space", "enter", "Done"],
];

fn on_virtual_key(
    key: On<VirtualKeyPressed<&'static str>>,
    mut target: ResMut<KeyboardTarget>,
    mut fields: Query<&mut EditableText>,
    mut focus: ResMut<InputFocus>,
    mut nav: ResMut<Nav>,
) {
    let Some(entity) = target.0 else {
        return;
    };
    match key.key {
        "Done" => {
            target.0 = None;
            focus.clear();
            nav.keyboard = false;
            return;
        }
        "del" => {
            if let Ok(mut field) = fields.get_mut(entity) {
                field.queue_edit(TextEdit::Backspace);
            }
        }
        k => {
            if let Ok(mut field) = fields.get_mut(entity) {
                let text = match k {
                    "space" => " ",
                    // New line in the note editor; single-line fields ignore it.
                    "enter" if field.allow_newlines => "\n",
                    "enter" => "",
                    k => k,
                };
                if !text.is_empty() {
                    field.queue_edit(TextEdit::Insert(text.into()));
                }
            }
        }
    }
    focus.set(entity, bevy::input_focus::FocusCause::Navigated);
}

/// The on-screen keyboard, docked at the bottom while a text field is focused by touch.
#[component]
pub fn keyboard(nav: Watch<Res<Nav>>) -> impl Scene {
    view! {
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::End,
            flex_direction: FlexDirection::Column,
        }
        GlobalZIndex(layer::KEYBOARD)
        Pickable::IGNORE
        Children [
            if $nav.keyboard && $nav.touch && !cfg!(target_arch = "wasm32") {
                Node {
                    width: Val::Percent(100.0),
                    padding: UiRect::new(Val::Px(6.0), Val::Px(6.0), Val::Px(8.0), Val::Px(14.0)),
                    border: UiRect::top(Val::Px(1.0)),
                }
                BackgroundColor(color::CARD)
                BorderColor::all(color::BORDER)
                KeyboardPanel
                Children [
                    @Keys {
                        @keys: {ROWS.iter().map(|row| row.to_vec()).collect::<Vec<_>>()}
                    }
                    Node { width: Val::Percent(100.0) }
                    on(on_virtual_key)
                ]
            }
        ]
    }
}

/// The phone's own keyboard on the web. A canvas cannot summon it, so a transparent textarea takes
/// the browser focus while a field is edited by touch. It holds one sentinel space: after each
/// input the new text past it is inserted into the field, and a missing sentinel is a backspace.
#[cfg(target_arch = "wasm32")]
mod native {
    use std::{cell::RefCell, rc::Rc};

    use bevy::{
        input_focus::InputFocus,
        prelude::*,
        text::{EditableText, TextEdit},
    };
    use wasm_bindgen::{JsCast, closure::Closure};
    use web_sys::HtmlTextAreaElement;

    use super::KeyboardTarget;
    use bevsidian_core::Nav;

    const SENTINEL: &str = " ";

    struct Proxy {
        area: HtmlTextAreaElement,
        /// A field is being edited by touch: the next touch keeps the keyboard up.
        wanted: Rc<RefCell<bool>>,
        typed: Rc<RefCell<Vec<TextEdit>>>,
    }

    thread_local! {
        static PROXY: Option<Proxy> = create();
    }

    fn create() -> Option<Proxy> {
        let document = web_sys::window()?.document()?;
        let area: HtmlTextAreaElement =
            document.create_element("textarea").ok()?.dyn_into().ok()?;
        for (name, value) in [
            ("autocomplete", "off"),
            ("autocorrect", "off"),
            ("autocapitalize", "off"),
            ("spellcheck", "false"),
            ("aria-hidden", "true"),
        ] {
            let _ = area.set_attribute(name, value);
        }
        // Fixed at the top so the browser doesn't scroll to it; 16px stops iOS zooming in.
        let _ = area.set_attribute(
            "style",
            "position:fixed;top:0;left:0;width:1px;height:1px;opacity:0;border:0;padding:0;\
             font-size:16px;resize:none;pointer-events:none;",
        );
        area.set_value(SENTINEL);
        document.body()?.append_child(&area).ok()?;

        let wanted = Rc::new(RefCell::new(false));
        let typed = Rc::new(RefCell::new(Vec::new()));

        let read = {
            let area = area.clone();
            let typed = typed.clone();
            move || {
                let value = area.value();
                let mut typed = typed.borrow_mut();
                match value.strip_prefix(SENTINEL) {
                    Some(text) if !text.is_empty() => typed.push(TextEdit::Insert(text.into())),
                    Some(_) => return,
                    None => {
                        typed.push(TextEdit::Backspace);
                        // Text typed over a selected sentinel.
                        if !value.is_empty() {
                            typed.push(TextEdit::Insert(value.into()));
                        }
                    }
                }
                area.set_value(SENTINEL);
            }
        };
        // Words being composed (autocorrect, swipe typing) arrive when the composition ends.
        let on_input = {
            let read = read.clone();
            Closure::<dyn FnMut(web_sys::InputEvent)>::new(move |event: web_sys::InputEvent| {
                if !event.is_composing() {
                    read();
                }
            })
        };
        let on_composed = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| read());
        // iOS only shows the keyboard for a focus inside a touch handler.
        let on_touch = {
            let area = area.clone();
            let wanted = wanted.clone();
            Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
                if *wanted.borrow() {
                    let _ = area.focus();
                }
            })
        };
        let _ = area.add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref());
        let _ = area.add_event_listener_with_callback(
            "compositionend",
            on_composed.as_ref().unchecked_ref(),
        );
        let _ = document
            .add_event_listener_with_callback("touchend", on_touch.as_ref().unchecked_ref());
        on_input.forget();
        on_composed.forget();
        on_touch.forget();

        Some(Proxy {
            area,
            wanted,
            typed,
        })
    }

    /// Shows or hides the phone's keyboard with the edited field and types into it.
    pub fn sync(
        target: Res<KeyboardTarget>,
        nav: Res<Nav>,
        mut fields: Query<&mut EditableText>,
        mut focus: ResMut<InputFocus>,
        mut was: Local<bool>,
    ) {
        PROXY.with(|proxy| {
            let Some(proxy) = proxy else {
                return;
            };
            let want = target.0.is_some() && nav.touch;
            if want != *was {
                *was = want;
                *proxy.wanted.borrow_mut() = want;
                if want {
                    // Enough on Android; iOS waits for the touch to end.
                    let _ = proxy.area.focus();
                } else {
                    let _ = proxy.area.blur();
                }
            }
            let typed: Vec<_> = proxy.typed.borrow_mut().drain(..).collect();
            let Some(entity) = target.0 else {
                return;
            };
            let Ok(mut field) = fields.get_mut(entity) else {
                return;
            };
            for edit in typed {
                match edit {
                    TextEdit::Insert(text) if !field.allow_newlines && text.contains('\n') => {
                        // Return on a single-line field closes the keyboard.
                        let text: String = text.replace('\n', "");
                        if !text.is_empty() {
                            field.queue_edit(TextEdit::Insert(text.into()));
                        }
                        focus.clear();
                    }
                    edit => field.queue_edit(edit),
                }
            }
        });
    }
}
