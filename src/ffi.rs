pub use std::ffi::c_void;
pub use std::marker::PhantomData;
use std::os::raw::c_char;
pub use std::ptr::NonNull;
use std::{ffi::CStr, mem::MaybeUninit};

/// The null pointer
#[repr(transparent)]
#[derive(Copy, Clone, Default)]
pub struct Null(Option<&'static Never>);
enum Never {}

impl std::fmt::Debug for Null {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Null")
    }
}

/// An immutably borrowed, null-terminated utf-8 string, represented as
/// a non-null c 'const char*'.
#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct Str<'a> {
    _ptr: NonNull<c_char>,
    _lt: PhantomData<&'a ()>,
}

impl<'a> Str<'a> {
    /// Fails if not null terminated
    pub fn from(s: &'a str) -> Result<Self, <Self as TryFrom<&str>>::Error> {
        s.try_into()
    }
    // TODO: const checked constructor
    pub const unsafe fn new_unchecked(b: &'a [u8]) -> Self {
        Str {
            _ptr: NonNull::new_unchecked(
                CStr::from_bytes_with_nul_unchecked(b).as_ptr() as *mut c_char,
            ),
            _lt: PhantomData,
        }
    }
    pub fn as_str(self) -> &'a str {
        unsafe {
            std::str::from_utf8_unchecked(
                CStr::from_ptr(self._ptr.as_ptr()).to_bytes(),
            )
        }
    }
}

impl Default for Str<'_> {
    fn default() -> Self {
        const INST: Str<'static> = unsafe { Str::new_unchecked(b"\0") };
        INST
    }
}

// TODO: User-defined unsize, once the compiler allows that.

impl<'a> From<&'a CStr> for Str<'a> {
    fn from(cstring: &'a CStr) -> Self {
        // Safety: CStr::as_ptr is always non-null.
        Str {
            _ptr: unsafe { (&*cstring.as_ptr()).into() },
            _lt: PhantomData,
        }
    }
}

/// Fails if not null terminated
impl<'a> TryFrom<&'a str> for Str<'a> {
    type Error = std::ffi::FromBytesWithNulError;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Ok(CStr::from_bytes_with_nul(value.as_bytes())?.into())
    }
}

/// A c string stored as a null-terminated utf-8 string inside an array of
/// bytes of fixed length.
#[repr(transparent)]
#[derive(Clone)]
pub struct CharArray<const N: usize>([u8; N]);

impl<const N: usize> CharArray<N> {
    pub fn as_str(&self) -> &str {
        unsafe {
            let len = self.0.iter().position(|&c| c == 0).unwrap_unchecked();
            let slice = std::slice::from_raw_parts(self.0.as_ptr(), len);
            std::str::from_utf8_unchecked(slice)
        }
    }
}

impl<const N: usize> std::fmt::Debug for CharArray<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CharArray<")?;
        N.fmt(f)?;
        f.write_str(">(")?;
        self.as_str().fmt(f)?;
        f.write_str(")")
    }
}

impl<const N: usize, const M: usize> PartialEq<CharArray<M>> for CharArray<N> {
    fn eq(&self, other: &CharArray<M>) -> bool {
        self.as_str() == other.as_str()
    }
}

impl<'a, const N: usize> PartialEq<Str<'a>> for CharArray<N> {
    fn eq(&self, other: &Str<'a>) -> bool {
        self.as_str() == other.as_str()
    }
}

/// An owned contiguous sequence of T, represented as a u32 and an inline array.
#[repr(C)]
#[derive(Debug)]
pub struct InlineSlice<T, const N: usize> {
    count: u32,
    value: [MaybeUninit<T>; N],
}

impl<T, const N: usize> InlineSlice<T, N> {
    pub fn len(&self) -> u32 {
        self.count
    }
    /// Convert back into a normal rust slice
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        unsafe {
            std::slice::from_raw_parts(
                self.value.as_ptr() as *const T,
                self.count as usize,
            )
        }
    }
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.as_slice().iter()
    }
}

impl<'a, T, const N: usize> std::iter::IntoIterator for &'a InlineSlice<T, N> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().into_iter()
    }
}

impl<T, const N: usize> Default for InlineSlice<T, N> {
    fn default() -> Self {
        Self {
            count: 0,
            // The MaybeUninit members are initialized when uninitialized
            value: unsafe { MaybeUninit::uninit().assume_init() },
        }
    }
}

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct UUID([u8; 16]);

// Note: Be *very* careful about how this is aligned in the outer struct.

/// An immutably borrowed contiguous sequence of T. Represented as a u32
/// followed by a pointer.
#[repr(C)]
#[derive(Debug)]
pub struct Slice<'a, T> {
    count: u32,
    ptr: *const T,
    _lt: PhantomData<&'a T>,
}

impl<'a, T> Copy for Slice<'a, T> {}
impl<'a, T> Clone for Slice<'a, T> {
    fn clone(&self) -> Self {
        Self { count: self.count, ptr: self.ptr, _lt: self._lt }
    }
}

impl<'a, T> Slice<'a, T> {
    pub fn from(arr: &'a [T]) -> Self {
        arr.into()
    }
    pub fn len(&self) -> u32 {
        self.count
    }
    /// Convert back into a normal rust slice
    pub fn as_slice(&self) -> &'a [T] {
        unsafe {
            let len = self.count as usize;
            std::slice::from_raw_parts(self.ptr, len)
        }
    }
}

impl<'a, T> Default for Slice<'a, T> {
    fn default() -> Self {
        (&[]).into()
    }
}

/// Panics if the slice has 2^32 or more elements
impl<'a, T> From<&'a [T]> for Slice<'a, T> {
    fn from(ts: &'a [T]) -> Self {
        Slice {
            count: ts.len().try_into().unwrap(),
            ptr: ts.as_ptr(),
            _lt: PhantomData,
        }
    }
}

/// Panics if the slice has 2^32 or more elements
impl<'a, T> From<&'a mut [T]> for Slice<'a, T> {
    fn from(ts: &'a mut [T]) -> Self {
        (&*ts).into()
    }
}

/// Panics if the Vec has 2^32 or more elements
impl<'a, T> From<&'a Vec<T>> for Slice<'a, T> {
    fn from(ts: &'a Vec<T>) -> Self {
        Slice {
            count: ts.len().try_into().unwrap(),
            ptr: ts.as_ptr(),
            _lt: PhantomData,
        }
    }
}

impl<'a, T, const N: usize> From<&'a [T; N]> for Slice<'a, T> {
    fn from(ts: &'a [T; N]) -> Self {
        Self {
            count: N as u32,
            ptr: ts.as_ptr(),
            _lt: PhantomData,
        }
    }
}

impl<'a, T> std::iter::IntoIterator for Slice<'a, T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().into_iter()
    }
}

/// An immutably borrowed contiguous sequence of T. Represented as a u32
/// followed by a pointer. This type differs from Slice only in that it is
/// aligned to a 4-byte boundary, for cases where the structure alignment of
/// Slice puts the count member in the wrong place on 64 bit systems. This type
/// does not use unaligned loads or stores and has no special alignment
/// requirement itself.
#[repr(C)]
#[derive(Debug)]
pub struct Slice_<'a, T> {
    count: u32,
    #[cfg(target_pointer_width = "32")]
    ptr: u32,
    // Avoid unaligned stores
    #[cfg(target_pointer_width = "64")]
    ptr: [u32; 2],
    _lt: PhantomData<&'a T>,
}

impl<'a, T> Copy for Slice_<'a, T> {}
impl<'a, T> Clone for Slice_<'a, T> {
    fn clone(&self) -> Self {
        Self { count: self.count, ptr: self.ptr, _lt: self._lt }
    }
}

impl<'a, T> Default for Slice_<'a, T> {
    fn default() -> Self {
        (&[]).into()
    }
}

impl<'a, T> Slice_<'a, T> {
    pub fn from(arr: &'a [T]) -> Self {
        arr.into()
    }
    pub fn len(&self) -> u32 {
        self.count
    }
    /// Convert back into a normal rust slice
    pub fn as_slice(&self) -> &'a [T] {
        unsafe {
            let ptr = std::mem::transmute(self.ptr);
            let len = self.count as usize;
            std::slice::from_raw_parts(ptr, len)
        }
    }
}

/// Panics if the slice has 2^32 or more elements
impl<'a, T> From<&'a [T]> for Slice_<'a, T> {
    fn from(ts: &'a [T]) -> Self {
        Slice_ {
            count: ts.len().try_into().unwrap(),
            ptr: unsafe { std::mem::transmute(ts.as_ptr()) },
            _lt: PhantomData,
        }
    }
}

/// Panics if the slice has 2^32 or more elements
impl<'a, T> From<&'a mut [T]> for Slice_<'a, T> {
    fn from(ts: &'a mut [T]) -> Self {
        (&*ts).into()
    }
}

impl<'a, T, const N: usize> From<&'a [T; N]> for Slice_<'a, T> {
    fn from(ts: &'a [T; N]) -> Self {
        Self {
            count: N as u32,
            ptr: unsafe { std::mem::transmute(ts.as_ptr()) },
            _lt: PhantomData,
        }
    }
}

impl<'a, T> std::iter::IntoIterator for Slice_<'a, T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().into_iter()
    }
}

/// An immutably borrowed contiguous sequence of bytes. Represented as a usize
/// followed by a pointer.
#[repr(C)]
#[derive(Debug)]
pub struct Bytes<'a> {
    len: usize,
    ptr: *const u8,
    _lt: PhantomData<&'a u8>,
}

impl<'a> Copy for Bytes<'a> {}
impl<'a> Clone for Bytes<'a> {
    fn clone(&self) -> Self {
        Self { len: self.len, ptr: self.ptr, _lt: self._lt }
    }
}

impl<'a> Bytes<'a> {
    pub fn from(arr: &'a [u8]) -> Self {
        arr.into()
    }
    pub fn len(&self) -> usize {
        self.len
    }
    /// Convert back into a normal rust slice
    pub fn as_slice(&self) -> &'a [u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }
}

impl<'a> Default for Bytes<'a> {
    fn default() -> Self {
        Self::from(&[])
    }
}

impl<'a> From<&'a [u8]> for Bytes<'a> {
    fn from(slice: &'a [u8]) -> Self {
        Bytes {
            len: slice.len(),
            ptr: slice.as_ptr(),
            _lt: PhantomData,
        }
    }
}

impl<'a> From<&'a [u32]> for Bytes<'a> {
    fn from(slice: &'a [u32]) -> Self {
        Bytes {
            len: slice.len() * 4,
            ptr: slice.as_ptr() as *const u8,
            _lt: PhantomData,
        }
    }
}

impl<'a> From<&'a Vec<u8>> for Bytes<'a> {
    fn from(vec: &'a Vec<u8>) -> Self {
        Bytes {
            len: vec.len(),
            ptr: vec.as_ptr(),
            _lt: PhantomData,
        }
    }
}

/// An immutably borrowed contiguous nonempty sequence of T. Represented as a
/// non-null pointer.
#[repr(transparent)]
#[derive(Debug)]
pub struct Array<'a, T> {
    _ptr: NonNull<T>,
    _lt: PhantomData<&'a T>,
}

impl<'a, T> Copy for Array<'a, T> {}
impl<'a, T> Clone for Array<'a, T> {
    fn clone(&self) -> Self {
        Self { _ptr: self._ptr, _lt: self._lt }
    }
}

impl<'a, T, const N: usize> From<&'a [T; N]> for Array<'a, T> {
    fn from(array: &'a [T; N]) -> Self {
        let _array_must_be_non_empty = N - 1;
        Self {
            _ptr: unsafe { NonNull::new_unchecked(array.as_ptr() as *mut T) },
            _lt: PhantomData,
        }
    }
}

impl<'a, T> Array<'a, T> {
    pub fn from_slice(slice: &'a [T]) -> Option<Array<'a, T>> {
        if slice.is_empty() {
            None
        } else {
            Some(Self {
                _ptr: unsafe {
                    NonNull::new_unchecked(slice.as_ptr() as *mut T)
                },
                _lt: PhantomData,
            })
        }
    }
    /// Convert back into a normal rust slice
    pub unsafe fn as_slice(self, len: u32) -> &'a [T] {
        std::slice::from_raw_parts(self._ptr.as_ptr(), len as usize)
    }
}

/// A mutably borrowed contiguous nonempty sequence of T. Represented as a
/// non-null pointer.
#[repr(transparent)]
#[derive(Debug)]
pub struct ArrayMut<'a, T> {
    _ptr: NonNull<T>,
    _lt: PhantomData<&'a mut T>,
}

impl<'a, T, const N: usize> From<&'a mut [T; N]> for ArrayMut<'a, T> {
    fn from(array: &'a mut [T; N]) -> Self {
        let _array_must_be_non_empty = N - 1;
        Self {
            _ptr: unsafe { NonNull::new_unchecked(array.as_ptr() as *mut T) },
            _lt: PhantomData,
        }
    }
}

impl<'a, T> ArrayMut<'a, T> {
    pub fn from_slice(slice: &'a mut [T]) -> Option<ArrayMut<'a, T>> {
        if slice.is_empty() {
            None
        } else {
            Some(Self {
                _ptr: unsafe {
                    NonNull::new_unchecked(slice.as_ptr() as *mut T)
                },
                _lt: PhantomData,
            })
        }
    }
}
