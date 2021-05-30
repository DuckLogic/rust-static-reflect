#![feature(
    never_type,
    // RFC has been accepted
    const_panic
)]

use static_reflect::{field_offset, StaticReflect, FieldReflect};
use static_reflect::types::{TypeInfo, FieldDef, StructureDef, UnionDef, UnionFieldDef, TypeId};
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
    const EXPECTED_UNION: TypeInfo<'static> = TypeInfo::Union(&UnionDef {
        name: "SimpleUnion",
        fields: &[
            SimpleUnion::NAMED_FIELD_INFO.text.erase(),
            SimpleUnion::NAMED_FIELD_INFO.b.erase(),
            SimpleUnion::NAMED_FIELD_INFO.f.erase(),
            SimpleUnion::NAMED_FIELD_INFO.nested.erase(),
        ],
        size: size_of::<SimpleUnion>(),
        alignment: align_of::<SimpleUnion>()
    });
    assert_eq!(EXPECTED_UNION, SimpleUnion::TYPE_INFO);
    assert_eq!(SimpleUnion::NAMED_FIELD_INFO.text, UnionFieldDef {
        name: "text",
        value_type: TypeId::<*mut String>::get(),
        index: 0
    });
    assert_eq!(SimpleUnion::NAMED_FIELD_INFO.b, UnionFieldDef {
        name: "b",
        value_type: TypeId::<bool>::get(),
        index: 1
    });
    assert_eq!(SimpleUnion::NAMED_FIELD_INFO.f, UnionFieldDef {
        name: "f",
        value_type: TypeId::<f32>::get(),
        index: 2
    });
    assert_eq!(SimpleUnion::NAMED_FIELD_INFO.nested, UnionFieldDef {
        name: "nested",
        value_type: TypeId::<Nested>::get(),
        index: 3
    });
}

#[test]
fn test_struct_types() {
    const NESTED_TYPE: TypeInfo<'static> = TypeInfo::Structure(&StructureDef {
        name: "Nested",
        fields: &[
            Nested::NAMED_FIELD_INFO.cycle.erase(),
            Nested::NAMED_FIELD_INFO.float.erase(),
            Nested::NAMED_FIELD_INFO.number.erase(),
        ],
        size: size_of::<Nested>(),
        alignment: align_of::<Nested>()
    });
    assert_eq!(Nested::TYPE_INFO, NESTED_TYPE);
    assert_eq!(Nested::NAMED_FIELD_INFO.cycle, FieldDef {
        name: "cycle",
        value_type: TypeId::<*mut SimpleStruct>::get(),
        offset: field_offset!(Nested, cycle),
        index: 0
    });
    assert_eq!(Nested::NAMED_FIELD_INFO.float, FieldDef {
        name: "float",
        value_type: TypeId::<f64>::get(),
        offset: field_offset!(Nested, float),
        index: 1
    });
    assert_eq!(Nested::NAMED_FIELD_INFO.number, FieldDef {
        name: "number",
        value_type: TypeId::<u64>::get(),
        offset: field_offset!(Nested, number),
        index: 2
    });
    let fields = vec![
        FieldDef {
            name: "text",
            value_type: TypeId::erased::<*mut String>(),
            offset: field_offset!(SimpleStruct, text),
            index: 0
        },
        FieldDef {
            name: "number",
            value_type: TypeId::erased::<u32>(),
            offset: field_offset!(SimpleStruct, number),
            index: 1
        },
        FieldDef {
            name: "float",
            value_type: TypeId::erased::<f64>(),
            offset: field_offset!(SimpleStruct, float),
            index: 2
        },
        FieldDef {
            name: "b",
            value_type: TypeId::erased::<bool>(),
            offset: field_offset!(SimpleStruct, b),
            index: 3
        },
        FieldDef {
            name: "unit",
            value_type: TypeId::erased::<()>(),
            offset: field_offset!(SimpleStruct, unit),
            index: 4
        },
        FieldDef {
            name: "nested_struct",
            // NOTE: We already checked Nested::STATIC_TYPE
            value_type: TypeId::erased::<Nested>(),
            offset: field_offset!(SimpleStruct, nested_struct),
            index: 5
        },
    ];
    let static_fields = leak_vec(fields);
    assert_eq!(
        SimpleStruct::TYPE_INFO,
        TypeInfo::Structure(&StructureDef {
            name: "SimpleStruct",
            fields: static_fields,
            size: size_of::<SimpleStruct>(),
            alignment: align_of::<SimpleStruct>(),
        })
    );
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
    const OPAQUE_ARRAY_TYPE: TypeInfo<'static> = TypeInfo::Structure(&StructureDef {
        name: "OpaqueArray",
        fields: &[
            OpaqueArray::NAMED_FIELD_INFO.first.erase(),
            OpaqueArray::NAMED_FIELD_INFO.array.erase(),
        ],
        size: size_of::<OpaqueArray>(),
        alignment: align_of::<OpaqueArray>()
    });
    assert_eq!(OPAQUE_ARRAY_TYPE, OpaqueArray::TYPE_INFO);
    assert_eq!(OpaqueArray::NAMED_FIELD_INFO.first, FieldDef {
        name: "first",
        value_type: TypeId::<i8>::get(), // It's actually a 'u8', but we assume_repr
        offset: field_offset!(OpaqueArray, first),
        index: 0
    });
    assert_eq!(OpaqueArray::NAMED_FIELD_INFO.array, FieldDef {
        name: "array",
        value_type: TypeId::<*mut String>::get(),
        offset: field_offset!(OpaqueArray, array),
        index: 1
    });
}
