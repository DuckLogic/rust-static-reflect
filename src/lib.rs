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
    const_option, // We use Option::unwrap
)]
#![cfg_attr(feature = "never", feature(never_type))]

mod macros;
#[cfg(feature = "builtins")]
pub mod builtins;
pub mod types;
pub mod funcs;

mod core;

pub use crate::types::TypeInfo;

use crate::types::{IntType, IntSize, FloatSize};
use std::ops::{Sub, Mul, Add};

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
    const TYPE_INFO: TypeInfo;
}

/// A primitive integer type
///
/// ## Safety
/// Must actually be one of the primitive integer types.
///
/// Trait info must accurately reflect the underlying type.
pub unsafe trait PrimInt: StaticReflect + Copy + Add<Self> + Sub<Self> + Mul<Self> + sealed::Sealed {
    /// The [IntType] of this integer
    const INT_TYPE: IntType;
    /// Whether or not this integer is signed
    const SIGNED: bool;
    /// The size of this integer
    ///
    /// If this integer is pointer-sized (`isize`/`usize`),
    /// then this will be its runtime size on the current platform.
    const INT_SIZE: IntSize;
}
/// A primitive float type
///
/// ## Safety
/// Must actually be one of the primitive float types.
///
/// Trait info must accurately reflect the underlying type.
pub unsafe trait PrimFloat: StaticReflect + Copy + sealed::Sealed {
    /// The size of this float
    const FLOAT_SIZE: FloatSize;
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
    /// field info by name (or index).
    ///
    /// If this variant is a regular structure,
    /// this structure will have one `FieldDef` corresponding to each
    ///named field.
    /// For example,
    ///````no_run
    /// use static_reflect::types::FieldDef;
    /// struct Example {
    ///     first: u32,
    ///     second: u32
    /// }
    /// // would generate ->
    /// struct ExampleNamedFields {
    ///     first: FieldDef,
    ///     second: FieldDef
    /// }
    /// ````
    ///
    ///
    /// If the target struct is a tuple-structs, the 'named field structure' is itself a structure.
    /// For example,
    /// ````no_run
    /// # use static_reflect::types::FieldDef;
    /// struct Example(u32, String);
    /// // would generate ->
    /// struct ExampleNamedFields(FieldDef, FieldDef);
    /// ````
    type NamedFieldInfo;
    /// Static information on this structure's fields.
    ///
    /// This is a singleton value.
    const NAMED_FIELD_INFO: Self::NamedFieldInfo;
}

mod sealed {
    pub trait Sealed {}
}
