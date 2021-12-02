/// Define a type's implementation of [StaticReflect](crate::StaticReflect) as an extern type
///
/// See [TypeInfo::Extern](crate::types::TypeInfo::Extern)
#[macro_export]
macro_rules! define_extern_type {
    ($target:ident) => ($crate::define_extern_type!($target => $target););
    ($target:ty => $defined_path:path) => {
        unsafe impl $crate::StaticReflect for $target {
            const TYPE_INFO: $crate::TypeInfo<'static> = $crate::TypeInfo::Extern {
                name: zerogc::epsilon::gc_str(stringify!($defined_path))
            };
        }
    };
}

/// Get the integer offset of the specified field
///
/// This is only well defined for `#[repr(C)]` types,
/// since `#[repr(Rust)]` types don't have a well-defined layout.
///
/// ## Examples
/// ````
/// # use static_reflect::field_offset;
/// # #[repr(C)]
/// # struct Nested {
/// #     nested: u32
/// # }
/// #[repr(C)]
/// struct Example {
///     first: u32,
///     second: u8,
///     third: Nested,
/// }
/// assert_eq!(field_offset!(Example, first), 0);
/// // You may specify the expected type by suffixing with `as $expected_type`
/// assert_eq!(field_offset!(Example, second as u8), 4);
/// // Fields can be arbitrarily nested
/// assert_eq!(field_offset!(Example, third.nested as u32), 8);
/// ````
///
/// ## Const eval
/// Assuming you specify the appropriate feature flags,
/// this macro can also be used in a const-eval context.
/// ````
/// #![feature(const_ptr_offset_from)]
/// # #![deny(unused_features, stable_features)]
/// # use static_reflect::field_offset;
/// struct Example {
///     first: u32,
///     second: u8,
/// }
/// const FIRST_OFFSET: usize = field_offset!(Example, first as u32);
/// assert_eq!(FIRST_OFFSET, 0);
/// const SECOND_OFFSET: usize = field_offset!(Example, second as u8);
/// assert_eq!(SECOND_OFFSET, 4);
/// ````
#[macro_export]
macro_rules! field_offset {
    ($target:path, $($field:tt).+) => (field_offset!($target, $($field).* as _));
    ($target:path, $($field:tt).+ as $expected_type:ty) => {
        unsafe {
            let uninit = core::mem::MaybeUninit::<$target>::uninit();
            let base = uninit.as_ptr();
            let ptr: *const $expected_type = core::ptr::addr_of!((*base)$(.$field)*);
            ptr.cast::<u8>().offset_from(base as *const u8) as usize
        }
    }
}
