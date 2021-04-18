//! Types builtin to the 'static reflection' system
//!
//! These are mostly FFI-safe alternatives to the standard library
//! types.
use std::mem::MaybeUninit;
use crate::StaticReflect;

/// A FFi-safe slice type (`&[T]`)
/// 
/// Unlike the rust type, this has a well-defined C representation.
///
/// Internally, this is just a pointer and a length.
///
/// ## Safety
/// This type maintains no in variants on its internal data.
///
/// However, since it is meant to be used with FFI and in other
/// unsafe situations, this is often fine.
/// The plus side is you can transmute to/from `[usize; 2]`
/// without fear.
#[derive(Debug)]
#[repr(C)]
pub struct AsmSlice<T> {
    /// A pointer to the start of the memory
    ///
    /// May never be null, unless
    pub ptr: *mut T,
    /// The length of the slice
    pub len: usize
}
/// A clone implementation that blindly
/// copies the underlying bytes.
impl<T: StaticReflect> Clone for AsmSlice<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: StaticReflect> Copy for AsmSlice<T> {}


/// A FFI-safe UTF8 string.
///
/// Unlike the rust type, this has a well-defined C representation.
///
/// ## Safety
/// The underlying is expected to be UTF8. However,
/// like its [AsmSlice] counterpart, all fields are public
/// and this type does not maintain any invariants.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct AsmStr {
    /// The underlying memory of the string
    pub bytes: AsmSlice<u8>
}
impl AsmStr {
    /// A pointer to the bytes of the string
    #[inline]
    pub fn bytes_ptr(&self) -> *mut u8 {
        self.bytes.ptr
    }
    /// The length of the string in bytes
    #[inline]
    pub fn len(&self) -> usize {
        self.bytes.len
    }
}

/// A FFI-safe alternative to Rust's [std::optional::Option].
///
/// Unlike the Rust type, this does not use the null-pointer
/// optimization.
///
/// NOTE: This type doesn't implement Drop.
///
/// ## Safety
/// This type does not enforce its safety variants,
/// just like [AsmSlice]. However, the first field must be
/// a valid `bool`.
///
/// A valid type can only be in one of two states:
/// 1. `{present: false, value: undefined}`
/// 2. `{present: false, value: any}`
#[derive(Debug)]
#[repr(C)]
pub struct AsmOption<T> {
    present: bool,
    value: MaybeUninit<T>
}
impl<T> AsmOption<T> {
    /// An option with no value
    #[inline]
    pub fn none() -> AsmOption<T> {
        AsmOption {
            present: false,
            value: MaybeUninit::uninit()
        }
    }
    /// An option with a value
    #[inline]
    pub fn some(value: T) -> AsmOption<T> {
        AsmOption {
            present: true,
            value: MaybeUninit::new(value)
        }
    }
    /// Assume that this option is valid
    ///
    /// Technically, it should already be invalid
    /// to have undefined internals.
    /// However, this is still unsafe as a sort of lint.
    #[inline]
    pub unsafe fn assume_valid(self) -> Option<T> {
        if self.present {
            Some(self.value.assume_init())
        } else {
            None
        }
    }
    /// If the value of the option is present.
    #[inline]
    pub fn is_present(&self) -> bool {
        self.present
    }
}
