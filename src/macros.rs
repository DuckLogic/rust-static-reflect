/// Define a type's implementation of [StaticReflect](crate::StaticReflect) as an extern type
///
/// See [TypeInfo::Extern](crate::types::TypeInfo::Extern)
#[macro_export]
macro_rules! define_extern_type {
    ($target:ident) => (define_extern_type!($target => $target););
    ($target:ty => $defined_path:path) => {
        impl ::static_reflect::StaticReflect for $target {
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
            /*
             * I'm going to assume the dereference is safe,
             * because of the presense of '(*uninit.as_mut_ptr()).field'
             * in the documentation for std::ptr::addr_of_mut
             */
            (std::ptr::addr_of!((*(1 as *mut $target))$(.$field)*) as usize) - 1
        }
    }
}
