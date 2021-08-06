//! Implementations of [StaticReflect] for core types (for `#![no_std]`)
use crate::{StaticReflect, PrimInt, PrimFloat};
use crate::types::{TypeInfo, IntSize, IntType, SimpleNonZeroRepr, FloatSize};
use std::mem::{self, ManuallyDrop};
use core::ptr::NonNull;
use std::num::{NonZeroI32, NonZeroU32, NonZeroU8, NonZeroUsize};

macro_rules! impl_primitive {
    ($target:ty => $info:expr) => {
        unsafe impl StaticReflect for $target {
            const TYPE_INFO: TypeInfo<'static> = $info;
        }
    }
}
macro_rules! impl_ints {
    ($($target:ty),*) => {
        $(unsafe impl StaticReflect for $target {
            const TYPE_INFO: TypeInfo<'static> = TypeInfo::Integer(Self::INT_TYPE);
        }
        unsafe impl PrimInt for $target {
            const INT_SIZE: IntSize = IntSize::unwrap_from_bytes(std::mem::size_of::<Self>());
            #[allow(unused_comparisons)]
            const SIGNED: bool = <$target>::MIN < 0;
            const INT_TYPE: IntType = IntType { size: Self::INT_SIZE, signed: Self::SIGNED };
        }
        impl crate::sealed::Sealed for $target {})*
    }
}
// NOTE: Pointer sized integers have machine-dependent implementation :(
impl_ints!(u8, u16, u32, u64, i8, i16, i32, i64, usize, isize);

#[cfg(feature = "builtins")]
impl_primitive!(str => TypeInfo::Str);
impl_primitive!(() => TypeInfo::Unit);
impl_primitive!(bool => TypeInfo::Bool);
impl_primitive!(f32 => TypeInfo::F32);
impl_primitive!(f64 => TypeInfo::F64);
impl crate::sealed::Sealed for f32 {}
unsafe impl PrimFloat for f32 {
    const FLOAT_SIZE: FloatSize = FloatSize::Single;
}
impl crate::sealed::Sealed for f64 {}
unsafe impl PrimFloat for f64 {
    const FLOAT_SIZE: FloatSize = FloatSize::Double;
}

// Builtin support for the never type
impl_primitive!(! => TypeInfo::Never);


/// Support [StaticReflect] for [ManuallyDrop] by just representing the inner type
unsafe impl<T: StaticReflect> StaticReflect for ManuallyDrop<T> {
    const TYPE_INFO: TypeInfo<'static> = {
        assert!(mem::size_of::<Self>() == mem::size_of::<T>());
        T::TYPE_INFO
    };
}

/// A pointer
///
/// NOTE: The pointed-to value can be anything,
/// even if it doesn't implement [StaticReflect].
///
/// This is fine since the static reflection system
/// doesn't maintain
/// information about pointers (to avoid cycles).
unsafe impl <T> StaticReflect for *mut T {
    const TYPE_INFO: TypeInfo<'static> = TypeInfo::Pointer;
}
/// An immutable pointer
///
/// The static reflection system makes no distinction between
/// mutable and immutable pointers.
unsafe impl <T> StaticReflect for *const T {
    const TYPE_INFO: TypeInfo<'static> = TypeInfo::Pointer;
}

unsafe impl <T> SimpleNonZeroRepr for NonNull<T> {}
unsafe impl <T> StaticReflect for NonNull<T> {
    const TYPE_INFO: TypeInfo<'static> = TypeInfo::Pointer;
}
unsafe impl SimpleNonZeroRepr for NonZeroUsize {}
unsafe impl StaticReflect for NonZeroUsize {
    const TYPE_INFO: TypeInfo<'static> = <usize as StaticReflect>::TYPE_INFO;
}
unsafe impl SimpleNonZeroRepr for NonZeroU32 {}
unsafe impl StaticReflect for NonZeroU32 {
    const TYPE_INFO: TypeInfo<'static> = <u32 as StaticReflect>::TYPE_INFO;
}
unsafe impl SimpleNonZeroRepr for NonZeroU8 {}
unsafe impl StaticReflect for NonZeroU8 {
    const TYPE_INFO: TypeInfo<'static> = <u8 as StaticReflect>::TYPE_INFO;
}
unsafe impl SimpleNonZeroRepr for NonZeroI32 {}
unsafe impl StaticReflect for NonZeroI32 {
    const TYPE_INFO: TypeInfo<'static> = <i32 as StaticReflect>::TYPE_INFO;
}


unsafe impl <T: SimpleNonZeroRepr> StaticReflect for Option<T> {
    /// We have the representation as our internals,
    /// except for the fact we might be null
    const TYPE_INFO: TypeInfo<'static> = <T as StaticReflect>::TYPE_INFO;
}
