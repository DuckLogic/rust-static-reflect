//! A system for getting static type information.
//!
//! This effectively gives (some) of the power of reflection
//! without any of the runtime cost!
//!
//! The original use case was type-checking generated code in a JIT compiler (with zero runtime cost).
//! However, other use cases are certainly possible :)
//!
//! Contributions are welcome!
//! I'd be happy to add more features as long as they align with the general philosophy
//! of compile-time reflection.
#![deny(missing_docs)]
#![feature(
    const_panic, const_option, // We use Option::unwrap
    const_fn_fn_ptr_basics, // We use PhantomData<fn() -> T>
    const_fn_trait_bound,
    // Used for field_offset macro
    const_raw_ptr_deref,
    const_maybe_uninit_as_ptr,
    const_ptr_offset_from,
)]
#![cfg_attr(feature = "never", feature(never_type))]

mod macros;
#[cfg(feature = "builtins")]
pub mod builtins;
pub mod types;
pub mod funcs;

mod core;

pub use crate::types::TypeInfo;

/// The trait for types whose information can be accessed via static reflection.
///
/// In order to proper access any fields,
/// the representation must be C-compatible.
/// Otherwise, transmutes and pointer-arithmetic are
/// pointless because they are already undefined behavior.
///
/// ## Safety
/// Incorrect implementation of this trait is considered
/// undefined behavior. All the static type information must
/// be correct at runtime.
/// 
/// For example, if this type gives field information via [FieldReflect],
/// then the field information **must** match the representation
/// at runtime.
/// 
/// The advantage of this is that other crates can rely
/// on the representation being stable (for example, JIT compilers can use it).
/// 
/// The type must be `#[repr(C)]` or have some other
/// form of FFI safety.
pub unsafe trait StaticReflect {
    /// The static information about the type's representation
    const TYPE_INFO: TypeInfo<'static>;
}

/// A type that supports accessing its fields via reflection.
/// 
/// All fields are assumed to be defined in a way that is compatible
/// with the the C ABI. In other words, the type must be `#[repr(C)]`
///
/// ## Safety
/// Implementing this type incorrectly is undefined behavior.
pub unsafe trait FieldReflect: StaticReflect {
    /// A magic structure that can be used to access
    /// field info by name
    type NamedFieldInfo;
    /// Static information on this structure's fields,
    /// where each field's information is given by name
    const NAMED_FIELD_INFO: Self::NamedFieldInfo;
}
