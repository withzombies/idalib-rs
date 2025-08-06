use crate::ffi::types::{
    create_struct_type, create_union_type, add_field_to_type,
    finalize_type, get_primitive_type_ordinal, get_type_size,
    create_enum_type, add_enum_member,
    create_array_type, create_pointer_type,
    add_bitfield_to_struct,
    create_function_type, add_function_parameter,
    set_function_attributes, create_function_pointer_type,
};
use crate::types::Type;
use crate::IDAError;

/// Trait for all type builders
pub trait TypeBuilder: Sized {
    /// Build the type and save it to the type library
    fn build(self) -> Result<Type, IDAError>;
    
    /// Validate the builder configuration before building
    fn validate(&self) -> Result<(), IDAError> {
        Ok(())
    }
}

/// Trait for type validation
pub trait TypeValidator {
    /// Validate the type configuration
    fn validate(&self) -> Result<(), IDAError>;
}

/// Builder for creating struct types
#[derive(Debug)]
pub struct StructBuilder {
    name: String,
    fields: Vec<StructField>,
    bitfields: Vec<BitfieldInfo>,
    is_union: bool,
}

#[derive(Debug)]
struct StructField {
    name: String,
    field_type: FieldType,
    offset: Option<u64>,
}

#[derive(Debug)]
struct BitfieldInfo {
    name: String,
    bit_offset: u32,
    bit_width: u32,
    is_unsigned: bool,
}

/// Represents a field type in a struct/union
#[derive(Debug, Clone)]
pub enum FieldType {
    /// A primitive type (int, float, etc.)
    Primitive(PrimitiveType),
    /// Reference to an existing type
    Existing(Type),
    /// Forward reference to a type being built (for self-referential types)
    /// The string is the name of the type being referenced
    ForwardRef(String),
}

/// Primitive types available in IDA
#[derive(Debug, Clone, Copy)]
pub enum PrimitiveType {
    Void,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float,
    Double,
    Char,
    Bool,
}

impl PrimitiveType {
    /// Get the IDA basic type code
    fn to_ida_type(self) -> u32 {
        match self {
            PrimitiveType::Void => 0x00,    // BT_VOID
            PrimitiveType::Int8 => 0x01,    // BT_INT8
            PrimitiveType::Int16 => 0x02,   // BT_INT16
            PrimitiveType::Int32 => 0x03,   // BT_INT32
            PrimitiveType::Int64 => 0x04,   // BT_INT64
            PrimitiveType::UInt8 => 0x05,   // BT_INT8 | BTMT_UNSIGNED
            PrimitiveType::UInt16 => 0x06,  // BT_INT16 | BTMT_UNSIGNED
            PrimitiveType::UInt32 => 0x07,  // BT_INT32 | BTMT_UNSIGNED
            PrimitiveType::UInt64 => 0x08,  // BT_INT64 | BTMT_UNSIGNED
            PrimitiveType::Bool => 0x08,    // BT_BOOL
            PrimitiveType::Float => 0x09,   // BT_FLOAT
            PrimitiveType::Double => 0x0A,  // BT_DOUBLE
            PrimitiveType::Char => 0x01,    // BT_INT8 (char is typically signed byte)
        }
    }

    /// Create a Type from this primitive
    pub fn to_type(self) -> Result<Type, IDAError> {
        let ordinal = get_primitive_type_ordinal(self.to_ida_type());
        if ordinal == 0 {
            return Err(IDAError::ffi_with("Failed to create primitive type"));
        }
        Ok(Type::from_ordinal(ordinal))
    }
}

impl StructBuilder {
    /// Create a new struct builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fields: Vec::new(),
            bitfields: Vec::new(),
            is_union: false,
        }
    }

    /// Create a new union builder
    pub fn new_union(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fields: Vec::new(),
            bitfields: Vec::new(),
            is_union: true,
        }
    }

    /// Add a field to the struct
    pub fn field(mut self, name: impl Into<String>, field_type: impl Into<FieldType>) -> Self {
        self.fields.push(StructField {
            name: name.into(),
            field_type: field_type.into(),
            offset: None,
        });
        self
    }

    /// Add a field with explicit offset (for structs only)
    pub fn field_at(
        mut self,
        name: impl Into<String>,
        field_type: impl Into<FieldType>,
        offset: u64,
    ) -> Self {
        if self.is_union {
            // Unions don't use explicit offsets
            return self.field(name, field_type);
        }
        self.fields.push(StructField {
            name: name.into(),
            field_type: field_type.into(),
            offset: Some(offset),
        });
        self
    }

    /// Set whether this is a union
    pub fn is_union(mut self, is_union: bool) -> Self {
        self.is_union = is_union;
        self
    }
    
    /// Add a bitfield to the struct
    pub fn bitfield(
        mut self,
        name: impl Into<String>,
        bit_offset: u32,
        bit_width: u32,
        is_unsigned: bool,
    ) -> Self {
        if self.is_union {
            // Unions don't support bitfields in the same way
            return self;
        }
        self.bitfields.push(BitfieldInfo {
            name: name.into(),
            bit_offset,
            bit_width,
            is_unsigned,
        });
        self
    }
    
    /// Add an unsigned bitfield (convenience method)
    pub fn unsigned_bitfield(
        self,
        name: impl Into<String>,
        bit_offset: u32,
        bit_width: u32,
    ) -> Self {
        self.bitfield(name, bit_offset, bit_width, true)
    }
    
    /// Add a signed bitfield (convenience method)
    pub fn signed_bitfield(
        self,
        name: impl Into<String>,
        bit_offset: u32,
        bit_width: u32,
    ) -> Self {
        self.bitfield(name, bit_offset, bit_width, false)
    }
    
    /// Add a self-referential field (pointer to this struct)
    /// Useful for linked lists, trees, etc.
    pub fn self_ref(self, name: impl Into<String>) -> Self {
        let struct_name = self.name.clone();
        self.field(name, FieldType::ForwardRef(struct_name))
    }
}

impl TypeValidator for StructBuilder {
    fn validate(&self) -> Result<(), IDAError> {
        // Check for empty name
        if self.name.is_empty() {
            return Err(IDAError::ffi_with("Struct/union name cannot be empty"));
        }
        
        // Check for duplicate field names
        let mut field_names = std::collections::HashSet::new();
        for field in &self.fields {
            if !field_names.insert(&field.name) {
                return Err(IDAError::ffi_with(format!(
                    "Duplicate field name '{}' in {}",
                    field.name, self.name
                )));
            }
        }
        
        // Check for duplicate bitfield names
        for bitfield in &self.bitfields {
            if !field_names.insert(&bitfield.name) {
                return Err(IDAError::ffi_with(format!(
                    "Duplicate bitfield name '{}' in {}",
                    bitfield.name, self.name
                )));
            }
        }
        
        // Validate bitfield positions don't overlap
        let mut bit_ranges: Vec<(u32, u32)> = Vec::new();
        for bitfield in &self.bitfields {
            let start = bitfield.bit_offset;
            let end = bitfield.bit_offset + bitfield.bit_width;
            
            // Check for overlaps
            for (existing_start, existing_end) in &bit_ranges {
                if (start >= *existing_start && start < *existing_end) || 
                   (end > *existing_start && end <= *existing_end) ||
                   (start <= *existing_start && end >= *existing_end) {
                    return Err(IDAError::ffi_with(format!(
                        "Bitfield '{}' overlaps with another bitfield (bits {}-{})",
                        bitfield.name, start, end
                    )));
                }
            }
            
            bit_ranges.push((start, end));
        }
        
        Ok(())
    }
}

impl TypeBuilder for StructBuilder {
    fn build(self) -> Result<Type, IDAError> {
        // Validate before building
        TypeValidator::validate(&self)?;
        // Create the empty struct/union
        let struct_ordinal = if self.is_union {
            create_union_type(&self.name)
        } else {
            create_struct_type(&self.name)
        };

        if struct_ordinal == 0 {
            return Err(IDAError::ffi_with(format!(
                "Failed to create {} '{}'",
                if self.is_union { "union" } else { "struct" },
                self.name
            )));
        }

        // Add fields
        let mut current_offset = 0u64;
        for field in self.fields {
            // Get the field type ordinal
            let field_type_ordinal = match field.field_type {
                FieldType::Primitive(prim) => {
                    get_primitive_type_ordinal(prim.to_ida_type())
                }
                FieldType::Existing(typ) => typ.ordinal(),
                FieldType::ForwardRef(ref name) => {
                    // For forward references, we need to create a pointer to the struct being built
                    // This allows self-referential structures like linked lists
                    if name == &self.name {
                        // Self-reference - create a pointer to this struct
                        create_pointer_type(struct_ordinal)
                    } else {
                        // Forward reference to another type - this would need a type registry
                        // For now, we'll return an error
                        return Err(IDAError::ffi_with(format!(
                            "Forward reference to '{}' not yet supported (only self-references allowed)",
                            name
                        )));
                    }
                }
            };

            if field_type_ordinal == 0 {
                return Err(IDAError::ffi_with(format!(
                    "Invalid field type for field '{}'",
                    field.name
                )));
            }

            let offset = field.offset.unwrap_or(current_offset);
            
            let success = add_field_to_type(
                struct_ordinal,
                &field.name,
                field_type_ordinal,
                offset,
            );

            if !success {
                return Err(IDAError::ffi_with(format!(
                    "Failed to add field '{}' to {}",
                    field.name,
                    self.name
                )));
            }

            // Update offset for next field (only for structs, not unions)
            if !self.is_union && field.offset.is_none() {
                let field_size = get_type_size(field_type_ordinal);
                current_offset += if field_size > 0 { field_size } else { 8 };
            }
        }

        // Add bitfields
        for bitfield in self.bitfields {
            let success = add_bitfield_to_struct(
                struct_ordinal,
                &bitfield.name,
                bitfield.bit_offset,
                bitfield.bit_width,
                bitfield.is_unsigned,
            );

            if !success {
                return Err(IDAError::ffi_with(format!(
                    "Failed to add bitfield '{}' to {}",
                    bitfield.name,
                    self.name
                )));
            }
        }

        // Finalize the type
        if !finalize_type(struct_ordinal) {
            return Err(IDAError::ffi_with("Failed to finalize type"));
        }

        Ok(Type::from_ordinal(struct_ordinal))
    }
}

// Implement From traits for convenient field type creation
impl From<PrimitiveType> for FieldType {
    fn from(prim: PrimitiveType) -> Self {
        FieldType::Primitive(prim)
    }
}

impl From<Type> for FieldType {
    fn from(typ: Type) -> Self {
        FieldType::Existing(typ)
    }
}


// Clone implementation for StructBuilder
impl Clone for StructBuilder {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            fields: self.fields.iter().map(|f| StructField {
                name: f.name.clone(),
                field_type: match &f.field_type {
                    FieldType::Primitive(p) => FieldType::Primitive(*p),
                    FieldType::Existing(t) => FieldType::Existing(t.clone()),
                    FieldType::ForwardRef(s) => FieldType::ForwardRef(s.clone()),
                },
                offset: f.offset,
            }).collect(),
            bitfields: self.bitfields.iter().map(|b| BitfieldInfo {
                name: b.name.clone(),
                bit_offset: b.bit_offset,
                bit_width: b.bit_width,
                is_unsigned: b.is_unsigned,
            }).collect(),
            is_union: self.is_union,
        }
    }
}

// We need to implement Clone for Type
impl Clone for Type {
    fn clone(&self) -> Self {
        Self::from_ordinal(self.ordinal())
    }
}

/// Builder for creating enum types
#[derive(Debug, Clone)]
pub struct EnumBuilder {
    name: String,
    width: u32,
    members: Vec<EnumMember>,
}

#[derive(Debug, Clone)]
struct EnumMember {
    name: String,
    value: i64,
}

impl EnumBuilder {
    /// Create a new enum builder with specified width in bytes (1, 2, 4, or 8)
    pub fn new(name: impl Into<String>, width: u32) -> Self {
        Self {
            name: name.into(),
            width,
            members: Vec::new(),
        }
    }

    /// Add a member to the enum with an explicit value
    pub fn member(mut self, name: impl Into<String>, value: i64) -> Self {
        self.members.push(EnumMember {
            name: name.into(),
            value,
        });
        self
    }

    /// Add a member with auto-incremented value
    pub fn auto_member(mut self, name: impl Into<String>) -> Self {
        let next_value = if let Some(last) = self.members.last() {
            last.value + 1
        } else {
            0
        };
        self.members.push(EnumMember {
            name: name.into(),
            value: next_value,
        });
        self
    }
}

impl TypeValidator for EnumBuilder {
    fn validate(&self) -> Result<(), IDAError> {
        // Check for empty name
        if self.name.is_empty() {
            return Err(IDAError::ffi_with("Enum name cannot be empty"));
        }
        
        // Validate width
        if ![1, 2, 4, 8].contains(&self.width) {
            return Err(IDAError::ffi_with(format!(
                "Invalid enum width {}. Must be 1, 2, 4, or 8",
                self.width
            )));
        }
        
        // Check for duplicate member names
        let mut member_names = std::collections::HashSet::new();
        for member in &self.members {
            if !member_names.insert(&member.name) {
                return Err(IDAError::ffi_with(format!(
                    "Duplicate enum member name '{}' in {}",
                    member.name, self.name
                )));
            }
        }
        
        Ok(())
    }
}

impl TypeBuilder for EnumBuilder {
    fn build(self) -> Result<Type, IDAError> {
        // Validate before building
        TypeValidator::validate(&self)?;

        // Create the enum
        let enum_ordinal = create_enum_type(&self.name, self.width);
        if enum_ordinal == 0 {
            return Err(IDAError::ffi_with(format!(
                "Failed to create enum '{}'",
                self.name
            )));
        }

        // Add members
        for member in self.members {
            if !add_enum_member(enum_ordinal, &member.name, member.value) {
                return Err(IDAError::ffi_with(format!(
                    "Failed to add member '{}' to enum '{}'",
                    member.name, self.name
                )));
            }
        }

        // Finalize the type
        if !finalize_type(enum_ordinal) {
            return Err(IDAError::ffi_with("Failed to finalize enum type"));
        }

        Ok(Type::from_ordinal(enum_ordinal))
    }
}

/// Builder for creating array types
#[derive(Debug, Clone)]
pub struct ArrayBuilder {
    element_type: FieldType,
    num_elements: u32,
}

impl ArrayBuilder {
    /// Create a new array builder
    pub fn new(element_type: impl Into<FieldType>, num_elements: u32) -> Self {
        Self {
            element_type: element_type.into(),
            num_elements,
        }
    }
}

impl TypeBuilder for ArrayBuilder {
    fn build(self) -> Result<Type, IDAError> {
        // Get the element type ordinal
        let element_ordinal = match self.element_type {
            FieldType::Primitive(prim) => get_primitive_type_ordinal(prim.to_ida_type()),
            FieldType::Existing(typ) => typ.ordinal(),
            FieldType::ForwardRef(_) => {
                return Err(IDAError::ffi_with(
                    "Forward references not supported in array element types"
                ));
            }
        };

        if element_ordinal == 0 {
            return Err(IDAError::ffi_with("Invalid element type for array"));
        }

        // Create the array type
        let array_ordinal = create_array_type(element_ordinal, self.num_elements);
        if array_ordinal == 0 {
            return Err(IDAError::ffi_with("Failed to create array type"));
        }

        Ok(Type::from_ordinal(array_ordinal))
    }
}

/// Builder for creating pointer types
#[derive(Debug, Clone)]
pub struct PointerBuilder {
    target_type: FieldType,
}

impl PointerBuilder {
    /// Create a new pointer builder
    pub fn new(target_type: impl Into<FieldType>) -> Self {
        Self {
            target_type: target_type.into(),
        }
    }
}

impl TypeBuilder for PointerBuilder {
    fn build(self) -> Result<Type, IDAError> {
        // Get the target type ordinal
        let target_ordinal = match self.target_type {
            FieldType::Primitive(prim) => get_primitive_type_ordinal(prim.to_ida_type()),
            FieldType::Existing(typ) => typ.ordinal(),
            FieldType::ForwardRef(_) => {
                return Err(IDAError::ffi_with(
                    "Forward references not supported in pointer target types"
                ));
            }
        };

        if target_ordinal == 0 {
            return Err(IDAError::ffi_with("Invalid target type for pointer"));
        }

        // Create the pointer type
        let pointer_ordinal = create_pointer_type(target_ordinal);
        if pointer_ordinal == 0 {
            return Err(IDAError::ffi_with("Failed to create pointer type"));
        }

        Ok(Type::from_ordinal(pointer_ordinal))
    }
}

/// Builder for creating function types
#[derive(Debug, Clone)]
pub struct FunctionBuilder {
    return_type: Option<FieldType>,
    parameters: Vec<FunctionParameter>,
    calling_convention: CallingConvention,
    is_vararg: bool,
    attributes: FunctionAttributes,
}

#[derive(Debug, Clone)]
struct FunctionParameter {
    name: String,
    param_type: FieldType,
    is_hidden: bool,
}

#[derive(Debug, Clone, Default)]
struct FunctionAttributes {
    is_noreturn: bool,
    is_pure: bool,
    is_static: bool,
    is_virtual: bool,
    is_const: bool,
    is_constructor: bool,
    is_destructor: bool,
}

/// Calling conventions
#[derive(Debug, Clone, Copy)]
pub enum CallingConvention {
    Unknown,
    Cdecl,
    Stdcall,
    Pascal,
    Fastcall,
    Thiscall,
    Swift,
    Golang,
    Custom(u32),
}

impl CallingConvention {
    fn to_ida_cc(self) -> u32 {
        match self {
            CallingConvention::Unknown => 0x10,   // CM_CC_UNKNOWN
            CallingConvention::Cdecl => 0x30,     // CM_CC_CDECL
            CallingConvention::Stdcall => 0x50,   // CM_CC_STDCALL
            CallingConvention::Pascal => 0x60,    // CM_CC_PASCAL
            CallingConvention::Fastcall => 0x70,  // CM_CC_FASTCALL
            CallingConvention::Thiscall => 0x80,  // CM_CC_THISCALL
            CallingConvention::Swift => 0x90,     // CM_CC_SWIFT
            CallingConvention::Golang => 0xB0,    // CM_CC_GOLANG
            CallingConvention::Custom(cc) => cc,
        }
    }
}

impl FunctionBuilder {
    /// Create a new function builder
    pub fn new() -> Self {
        Self {
            return_type: None,
            parameters: Vec::new(),
            calling_convention: CallingConvention::Unknown,
            is_vararg: false,
            attributes: FunctionAttributes::default(),
        }
    }

    /// Set the return type
    pub fn returns(mut self, return_type: impl Into<FieldType>) -> Self {
        self.return_type = Some(return_type.into());
        self
    }

    /// Add a parameter
    pub fn param(mut self, name: impl Into<String>, param_type: impl Into<FieldType>) -> Self {
        self.parameters.push(FunctionParameter {
            name: name.into(),
            param_type: param_type.into(),
            is_hidden: false,
        });
        self
    }

    /// Add a hidden parameter (like 'this' pointer)
    pub fn hidden_param(mut self, name: impl Into<String>, param_type: impl Into<FieldType>) -> Self {
        self.parameters.push(FunctionParameter {
            name: name.into(),
            param_type: param_type.into(),
            is_hidden: true,
        });
        self
    }

    /// Set calling convention
    pub fn calling_convention(mut self, cc: CallingConvention) -> Self {
        self.calling_convention = cc;
        self
    }

    /// Set vararg flag
    pub fn vararg(mut self, is_vararg: bool) -> Self {
        self.is_vararg = is_vararg;
        self
    }

    /// Mark function as noreturn
    pub fn noreturn(mut self) -> Self {
        self.attributes.is_noreturn = true;
        self
    }

    /// Mark function as pure
    pub fn pure_func(mut self) -> Self {
        self.attributes.is_pure = true;
        self
    }

    /// Mark function as static
    pub fn static_func(mut self) -> Self {
        self.attributes.is_static = true;
        self
    }

    /// Mark function as virtual
    pub fn virtual_func(mut self) -> Self {
        self.attributes.is_virtual = true;
        self
    }

    /// Mark function as const (member function)
    pub fn const_func(mut self) -> Self {
        self.attributes.is_const = true;
        self
    }

    /// Mark function as constructor
    pub fn constructor(mut self) -> Self {
        self.attributes.is_constructor = true;
        self
    }

    /// Mark function as destructor
    pub fn destructor(mut self) -> Self {
        self.attributes.is_destructor = true;
        self
    }
}

impl TypeValidator for FunctionBuilder {
    fn validate(&self) -> Result<(), IDAError> {
        // Check for duplicate parameter names
        let mut param_names = std::collections::HashSet::new();
        for param in &self.parameters {
            if !param.name.is_empty() && !param_names.insert(&param.name) {
                return Err(IDAError::ffi_with(format!(
                    "Duplicate parameter name '{}'",
                    param.name
                )));
            }
        }
        
        // Validate that constructor/destructor don't have conflicting attributes
        if self.attributes.is_constructor && self.attributes.is_destructor {
            return Err(IDAError::ffi_with(
                "Function cannot be both constructor and destructor"
            ));
        }
        
        Ok(())
    }
}

impl TypeBuilder for FunctionBuilder {
    fn build(self) -> Result<Type, IDAError> {
        // Validate before building
        TypeValidator::validate(&self)?;
        
        // Get return type ordinal
        let return_ordinal = match self.return_type {
            Some(FieldType::Primitive(prim)) => get_primitive_type_ordinal(prim.to_ida_type()),
            Some(FieldType::Existing(typ)) => typ.ordinal(),
            Some(FieldType::ForwardRef(_)) => {
                return Err(IDAError::ffi_with(
                    "Forward references not supported in return types"
                ));
            }
            None => 0, // void return
        };
        
        // Create the function type
        let func_ordinal = create_function_type(
            return_ordinal,
            self.calling_convention.to_ida_cc(),
            self.is_vararg,
        );
        
        if func_ordinal == 0 {
            return Err(IDAError::ffi_with("Failed to create function type"));
        }
        
        // Add parameters
        for param in self.parameters {
            let param_ordinal = match param.param_type {
                FieldType::Primitive(prim) => get_primitive_type_ordinal(prim.to_ida_type()),
                FieldType::Existing(typ) => typ.ordinal(),
                FieldType::ForwardRef(_) => {
                    return Err(IDAError::ffi_with(
                        "Forward references not supported in parameter types"
                    ));
                }
            };
            
            if param_ordinal == 0 {
                return Err(IDAError::ffi_with(format!(
                    "Invalid type for parameter '{}'",
                    param.name
                )));
            }
            
            if !add_function_parameter(
                func_ordinal,
                &param.name,
                param_ordinal,
                param.is_hidden,
            ) {
                return Err(IDAError::ffi_with(format!(
                    "Failed to add parameter '{}'",
                    param.name
                )));
            }
        }
        
        // Set function attributes
        if !set_function_attributes(
            func_ordinal,
            self.attributes.is_noreturn,
            self.attributes.is_pure,
            self.attributes.is_static,
            self.attributes.is_virtual,
            self.attributes.is_const,
            self.attributes.is_constructor,
            self.attributes.is_destructor,
        ) {
            return Err(IDAError::ffi_with("Failed to set function attributes"));
        }
        
        Ok(Type::from_ordinal(func_ordinal))
    }
}

/// Builder for creating function pointer types
#[derive(Debug, Clone)]
pub struct FunctionPointerBuilder {
    function_type: Type,
}

impl FunctionPointerBuilder {
    /// Create a new function pointer builder
    pub fn new(function_type: Type) -> Self {
        Self { function_type }
    }
}

impl TypeBuilder for FunctionPointerBuilder {
    fn build(self) -> Result<Type, IDAError> {
        let ptr_ordinal = create_function_pointer_type(self.function_type.ordinal());
        
        if ptr_ordinal == 0 {
            return Err(IDAError::ffi_with("Failed to create function pointer type"));
        }
        
        Ok(Type::from_ordinal(ptr_ordinal))
    }
}

/// Convenience module for builder creation
pub mod builders {
    use super::*;

    /// Create a new struct builder
    pub fn struct_type(name: impl Into<String>) -> StructBuilder {
        StructBuilder::new(name)
    }

    /// Create a new union builder
    pub fn union_type(name: impl Into<String>) -> StructBuilder {
        StructBuilder::new_union(name)
    }

    /// Create a new enum builder
    pub fn enum_type(name: impl Into<String>, width: u32) -> EnumBuilder {
        EnumBuilder::new(name, width)
    }

    /// Create a new array builder
    pub fn array_type(element_type: impl Into<FieldType>, num_elements: u32) -> ArrayBuilder {
        ArrayBuilder::new(element_type, num_elements)
    }

    /// Create a new pointer builder
    pub fn pointer_type(target_type: impl Into<FieldType>) -> PointerBuilder {
        PointerBuilder::new(target_type)
    }

    /// Create a new function builder
    pub fn function_type() -> FunctionBuilder {
        FunctionBuilder::new()
    }

    /// Create a new function pointer builder
    pub fn function_pointer(function_type: Type) -> FunctionPointerBuilder {
        FunctionPointerBuilder::new(function_type)
    }

    /// Create primitive type builders
    pub fn int8() -> PrimitiveType {
        PrimitiveType::Int8
    }

    pub fn int16() -> PrimitiveType {
        PrimitiveType::Int16
    }

    pub fn int32() -> PrimitiveType {
        PrimitiveType::Int32
    }

    pub fn int64() -> PrimitiveType {
        PrimitiveType::Int64
    }

    pub fn uint8() -> PrimitiveType {
        PrimitiveType::UInt8
    }

    pub fn uint16() -> PrimitiveType {
        PrimitiveType::UInt16
    }

    pub fn uint32() -> PrimitiveType {
        PrimitiveType::UInt32
    }

    pub fn uint64() -> PrimitiveType {
        PrimitiveType::UInt64
    }

    pub fn float() -> PrimitiveType {
        PrimitiveType::Float
    }

    pub fn double() -> PrimitiveType {
        PrimitiveType::Double
    }

    pub fn void() -> PrimitiveType {
        PrimitiveType::Void
    }

    pub fn char() -> PrimitiveType {
        PrimitiveType::Char
    }

    pub fn bool() -> PrimitiveType {
        PrimitiveType::Bool
    }
}