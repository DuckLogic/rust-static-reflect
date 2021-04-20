#![feature(const_raw_ptr_to_usize_cast)]
use std::os::raw::c_void;

use duckasm_derive::duckasm_func;
use static_repr::AsmRepr;
use duckasm_repr::funcs::{FunctionDeclaration, FunctionLocation, SignatureDef};
use duckasm_repr::types::AsmType;
use std::marker::PhantomData;

#[duckasm_func]
#[export_name = "better_name"]
extern "C" fn stupid_name(first: f32, second: f32) {
    eprintln!("Test {}: {}", first, second);
}

#[no_mangle]
#[duckasm_func]
unsafe extern "C" fn dynamically_linked(first: u32, second: *mut String) -> f32 {
    eprintln!("Test {}: {}", first, &*second);
    3.14
}

#[duckasm_func(
    absolute // NOTE: This removes the requirement for #[no_mangle]
)]
extern "C" fn absolute_address_example(x: f64, y: f64) -> f64 {
    (x * x + y * y).sqrt()
}

#[duckasm_func]
extern "C" {
    /*
     * TODO: These are considered 'dead' even though DuckAsm uses them
     * Just because they're not invoked directly by Rust code,
     * doesn't mean their are actually unused.....
     */
    #[allow(dead_code)]
    #[link_name = "sqrtf"]
    fn sqrt(small: f32) -> f32;
    #[allow(dead_code)]
    fn malloc(size: usize) -> *mut c_void;
}

#[test]
fn extern_block() {
    assert_eq!(
        _FUNC_malloc,
        FunctionDeclaration::<*mut c_void, (usize,)> {
            name: "malloc",
            is_unsafe: true, // Foreign functions are always unsafe (in spite of lack of keyword)
            location: FunctionLocation::DynamicallyLinked { link_name: None },
            signature: SignatureDef {
                argument_types: &[usize::STATIC_TYPE],
                return_type: &AsmType::Pointer,
                calling_convention: Default::default()
            },
            return_type: PhantomData,
            arg_types: PhantomData
        }
    );
    assert_eq!(
        _FUNC_sqrt,
        FunctionDeclaration::<f32, (f32,)> {
            name: "sqrt",
            is_unsafe: true, // NOTE: Foreign function
            location: FunctionLocation::DynamicallyLinked { link_name: Some("sqrtf".into()) },
            signature: SignatureDef {
                argument_types: &[f32::STATIC_TYPE],
                return_type: &f32::STATIC_TYPE,
                calling_convention: Default::default()
            },
            return_type: PhantomData,
            arg_types: PhantomData
        }
    );
}

/// Tests functions defined in rust code,
/// which are exported using the C abi
#[test]
fn rust_funcs() {
    assert_eq!(
        _FUNC_dynamically_linked,
        FunctionDeclaration::<f32, (u32, *mut String)> {
            name: "dynamically_linked",
            is_unsafe: true,
            location: FunctionLocation::DynamicallyLinked { link_name: None },
            signature: SignatureDef {
                argument_types: &[u32::STATIC_TYPE, AsmType::Pointer],
                return_type: &AsmType::Float { bytes: 4 },
                calling_convention: Default::default()
            },
            return_type: PhantomData,
            arg_types: PhantomData
        }
    );
    assert_eq!(
        _FUNC_stupid_name,
        FunctionDeclaration::<(), (f32, f32)> {
            name: "stupid_name",
            is_unsafe: false,
            location: FunctionLocation::DynamicallyLinked { link_name: Some("better_name".into()) },
            signature: SignatureDef {
                argument_types: &[f32::STATIC_TYPE, AsmType::Float { bytes: 4 }],
                return_type: &AsmType::Unit,
                calling_convention: Default::default()
            },
            return_type: PhantomData,
            arg_types: PhantomData
        }
    );
    assert_eq!(
        _FUNC_absolute_address_example,
        FunctionDeclaration::<f64, (f64, f64)> {
            name: "absolute_address_example",
            is_unsafe: false,
            location: FunctionLocation::AbsoluteAddress(absolute_address_example as *const ()),
            signature: SignatureDef {
                argument_types: &[f64::STATIC_TYPE, f64::STATIC_TYPE],
                return_type: &f64::STATIC_TYPE,
                calling_convention: Default::default()
            },
            return_type: PhantomData,
            arg_types: PhantomData
        }
    );
}

