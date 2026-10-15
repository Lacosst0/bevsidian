//! [`Binding`]: the value a control shows and sets, either a component's own state or a field
//! of a resource.

use std::sync::Arc;

use bevy::{ecs::component::Mutable, prelude::*};
use bevy_radioactive::view::Watch;

/// What a control (a switch, a slider, a text field…) shows and sets. Reading it with `$` in
/// a `view!` tracks the watch behind it; setting it from a handler goes through `Commands`,
/// and only changes the value (and wakes watchers) when it differs.
///
/// ```ignore
/// // State of the component: `#[component] fn panel(open: Watch<Local<bool>>)`.
/// @{switch(Binding::local(open))}
/// // A field of a resource: `#[component] fn settings(nav: Watch<Res<Nav>>)`.
/// @{switch(Binding::resource(nav, |n: &Nav| n.sound, |n, on| n.sound = on))}
/// ```
pub struct Binding<T> {
    get: Arc<dyn Fn(bool) -> T + Send + Sync>,
    set: Arc<dyn Fn(&mut Commands, T) + Send + Sync>,
}

impl<T> Clone for Binding<T> {
    fn clone(&self) -> Self {
        Self {
            get: self.get.clone(),
            set: self.set.clone(),
        }
    }
}

impl<T: Clone + PartialEq + Default + Send + Sync + 'static> Binding<T> {
    /// Local state of a component.
    pub fn local(watch: Watch<Local<'static, T>>) -> Self {
        let read = watch.clone();
        Self {
            get: Arc::new(move |tracked| {
                if tracked {
                    read.get()
                } else {
                    read.get_untracked()
                }
            }),
            set: Arc::new(move |_, value| {
                watch.set_if_neq(value);
            }),
        }
    }
}

impl<T: Clone + PartialEq + Send + Sync + 'static> Binding<T> {
    /// Part of a component's local state `S`, read with `get` and written with `set`: one
    /// checkbox per item of a set, say.
    pub fn local_part<S: FromWorld + Default + Send + Sync + 'static>(
        watch: Watch<Local<'static, S>>,
        get: impl Fn(&S) -> T + Send + Sync + 'static,
        set: impl Fn(&mut S, T) + Send + Sync + 'static,
    ) -> Self {
        let get = Arc::new(get);
        let (read, write, current) = (watch.clone(), watch, get.clone());
        Self {
            get: Arc::new(move |tracked| {
                if tracked {
                    read.with(|state| get(state))
                } else {
                    read.with_untracked(|state| get(state))
                }
            }),
            set: Arc::new(move |_, value| {
                if write.with_untracked(|state| current(state) != value) {
                    write.update(|state| set(state, value));
                }
            }),
        }
    }

    /// A field of resource `R`, read with `get` and written with `set`.
    pub fn resource<R: Resource<Mutability = Mutable> + Clone>(
        watch: Watch<Res<'static, R>>,
        get: impl Fn(&R) -> T + Send + Sync + Clone + 'static,
        set: impl Fn(&mut R, T) + Send + Sync + Clone + 'static,
    ) -> Self {
        let read = get.clone();
        Self {
            get: Arc::new(move |tracked| {
                if tracked {
                    watch.with(&read)
                } else {
                    watch.with_untracked(&read)
                }
            }),
            set: Arc::new(move |commands, value| {
                let (get, set) = (get.clone(), set.clone());
                commands.queue(move |world: &mut World| {
                    let mut resource = world.resource_mut::<R>();
                    if get(&resource) != value {
                        set(&mut resource, value);
                    }
                });
            }),
        }
    }

    /// The value, tracked: a `view!` region or patch that reads it updates when it changes.
    pub fn get(&self) -> T {
        (self.get)(true)
    }

    /// The value, untracked.
    pub fn get_untracked(&self) -> T {
        (self.get)(false)
    }

    /// Sets the value (if it differs).
    pub fn set(&self, commands: &mut Commands, value: T) {
        (self.set)(commands, value);
    }

    /// Sets the value to `f` of the current one.
    pub fn update(&self, commands: &mut Commands, f: impl FnOnce(&T) -> T) {
        let next = f(&self.get_untracked());
        self.set(commands, next);
    }
}

impl<T: Clone + PartialEq + Default + Send + Sync + 'static> Default for Binding<T> {
    /// Fresh local state at `T::default()`.
    fn default() -> Self {
        Self::local(Watch::default())
    }
}
