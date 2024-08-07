use std::marker::PhantomData;
use std::os::raw::c_void;

use static_reflect::funcs::{FunctionDeclaration, FunctionLocation, SignatureDef};
use static_reflect::types::{FloatSize, TypeInfo};
use static_reflect::{reflect_func, StaticReflect};

#[reflect_func]
#[export_name = "better_name"]
extern "C" fn stupid_name(first: f32, second: f32) {
    eprintln!("Test {}: {}", first, second);
}

#[no_mangle]
#[reflect_func]
unsafe extern "C" fn dynamically_linked(first: u32, second: *mut String) -> f32 {
    eprintln!("Test {}: {}", first, &*second);
    #[allow(clippy::approx_constant)]
    3.14
}

#[reflect_func(
    absolute // NOTE: This removes the requirement for #[no_mangle]
)]
extern "C" fn absolute_address_example(x: f64, y: f64) -> f64 {
    (x * x + y * y).sqrt()
}

#[reflect_func]
extern "C" {
    /*
     * TODO: These are considered 'dead' even though DuckAsm uses them
     * Just because they're not invoked directly by Rust code,
     * doesn't mean they are actually unused.....
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
            location: Some(FunctionLocation::DynamicallyLinked { link_name: None }),
            signature: SignatureDef {
                argument_types: &[usize::TYPE_INFO],
                return_type: &TypeInfo::Pointer,
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
            location: Some(FunctionLocation::DynamicallyLinked {
                link_name: Some("sqrtf")
            }),
            signature: SignatureDef {
                argument_types: &[f32::TYPE_INFO],
                return_type: &f32::TYPE_INFO,
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
            location: Some(FunctionLocation::DynamicallyLinked { link_name: None }),
            signature: SignatureDef {
                argument_types: &[u32::TYPE_INFO, TypeInfo::Pointer],
                return_type: &TypeInfo::F32,
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
            location: Some(FunctionLocation::DynamicallyLinked {
                link_name: Some("better_name")
            }),
            signature: SignatureDef {
                argument_types: &[
                    f32::TYPE_INFO,
                    TypeInfo::Float {
                        size: FloatSize::Single
                    }
                ],
                return_type: &TypeInfo::Unit,
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
            location: Some(FunctionLocation::AbsoluteAddress(
                absolute_address_example as *const ()
            )),
            signature: SignatureDef {
                argument_types: &[f64::TYPE_INFO, f64::TYPE_INFO],
                return_type: &f64::TYPE_INFO,
                calling_convention: Default::default()
            },
            return_type: PhantomData,
            arg_types: PhantomData
        }
    );
}
