use crate::{store_value, RwSignal, Scope, StoredValue, WriteSignal};

/// A wrapper for any kind of settable reactive signal: a [WriteSignal](crate::WriteSignal),
/// [RwSignal](crate::RwSignal), or closure that receives a value and sets a signal depending
/// on it.
///
/// This allows you to create APIs that take any kind of `SignalSetter<T>` as an argument,
/// rather than adding a generic `F: Fn(T)`. Values can be set with the same
/// function call or `set()`, API as other signals.
///
/// ```rust
/// # use leptos_reactive::*;
/// # create_scope(create_runtime(), |cx| {
/// let (count, set_count) = create_signal(cx, 2);
/// let set_double_input = SignalSetter::map(cx, move |n| set_count(n * 2));
///
/// // this function takes any kind of signal setter
/// fn set_to_4(setter: &SignalSetter<i32>) {
///   // ✅ calling the signal sets the value
///   //    it is a shorthand for arg.set()
///   setter(4);
/// }
///
/// set_to_4(&set_count.into());
/// assert_eq!(count(), 4);
/// set_to_4(&set_double_input);
/// assert_eq!(count(), 8);
/// # });
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct SignalSetter<T>(SignalSetterTypes<T>)
where
    T: 'static;

impl<T> Clone for SignalSetter<T> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<T> Copy for SignalSetter<T> {}

impl<T> SignalSetter<T>
where
    T: 'static,
{
    /// Wraps a signal-setting closure, i.e., any computation that sets one or more
    /// reactive signals.
    /// ```rust
    /// # use leptos_reactive::*;
    /// # create_scope(create_runtime(), |cx| {
    /// let (count, set_count) = create_signal(cx, 2);
    /// let set_double_count = SignalSetter::map(cx, move |n| set_count(n * 2));
    ///
    /// // this function takes any kind of signal setter
    /// fn set_to_4(setter: &SignalSetter<i32>) {
    ///   // ✅ calling the signal sets the value
    ///   //    it is a shorthand for arg.set()
    ///   setter(4)
    /// }
    ///
    /// set_to_4(&set_count.into());
    /// assert_eq!(count(), 4);
    /// set_to_4(&set_double_count);
    /// assert_eq!(count(), 8);
    /// # });
    /// ```
    pub fn map(cx: Scope, mapped_setter: impl Fn(T) + 'static) -> Self {
        Self(SignalSetterTypes::Mapped(
            cx,
            store_value(cx, Box::new(mapped_setter)),
        ))
    }

    /// Calls the setter function with the given value.
    ///
    /// ```rust
    /// # use leptos_reactive::*;
    /// # create_scope(create_runtime(), |cx| {
    /// let (count, set_count) = create_signal(cx, 2);
    /// let set_double_count = SignalSetter::map(cx, move |n| set_count(n * 2));
    ///
    /// // this function takes any kind of signal setter
    /// fn set_to_4(setter: &SignalSetter<i32>) {
    ///   // ✅ calling the signal sets the value
    ///   //    it is a shorthand for arg.set()
    ///   setter(4);
    /// }
    ///
    /// set_to_4(&set_count.into());
    /// assert_eq!(count(), 4);
    /// set_to_4(&set_double_count);
    /// assert_eq!(count(), 8);
    /// # });
    pub fn set(&self, value: T) {
        match &self.0 {
            SignalSetterTypes::Write(s) => s.set(value),
            SignalSetterTypes::Mapped(_, s) => s.with(|s| s(value)),
        }
    }
}

impl<T> From<WriteSignal<T>> for SignalSetter<T> {
    fn from(value: WriteSignal<T>) -> Self {
        Self(SignalSetterTypes::Write(value))
    }
}

impl<T> From<RwSignal<T>> for SignalSetter<T> {
    fn from(value: RwSignal<T>) -> Self {
        Self(SignalSetterTypes::Write(value.write_only()))
    }
}

enum SignalSetterTypes<T>
where
    T: 'static,
{
    Write(WriteSignal<T>),
    Mapped(Scope, StoredValue<Box<dyn Fn(T)>>),
}

impl<T> Clone for SignalSetterTypes<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Write(arg0) => Self::Write(*arg0),
            Self::Mapped(cx, f) => Self::Mapped(*cx, *f),
        }
    }
}

impl<T> Copy for SignalSetterTypes<T> {}

impl<T> std::fmt::Debug for SignalSetterTypes<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Write(arg0) => f.debug_tuple("WriteSignal").field(arg0).finish(),
            Self::Mapped(_, _) => f.debug_tuple("Mapped").finish(),
        }
    }
}

impl<T> PartialEq for SignalSetterTypes<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Write(l0), Self::Write(r0)) => l0 == r0,
            (Self::Mapped(_, l0), Self::Mapped(_, r0)) => std::ptr::eq(l0, r0),
            _ => false,
        }
    }
}

impl<T> Eq for SignalSetterTypes<T> where T: PartialEq {}

#[cfg(not(feature = "stable"))]
impl<T> FnOnce<(T,)> for SignalSetter<T>
where
    T: 'static,
{
    type Output = ();

    extern "rust-call" fn call_once(self, args: (T,)) -> Self::Output {
        self.set(args.0)
    }
}

#[cfg(not(feature = "stable"))]
impl<T> FnMut<(T,)> for SignalSetter<T>
where
    T: 'static,
{
    extern "rust-call" fn call_mut(&mut self, args: (T,)) -> Self::Output {
        self.set(args.0)
    }
}

#[cfg(not(feature = "stable"))]
impl<T> Fn<(T,)> for SignalSetter<T>
where
    T: 'static,
{
    extern "rust-call" fn call(&self, args: (T,)) -> Self::Output {
        self.set(args.0)
    }
}
