//! Types builtin to the 'static reflection' system
//!
//! These are mostly FFI-safe alternatives to the standard library
//! types.
use std::mem::MaybeUninit;
use crate::{StaticReflect, TypeInfo, field_offset};

use zerogc::{CollectorId, epsilon};
#[cfg(feature = "gc")]
use zerogc_derive::Trace;

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
impl<'a, T: 'a> From<&'a [T]> for AsmSlice<T> {
    #[inline]
    fn from(slice: &'a [T]) -> Self {
        AsmSlice { ptr: slice.as_ptr() as *mut _, len: slice.len() }
    }
}
unsafe impl<T: StaticReflect> StaticReflect for AsmSlice<T> {
    const TYPE_INFO: TypeInfo<'static> = TypeInfo::Slice {
        element_type: epsilon::gc(&T::TYPE_INFO)
    };
}

/// Assuming there is no mutation of the underlying memory,
/// this is safe to send between threads
unsafe impl<T: Sync> Send for AsmSlice<T> {}

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
#[cfg_attr(feature = "gc", derive(Trace))]
#[cfg_attr(feature = "gc", zerogc(nop_trace, copy))]
pub struct AsmStr {
    /// The underlying memory of the string
    #[cfg_attr(feature = "gc", zerogc(unsafe_skip_trace))]
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
    /// Check if the string is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.bytes.len == 0
    }
}
unsafe impl StaticReflect for AsmStr {
    const TYPE_INFO: TypeInfo<'static> = TypeInfo::Str;
}
impl<'a> From<&'a str> for AsmStr {
    fn from(s: &'a str) -> AsmStr {
        AsmStr { bytes: s.as_bytes().into() }
    }
}

/// A FFI-safe alternative to Rust's [std::option::Option].
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
impl AsmOption<()> {
    /// The offset of the 'present' field
    ///
    /// This should be zero, regardless of the inner type
    #[inline]
    pub const fn present_field_offset() -> usize {
        assert!(field_offset!(AsmOption::<()>, present) == 0);
        0
    }
    /// The offset of the value field
    ///
    /// This should be equal to the type's alignment
    #[inline]
    pub const fn value_field_offset<Id: CollectorId>(element_type: &TypeInfo<Id>) -> usize {
        element_type.alignment()
    }
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
    /// Create an option with a value
    #[inline]
    pub fn some(value: T) -> AsmOption<T> {
        AsmOption {
            present: true,
            value: MaybeUninit::new(value)
        }
    }
    /// Assume that this option is valid
    ///
    /// This type is often used to ferry things across FFI boundaries,
    /// so it's the callers repsonsibility to be safe with it.
    ///
    /// ## Safety
    /// The caller assumes that the underlying memory is valid.
    /// If not, undefined behavior will result.
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
impl<T> From<Option<T>> for AsmOption<T> {
    fn from(o: Option<T>) -> AsmOption<T> {
        match o {
            None => AsmOption::none(),
            Some(v) => AsmOption::some(v)
        }
    }
}
unsafe impl<T: StaticReflect> StaticReflect for AsmOption<T> {
    const TYPE_INFO: TypeInfo<'static> = TypeInfo::Optional(epsilon::gc(&T::TYPE_INFO));
}

/// This is an owned value, so it's safe to send
unsafe impl<T: Send> Send for AsmOption<T> {}
