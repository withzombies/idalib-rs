// CXX bridge for type builder functions that are too complex for autocxx

#[cxx::bridge]
pub mod ffi_types {
    unsafe extern "C++" {
        include!("types_bridge.h");
        
        // Type creation functions
        fn create_struct_type(name: &str) -> u32;
        fn create_union_type(name: &str) -> u32;
        fn add_field_to_type(
            type_ordinal: u32,
            field_name: &str,
            field_type_ordinal: u32,
            offset: u64,
        ) -> bool;
        fn finalize_type(type_ordinal: u32) -> bool;
        
        // Helper functions
        fn get_primitive_type_ordinal(bt_type: u32) -> u32;
        fn get_type_size(ordinal: u32) -> u64;
        
        // Enum type functions
        fn create_enum_type(name: &str, width: u32) -> u32;
        fn add_enum_member(enum_ordinal: u32, member_name: &str, value: i64) -> bool;
        
        // Array type functions
        fn create_array_type(element_type_ordinal: u32, num_elements: u32) -> u32;
        
        // Pointer type functions
        fn create_pointer_type(target_type_ordinal: u32) -> u32;
        
        // Bitfield type functions
        fn add_bitfield_to_struct(
            struct_ordinal: u32,
            field_name: &str,
            bit_offset: u32,
            bit_width: u32,
            is_unsigned: bool,
        ) -> bool;
        
        // Function type functions
        fn create_function_type(
            return_type_ordinal: u32,
            calling_convention: u32,
            is_vararg: bool,
        ) -> u32;
        fn add_function_parameter(
            func_ordinal: u32,
            param_name: &str,
            param_type_ordinal: u32,
            is_hidden: bool,
        ) -> bool;
        fn set_function_attributes(
            func_ordinal: u32,
            is_noreturn: bool,
            is_pure: bool,
            is_static: bool,
            is_virtual: bool,
            is_const: bool,
            is_constructor: bool,
            is_destructor: bool,
        ) -> bool;
        fn create_function_pointer_type(func_type_ordinal: u32) -> u32;
    }
}