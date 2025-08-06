#pragma once

#include <cstdint>
#include <string>
#include "rust/cxx.h"
#include "typeinf.hpp"
#include "ida.hpp"
#include "idp.hpp"
#include "loader.hpp"

// Create a new struct type and return its ordinal
inline uint32_t create_struct_type(rust::Str name) {
    std::string name_str(name);
    til_t* til = get_idati();
    if (!til) return 0;
    
    // Allocate a new ordinal for this type
    uint32_t ordinal = alloc_type_ordinal(til);
    if (ordinal == 0) return 0;
    
    // Create empty struct
    tinfo_t tif;
    udt_type_data_t udt;
    udt.is_union = false;
    
    if (!tif.create_udt(udt)) {
        return 0;
    }
    
    // Save with the allocated ordinal
    if (tif.set_numbered_type(til, ordinal, NTF_TYPE) != 0) {
        return 0;
    }
    
    // Also save with name
    tif.set_named_type(til, name_str.c_str(), NTF_TYPE);
    
    return ordinal;
}

// Create a new union type and return its ordinal
inline uint32_t create_union_type(rust::Str name) {
    std::string name_str(name);
    til_t* til = get_idati();
    if (!til) return 0;
    
    uint32_t ordinal = alloc_type_ordinal(til);
    if (ordinal == 0) return 0;
    
    tinfo_t tif;
    udt_type_data_t udt;
    udt.is_union = true;
    
    if (!tif.create_udt(udt)) {
        return 0;
    }
    
    if (tif.set_numbered_type(til, ordinal, NTF_TYPE) != 0) {
        return 0;
    }
    
    tif.set_named_type(til, name_str.c_str(), NTF_TYPE);
    
    return ordinal;
}

// Add a field to an existing struct/union
inline bool add_field_to_type(
    uint32_t type_ordinal,
    rust::Str field_name,
    uint32_t field_type_ordinal,
    uint64_t offset
) {
    til_t* til = get_idati();
    if (!til) return false;
    
    // Get the struct type
    tinfo_t struct_tif;
    if (!struct_tif.get_numbered_type(til, type_ordinal)) {
        return false;
    }
    
    // Get the field type
    tinfo_t field_tif;
    if (!field_tif.get_numbered_type(til, field_type_ordinal)) {
        return false;
    }
    
    // Get existing UDT details
    udt_type_data_t udt;
    if (!struct_tif.get_udt_details(&udt)) {
        return false;
    }
    
    // Add new field
    udm_t member;
    member.name = qstring(field_name.data(), field_name.size());
    member.type = field_tif;
    member.offset = offset * 8; // Convert to bits
    member.size = field_tif.get_size() * 8;
    
    udt.push_back(member);
    
    // Recreate the type with the new field
    tinfo_t new_tif;
    if (!new_tif.create_udt(udt)) {
        return false;
    }
    
    // Update the type
    return new_tif.set_numbered_type(til, type_ordinal, NTF_REPLACE) == 0;
}

// Finalize type (ensure it's properly saved)
inline bool finalize_type(uint32_t type_ordinal) {
    til_t* til = get_idati();
    if (!til) return false;
    
    tinfo_t tif;
    if (!tif.get_numbered_type(til, type_ordinal)) {
        return false;
    }
    
    // Force synchronization
    return tif.set_numbered_type(til, type_ordinal, NTF_REPLACE) == 0;
}

// Get or create primitive type ordinal
inline uint32_t get_primitive_type_ordinal(uint32_t bt_type) {
    tinfo_t tif;
    
    // Create the primitive type
    type_t ida_type = static_cast<type_t>(bt_type);
    if (!tif.create_simple_type(ida_type)) {
        // Fallback to predefined ordinals for common types
        switch (bt_type) {
            case BTF_VOID: return 1;
            case BTF_INT8: return 2;
            case BTF_INT16: return 3;
            case BTF_INT32: return 4;
            case BTF_INT64: return 5;
            case BTF_UINT8: return 6;
            case BTF_UINT16: return 7;
            case BTF_UINT32: return 8;
            case BTF_UINT64: return 9;
            case BTF_FLOAT: return 10;
            case BTF_DOUBLE: return 11;
            default: return 0;
        }
    }
    
    til_t* til = get_idati();
    if (!til) return 0;
    
    // Try to find existing ordinal
    uint32_t limit = get_ordinal_limit(til);
    for (uint32_t i = 1; i < limit; i++) {
        tinfo_t check_tif;
        if (check_tif.get_numbered_type(til, i)) {
            if (tif.equals_to(check_tif)) {
                return i;
            }
        }
    }
    
    // Create new ordinal
    uint32_t new_ordinal = alloc_type_ordinal(til);
    if (new_ordinal == 0) return 0;
    
    if (tif.set_numbered_type(til, new_ordinal, NTF_TYPE) != 0) {
        return 0;
    }
    
    return new_ordinal;
}

// Get size of a type
inline uint64_t get_type_size(uint32_t ordinal) {
    til_t* til = get_idati();
    if (!til) return 0;
    
    tinfo_t tif;
    if (!tif.get_numbered_type(til, ordinal)) {
        return 0;
    }
    
    return tif.get_size();
}

// ============================================================================
// Enum Type Functions
// ============================================================================

// Create a new enum type and return its ordinal
inline uint32_t create_enum_type(rust::Str name, uint32_t width) {
    std::string name_str(name);
    til_t* til = get_idati();
    if (!til) return 0;
    
    // Allocate ordinal for enum
    uint32_t ordinal = alloc_type_ordinal(til);
    if (ordinal == 0) return 0;
    
    // Create enum type
    tinfo_t tif;
    enum_type_data_t etd;
    // Set width through set_nbytes method
    if (!etd.set_nbytes(width)) {
        return 0; // Invalid width
    }
    
    if (!tif.create_enum(etd)) {
        return 0;
    }
    
    // Save the enum type
    if (tif.set_numbered_type(til, ordinal, NTF_TYPE) != 0) {
        return 0;
    }
    
    // Also save with name
    tif.set_named_type(til, name_str.c_str(), NTF_TYPE);
    
    return ordinal;
}

// Add a member to an enum
inline bool add_enum_member(uint32_t enum_ordinal, rust::Str member_name, int64_t value) {
    til_t* til = get_idati();
    if (!til) return false;
    
    // Get the enum type
    tinfo_t enum_tif;
    if (!enum_tif.get_numbered_type(til, enum_ordinal)) {
        return false;
    }
    
    // Get enum details
    enum_type_data_t etd;
    if (!enum_tif.get_enum_details(&etd)) {
        return false;
    }
    
    // Add new member
    edm_t member;
    member.name = qstring(member_name.data(), member_name.size());
    member.value = value;
    
    etd.push_back(member);
    
    // Recreate enum with new member
    tinfo_t new_tif;
    if (!new_tif.create_enum(etd)) {
        return false;
    }
    
    // Update the type
    return new_tif.set_numbered_type(til, enum_ordinal, NTF_REPLACE) == 0;
}

// ============================================================================
// Array Type Functions
// ============================================================================

// Create an array type
inline uint32_t create_array_type(uint32_t element_type_ordinal, uint32_t num_elements) {
    til_t* til = get_idati();
    if (!til) return 0;
    
    // Get element type
    tinfo_t elem_tif;
    if (!elem_tif.get_numbered_type(til, element_type_ordinal)) {
        return 0;
    }
    
    // Allocate ordinal
    uint32_t ordinal = alloc_type_ordinal(til);
    if (ordinal == 0) return 0;
    
    // Create array type
    tinfo_t tif;
    array_type_data_t atd;
    atd.elem_type = elem_tif;
    atd.nelems = num_elements;
    
    if (!tif.create_array(atd)) {
        return 0;
    }
    
    // Save the array type
    if (tif.set_numbered_type(til, ordinal, NTF_TYPE) != 0) {
        return 0;
    }
    
    return ordinal;
}

// ============================================================================
// Bitfield Type Functions
// ============================================================================

// Add a bitfield member to a struct
inline bool add_bitfield_to_struct(
    uint32_t struct_ordinal,
    rust::Str field_name,
    uint32_t bit_offset,
    uint32_t bit_width,
    bool is_unsigned
) {
    til_t* til = get_idati();
    if (!til) return false;
    
    // Get the struct type
    tinfo_t struct_tif;
    if (!struct_tif.get_numbered_type(til, struct_ordinal)) {
        return false;
    }
    
    // Get existing UDT details
    udt_type_data_t udt;
    if (!struct_tif.get_udt_details(&udt)) {
        return false;
    }
    
    // Create bitfield type for the member
    // Calculate the nbytes based on the offset and width
    uint32_t end_bit = bit_offset + bit_width;
    uint32_t nbytes = 1;
    if (end_bit > 8) nbytes = 2;
    if (end_bit > 16) nbytes = 4;
    if (end_bit > 32) nbytes = 8;
    
    tinfo_t bitfield_tif;
    bitfield_type_data_t bfd(nbytes, bit_width, is_unsigned);
    if (!bitfield_tif.create_bitfield(bfd)) {
        return false;
    }
    
    // Add bitfield member
    udm_t member;
    member.name = qstring(field_name.data(), field_name.size());
    member.type = bitfield_tif;
    member.offset = bit_offset; // Offset in bits
    member.size = bit_width;    // Size in bits
    
    udt.push_back(member);
    
    // Recreate the type with the new bitfield
    tinfo_t new_tif;
    if (!new_tif.create_udt(udt)) {
        return false;
    }
    
    // Update the type
    return new_tif.set_numbered_type(til, struct_ordinal, NTF_REPLACE) == 0;
}

// ============================================================================
// Function Type Functions
// ============================================================================

// Create a function type
inline uint32_t create_function_type(
    uint32_t return_type_ordinal,
    uint32_t calling_convention,
    bool is_vararg
) {
    til_t* til = get_idati();
    if (!til) return 0;
    
    // Allocate ordinal
    uint32_t ordinal = alloc_type_ordinal(til);
    if (ordinal == 0) return 0;
    
    // Create function type
    func_type_data_t ftd;
    
    // Set return type
    if (return_type_ordinal != 0) {
        tinfo_t ret_tif;
        if (!ret_tif.get_numbered_type(til, return_type_ordinal)) {
            return 0;
        }
        ftd.rettype = ret_tif;
    }
    
    // Set calling convention
#if IDA_SDK_VERSION >= 920
    ftd.set_cc(calling_convention);
    
    // Set vararg flag
    if (is_vararg) {
        ftd.set_cc(ftd.get_cc() | CM_CC_ELLIPSIS);
    }
#else
    ftd.cc = calling_convention;
    
    // Set vararg flag
    if (is_vararg) {
        ftd.cc = ftd.cc | CM_CC_ELLIPSIS;
    }
#endif
    
    // Create the function type
    tinfo_t tif;
    if (!tif.create_func(ftd)) {
        return 0;
    }
    
    // Save the function type
    if (tif.set_numbered_type(til, ordinal, NTF_TYPE) != 0) {
        return 0;
    }
    
    return ordinal;
}

// Add a parameter to a function type
inline bool add_function_parameter(
    uint32_t func_ordinal,
    rust::Str param_name,
    uint32_t param_type_ordinal,
    bool is_hidden
) {
    til_t* til = get_idati();
    if (!til) return false;
    
    // Get the function type
    tinfo_t func_tif;
    if (!func_tif.get_numbered_type(til, func_ordinal)) {
        return false;
    }
    
    // Get function details
    func_type_data_t ftd;
    if (!func_tif.get_func_details(&ftd)) {
        return false;
    }
    
    // Get parameter type
    tinfo_t param_tif;
    if (!param_tif.get_numbered_type(til, param_type_ordinal)) {
        return false;
    }
    
    // Create new parameter
    funcarg_t arg;
    arg.name = qstring(param_name.data(), param_name.size());
    arg.type = param_tif;
    if (is_hidden) {
        arg.flags |= FAI_HIDDEN;
    }
    
    // Add parameter to function
    ftd.push_back(arg);
    
    // Recreate function type with new parameter
    tinfo_t new_tif;
    if (!new_tif.create_func(ftd)) {
        return false;
    }
    
    // Update the type
    return new_tif.set_numbered_type(til, func_ordinal, NTF_REPLACE) == 0;
}

// Set function attributes
inline bool set_function_attributes(
    uint32_t func_ordinal,
    bool is_noreturn,
    bool is_pure,
    bool is_static,
    bool is_virtual,
    bool is_const,
    bool is_constructor,
    bool is_destructor
) {
    til_t* til = get_idati();
    if (!til) return false;
    
    // Get the function type
    tinfo_t func_tif;
    if (!func_tif.get_numbered_type(til, func_ordinal)) {
        return false;
    }
    
    // Get function details
    func_type_data_t ftd;
    if (!func_tif.get_func_details(&ftd)) {
        return false;
    }
    
    // Set attributes
    if (is_noreturn) ftd.flags |= FTI_NORET;
    if (is_pure) ftd.flags |= FTI_PURE;
    if (is_static) ftd.flags |= FTI_STATIC;
    if (is_virtual) ftd.flags |= FTI_VIRTUAL;
    if (is_const) ftd.flags |= FTI_CONST;
    if (is_constructor) ftd.flags |= FTI_CTOR;
    if (is_destructor) ftd.flags |= FTI_DTOR;
    
    // Recreate function type with new attributes
    tinfo_t new_tif;
    if (!new_tif.create_func(ftd)) {
        return false;
    }
    
    // Update the type
    return new_tif.set_numbered_type(til, func_ordinal, NTF_REPLACE) == 0;
}

// Create a function pointer type
inline uint32_t create_function_pointer_type(uint32_t func_type_ordinal) {
    til_t* til = get_idati();
    if (!til) return 0;
    
    // Get the function type
    tinfo_t func_tif;
    if (!func_tif.get_numbered_type(til, func_type_ordinal)) {
        return 0;
    }
    
    // Allocate ordinal for the pointer
    uint32_t ordinal = alloc_type_ordinal(til);
    if (ordinal == 0) return 0;
    
    // Create pointer to function
    tinfo_t ptr_tif;
    ptr_type_data_t ptd;
    ptd.taptr_bits = 0;
    ptd.obj_type = func_tif;
    
    if (!ptr_tif.create_ptr(ptd)) {
        return 0;
    }
    
    // Save the function pointer type
    if (ptr_tif.set_numbered_type(til, ordinal, NTF_TYPE) != 0) {
        return 0;
    }
    
    return ordinal;
}

// ============================================================================
// Pointer Type Functions
// ============================================================================

// Create a pointer type
inline uint32_t create_pointer_type(uint32_t target_type_ordinal) {
    til_t* til = get_idati();
    if (!til) return 0;
    
    // Get target type
    tinfo_t target_tif;
    if (!target_tif.get_numbered_type(til, target_type_ordinal)) {
        return 0;
    }
    
    // Allocate ordinal
    uint32_t ordinal = alloc_type_ordinal(til);
    if (ordinal == 0) return 0;
    
    // Create pointer type
    tinfo_t tif;
    ptr_type_data_t ptd;
    ptd.taptr_bits = 0; // Standard pointer (no special attributes)
    ptd.obj_type = target_tif;
    
    if (!tif.create_ptr(ptd)) {
        return 0;
    }
    
    // Save the pointer type
    if (tif.set_numbered_type(til, ordinal, NTF_TYPE) != 0) {
        return 0;
    }
    
    return ordinal;
}