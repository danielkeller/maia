use std::ffi::CStr;
use std::marker::PhantomData;
use std::os::raw::c_char;
use std::ptr::NonNull;

/// A non-null, immutably borrowed, null-terminated utf-8 string, represented as
/// a c 'const char*'.
#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct Str<'a> {
    #[allow(dead_code)]
    ptr: NonNull<c_char>,
    _lt: PhantomData<&'a ()>,
}

impl<'a> Str<'a> {
    pub fn from(s: &'a str) -> Result<Self, <Self as TryFrom<&str>>::Error> {
        s.try_into()
    }
    // TODO: const checked constructor
    pub const unsafe fn new_unchecked(b: &'a [u8]) -> Self {
        Str {
            ptr: NonNull::new_unchecked(
                CStr::from_bytes_with_nul_unchecked(b).as_ptr() as *mut c_char,
            ),
            _lt: PhantomData,
        }
    }
}

// TODO: User-defined unsize, once the compiler allows that.

impl<'a> From<&'a CStr> for Str<'a> {
    fn from(cstring: &'a CStr) -> Self {
        // Safety: CStr::as_ptr is always non-null.
        Str {
            ptr: unsafe { (&*cstring.as_ptr()).into() },
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

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct UUID([u8; 16]);

// Note: Be *very* careful about how this is aligned in the outer struct.

/// A borrowed contiguous sequence of T. Represented as a u32 followed by a
/// pointer.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Slice<'a, T> {
    count: u32,
    ptr: *const T,
    _lt: PhantomData<&'a T>,
}

impl<'a, T> Slice<'a, T> {
    pub fn from(arr: &'a [T]) -> Self {
        arr.into()
    }
    pub fn len(&self) -> u32 {
        self.count
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
impl<'a, T, const N: usize> From<&'a [T; N]> for Slice<'a, T> {
    fn from(ts: &'a [T; N]) -> Self {
        ts.as_slice().into()
    }
}

#[cfg(target_pointer_width = "32")]
type Slice_<'a, T> = Slice<'a, T>;

/// A borrowed contiguous sequence of T. Represented as a u32 followed by a
/// pointer. This type differs from Slice only in that it is aligned to a 4-byte
/// boundary, for cases where the structure alignment of Slice puts the count
/// member in the wrong place on 64 bit systems.
#[cfg(target_pointer_width = "64")]
#[repr(C)]
#[derive(Debug)]
pub struct Slice_<'a, T> {
    count: u32,
    // Avoid unaligned stores
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

#[cfg(target_pointer_width = "64")]
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

#[cfg(target_pointer_width = "64")]
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

#[cfg(target_pointer_width = "64")]
/// Panics if the slice has 2^32 or more elements
impl<'a, T, const N: usize> From<&'a [T; N]> for Slice_<'a, T> {
    fn from(ts: &'a [T; N]) -> Self {
        ts.as_slice().into()
    }
}
