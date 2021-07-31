/// Define a type's implementation of [StaticReflect](crate::StaticReflect) as an extern type
///
/// See [TypeInfo::Extern](crate::types::TypeInfo::Extern)
#[macro_export]
macro_rules! define_extern_type {
    ($target:ident) => ($crate::define_extern_type!($target => $target););
    ($target:ty => $defined_path:path) => {
        unsafe impl ::static_reflect::StaticReflect for $target {
            const TYPE_INFO: $crate::TypeInfo<'static> = $crate::TypeInfo::Extern {
                name: stringify!($defined_path)
            };
        }
    };
}

/// Get the integer offset of the specified field
#[macro_export]
macro_rules! field_offset {
    ($target:path, $($field:ident),+) => {
        unsafe {
            let uninit = core::mem::MaybeUninit::<$target>::uninit();
            let base = uninit.as_ptr();
            (core::ptr::addr_of!((*base)$(.$field)*).cast::<u8>().offset_from(base as *const u8) as usize)
        }
    }
}
