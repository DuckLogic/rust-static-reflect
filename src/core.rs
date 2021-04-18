use crate::StaticReflect;
use crate::types::{TypeInfo, IntSize};

macro_rules! impl_primitive {
    ($target:ty => $info:expr) => {
        unsafe impl StaticReflect for $target {
            const TYPE_INFO: &'static TypeInfo<'static> = &$info;
        }
    }
}
macro_rules! impl_ints {
    ($($target:ty),*) => {
        $(unsafe impl StaticReflect for $target {
            #[allow(unused_comparisons)]
            const TYPE_INFO: &'static TypeInfo<'static> = &TypeInfo::Integer {
                size: {
                    let size = std::mem::size_of::<$target>();
                    match IntSize::from_bytes(size) {
                        Ok(s) => s,
                        Err(_) => panic!("Invalid size")
                    }
                },
                signed: <$target>::MIN < 0,
            };
        })*
    }
}
// NOTE: Pointer sized integers have machine-dependent implementation :(
impl_ints!(u8, u16, u32, u64, i8, i16, i32, i64, usize, isize);

#[cfg(feature = "builtins")]
impl_primitive!(str => TypeInfo::Str);
impl_primitive!(() => TypeInfo::Unit);
unsafe impl <T: StaticReflect> StaticReflect for *mut T {
    const TYPE_INFO: &'static TypeInfo<'static> = &TypeInfo::Pointer;
}
unsafe impl <T: StaticReflect> StaticReflect for *const T {
    const TYPE_INFO: &'static TypeInfo<'static> = &TypeInfo::Pointer;
}
