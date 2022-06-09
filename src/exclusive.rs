// Copied from https://github.com/rust-lang/rust/pull/97629

#[derive(Default)]
#[repr(transparent)]
pub struct Exclusive<T> {
    inner: T,
}

// Safety: 'inner' is inaccessible from a shared ref
unsafe impl<T> Sync for Exclusive<T> {}

impl<T> std::fmt::Debug for Exclusive<T> {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> Result<(), std::fmt::Error> {
        f.debug_struct("Exclusive").finish_non_exhaustive()
    }
}

impl<T> Exclusive<T> {
    /// Wrap a value in an `Exclusive`
    #[must_use]
    pub const fn new(t: T) -> Self {
        Self { inner: t }
    }
    /// Unwrap the value contained in the `Exclusive`
    // #[must_use]
    // pub fn into_inner(self) -> T {
    //     self.inner
    // }
    /// Get exclusive access to the underlying value.
    #[must_use]
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}
