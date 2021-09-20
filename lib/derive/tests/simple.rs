#![feature(
    never_type,
    // RFC has been accepted
    const_panic
)]
use pretty_assertions::assert_eq;

use zerogc::epsilon::{gc, leaked, gc_array, gc_str};
use static_reflect::{field_offset, StaticReflect, FieldReflect};
use static_reflect::types::{TypeInfo, FieldDef, StructureDef, UntaggedUnionDef, UnionFieldDef, TypeId, CStyleEnumVariant, CStyleEnumDef, DiscriminantValue, IntType};
use std::mem::{size_of, align_of};

use static_reflect_derive::{StaticReflect};

#[derive(Copy, Clone, Debug, PartialEq, StaticReflect)]
#[repr(C)]
pub struct Nested {
    cycle: *mut SimpleStruct,
    float: f64,
    number: u64,
}

#[derive(StaticReflect)]
#[repr(C)]
pub struct SimpleStruct {
    // We can have pointers to anything
    text: *mut String,
    number: u32,
    float: f64,
    b: bool,
    unit: (),
    nested_struct: Nested
}

fn leak_vec<T>(elements: Vec<T>) -> &'static [T] {
    let ptr = elements.as_ptr();
    let len = elements.len();
    std::mem::forget(elements);
    unsafe { std::slice::from_raw_parts(ptr, len) }
}

#[repr(C)]
#[derive(StaticReflect)]
union SimpleUnion {
    pub text: *mut String,
    b: bool,
    f: f32,
    nested: Nested
}

#[test]
fn test_union_types() {
    let expected_union: TypeInfo<'static> = TypeInfo::UntaggedUnion(leaked(UntaggedUnionDef {
        name: gc_str("SimpleUnion"),
        fields: gc_array(vec![
            SimpleUnion::NAMED_FIELD_INFO.text.erase(),
            SimpleUnion::NAMED_FIELD_INFO.b.erase(),
            SimpleUnion::NAMED_FIELD_INFO.f.erase(),
            SimpleUnion::NAMED_FIELD_INFO.nested.erase(),
        ].leak()),
        size: size_of::<SimpleUnion>(),
        alignment: align_of::<SimpleUnion>()
    }));
    assert_eq!(expected_union, SimpleUnion::TYPE_INFO);
    assert_eq!(SimpleUnion::NAMED_FIELD_INFO.text, UnionFieldDef {
        name: gc_str("text"),
        value_type: TypeId::<*mut String>::get(),
        index: 0
    });
    assert_eq!(SimpleUnion::NAMED_FIELD_INFO.b, UnionFieldDef {
        name: gc_str("b"),
        value_type: TypeId::<bool>::get(),
        index: 1
    });
    assert_eq!(SimpleUnion::NAMED_FIELD_INFO.f, UnionFieldDef {
        name: gc_str("f"),
        value_type: TypeId::<f32>::get(),
        index: 2
    });
    assert_eq!(SimpleUnion::NAMED_FIELD_INFO.nested, UnionFieldDef {
        name: gc_str("nested"),
        value_type: TypeId::<Nested>::get(),
        index: 3
    });
}

#[test]
fn test_struct_types() {
    let nested_type: TypeInfo<'static> = TypeInfo::Structure(leaked(StructureDef {
        name: gc_str("Nested"),
        fields: gc_array(vec![
            Nested::NAMED_FIELD_INFO.cycle.erase(),
            Nested::NAMED_FIELD_INFO.float.erase(),
            Nested::NAMED_FIELD_INFO.number.erase(),
        ].leak()),
        size: size_of::<Nested>(),
        alignment: align_of::<Nested>()
    }));
    assert_eq!(Nested::TYPE_INFO, nested_type);
    assert_eq!(Nested::NAMED_FIELD_INFO.cycle, FieldDef {
        name: Some(gc_str("cycle")),
        value_type: TypeId::<*mut SimpleStruct>::get(),
        offset: field_offset!(Nested, cycle),
        index: 0
    });
    assert_eq!(Nested::NAMED_FIELD_INFO.float, FieldDef {
        name: Some(gc_str("float")),
        value_type: TypeId::<f64>::get(),
        offset: field_offset!(Nested, float),
        index: 1
    });
    assert_eq!(Nested::NAMED_FIELD_INFO.number, FieldDef {
        name: Some(gc_str("number")),
        value_type: TypeId::<u64>::get(),
        offset: field_offset!(Nested, number),
        index: 2
    });
    let fields = vec![
        FieldDef {
            name: Some(gc_str("text")),
            value_type: TypeId::erased::<*mut String>(),
            offset: field_offset!(SimpleStruct, text),
            index: 0
        },
        FieldDef {
            name: Some(gc_str("number")),
            value_type: TypeId::erased::<u32>(),
            offset: field_offset!(SimpleStruct, number),
            index: 1
        },
        FieldDef {
            name: Some(gc_str("float")),
            value_type: TypeId::erased::<f64>(),
            offset: field_offset!(SimpleStruct, float),
            index: 2
        },
        FieldDef {
            name: Some(gc_str("b")),
            value_type: TypeId::erased::<bool>(),
            offset: field_offset!(SimpleStruct, b),
            index: 3
        },
        FieldDef {
            name: Some(gc_str("unit")),
            value_type: TypeId::erased::<()>(),
            offset: field_offset!(SimpleStruct, unit),
            index: 4
        },
        FieldDef {
            name: Some(gc_str("nested_struct")),
            // NOTE: We already checked Nested::STATIC_TYPE
            value_type: TypeId::erased::<Nested>(),
            offset: field_offset!(SimpleStruct, nested_struct),
            index: 5
        },
    ];
    let static_fields = leak_vec(fields);
    assert_eq!(
        SimpleStruct::TYPE_INFO,
        TypeInfo::Structure(leaked(StructureDef {
            name: gc_str("SimpleStruct"),
            fields: gc_array(static_fields),
            size: size_of::<SimpleStruct>(),
            alignment: align_of::<SimpleStruct>(),
        }))
    );
}


#[derive(StaticReflect)]
#[repr(C)]
struct SimpleTupleStruct(*mut String, f32, Nested);

#[test]
fn test_tuple_struct() {
    let fields = vec![
        FieldDef {
            name: None,
            value_type: TypeId::erased::<*mut String>(),
            offset: field_offset!(SimpleTupleStruct, 0),
            index: 0
        },
        FieldDef {
            name: None,
            value_type: TypeId::erased::<f32>(),
            offset: field_offset!(SimpleTupleStruct, 1),
            index: 1
        },
        FieldDef {
            name: None,
            // NOTE: We already checked Nested::STATIC_TYPE
            value_type: TypeId::erased::<Nested>(),
            offset: field_offset!(SimpleTupleStruct, 2),
            index: 2
        },
    ];
    assert_eq!(SimpleTupleStruct::NAMED_FIELD_INFO.0.erase(), fields[0]);
    assert_eq!(SimpleTupleStruct::NAMED_FIELD_INFO.1.erase(), fields[1]);
    assert_eq!(SimpleTupleStruct::NAMED_FIELD_INFO.2.erase(), fields[2]);
    let static_fields = leak_vec(fields);
    assert_eq!(
        SimpleTupleStruct::TYPE_INFO,
        TypeInfo::Structure(gc(Box::leak(Box::new(StructureDef {
            name: gc_str("SimpleTupleStruct"),
            fields: gc_array(static_fields),
            size: size_of::<SimpleTupleStruct>(),
            alignment: align_of::<SimpleTupleStruct>(),
        }))))
    );
}

#[derive(StaticReflect)]
#[repr(C)]
#[allow(dead_code)]
enum SimpleEnum {
    Zero,
    Two = 2,
    Eight = 8,
    Four = 4,
    Implicit
}

#[test] // Currently fails because of #2
fn test_simple_enum(){
    assert_eq!(
        SimpleEnum::TYPE_INFO,
        TypeInfo::CStyleEnum(gc(Box::leak(Box::new(CStyleEnumDef {
            name: gc_str("SimpleEnum"),
            discriminant: IntType::ISIZE,
            variants: gc_array(Vec::leak(vec![
                CStyleEnumVariant {
                    index: 0,
                    name: gc_str("Zero"),
                    discriminant: DiscriminantValue::Default {
                        declaration_index: 0,
                    },
                },
                CStyleEnumVariant {
                    index: 1,
                    name: gc_str("Two"),
                    discriminant: DiscriminantValue::ExplicitInteger {
                        bits: 2,
                    },
                },
                CStyleEnumVariant {
                    index: 2,
                    name: gc_str("Eight"),
                    discriminant: DiscriminantValue::ExplicitInteger {
                        bits: 8,
                    },
                },
                CStyleEnumVariant {
                    index: 3,
                    name: gc_str("Four"),
                    discriminant: DiscriminantValue::ExplicitInteger {
                        bits: 4,
                    },
                },
                CStyleEnumVariant {
                    index: 4,
                    name: gc_str("Implciit"),
                    discriminant: DiscriminantValue::ImplicitlyOffset {
                        bits: 5,
                    },
                }
            ]))
        }))))
    )
}

#[derive(StaticReflect)]
#[repr(C)]
struct OpaqueArray {
    #[reflect(assume_repr = "i8")]
    first: u8,
    #[static_reflect(opaque_array)]
    array: [*mut String; 42]
}

#[test]
fn test_options() {
    let opaque_array_type: TypeInfo<'static> = TypeInfo::Structure(leaked(StructureDef {
        name: gc_str("OpaqueArray"),
        fields: gc_array(vec![
            OpaqueArray::NAMED_FIELD_INFO.first.erase(),
            OpaqueArray::NAMED_FIELD_INFO.array.erase(),
        ].leak()),
        size: size_of::<OpaqueArray>(),
        alignment: align_of::<OpaqueArray>()
    }));
    assert_eq!(opaque_array_type, OpaqueArray::TYPE_INFO);
    assert_eq!(OpaqueArray::NAMED_FIELD_INFO.first, FieldDef {
        name: Some(gc_str("first")),
        value_type: TypeId::<i8>::get(), // It's actually a 'u8', but we assume_repr
        offset: field_offset!(OpaqueArray, first),
        index: 0
    });
    assert_eq!(OpaqueArray::NAMED_FIELD_INFO.array, FieldDef {
        name: Some(gc_str("array")),
        value_type: TypeId::<*mut String>::get(),
        offset: field_offset!(OpaqueArray, array),
        index: 1
    });
}
