//! App-wide reactive values. A [`Signal`] is a `bevy_radioactive` watch over local state
//! that lives outside any one component: the app creates it once (in [`Nav`](crate::Nav) or
//! [`Vault`](crate::Vault)) and hands clones to systems, which read and set it, and to
//! `view!` scenes, which read it with `$` and update when it changes. Every clone refers to
//! the same value.

use bevy::prelude::Local;
use bevy_radioactive::view::Watch;

/// A shared value that views react to.
pub type Signal<T> = Watch<Local<'static, T>>;

/// A new signal holding `value()`.
pub fn signal<T: Default + Send + Sync + 'static>(value: impl FnOnce() -> T) -> Signal<T> {
    Watch::new(value())
}

/// A value computed from signals. Reading it runs the computation, and the signals it reads
/// are tracked as the reader's own, so a view that reads `$memo` updates when they change.
pub struct Memo<T>(std::sync::Arc<dyn Fn() -> T + Send + Sync>);

impl<T> Clone for Memo<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Memo<T> {
    /// Computes the value now.
    pub fn get(&self) -> T {
        (self.0)()
    }

    /// Computes the value and reads it through `f`.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        f(&self.get())
    }
}

/// A value computed from signals by `f` (see [`Memo`]).
pub fn memo<T>(f: impl Fn() -> T + Send + Sync + 'static) -> Memo<T> {
    Memo(std::sync::Arc::new(f))
}
