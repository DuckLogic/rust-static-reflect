//! Implementations of [StaticReflect] for core types (for `#![no_std]`)
use crate::StaticReflect;
use crate::types::{TypeInfo};

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
            #[allow(unused_comparisons)]
            const TYPE_INFO: TypeInfo<'static> = {
                let size = std::mem::size_of::<$target>();
                let signed = <$target>::MIN < 0;
                TypeInfo::integer(size, signed)
            };
        })*
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
