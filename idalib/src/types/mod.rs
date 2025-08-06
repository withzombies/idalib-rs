// Core types module
mod types_core;
pub use types_core::*;

// Export the builder module
pub mod builder;

// Re-export commonly used builder items at the module level
pub use builder::{
    builders, FieldType, PrimitiveType, StructBuilder, TypeBuilder,
    EnumBuilder, ArrayBuilder, PointerBuilder,
    FunctionBuilder, FunctionPointerBuilder, CallingConvention,
};