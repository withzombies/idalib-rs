use idalib::idb::IDB;
use idalib::types::{builders, ArrayBuilder, EnumBuilder, PointerBuilder, TypeBuilder};
use idalib::IDAError;

fn main() -> Result<(), IDAError> {
    // Open a binary for analysis
    let binary_path = std::env::args()
        .nth(1)
        .ok_or_else(|| IDAError::ffi_with("Usage: advanced_types <binary_path>"))?;

    let mut idb = IDB::open(&binary_path)?;

    println!("🔧 Building advanced type definitions...\n");

    // Create an enum with explicit values
    let status_enum = builders::enum_type("Status", 4)
        .member("STATUS_SUCCESS", 0)
        .member("STATUS_ERROR", -1)
        .member("STATUS_PENDING", 100)
        .member("STATUS_TIMEOUT", 200)
        .build()?;
    println!("Created enum 'Status' with ordinal {}", status_enum.ordinal());

    // Create an enum with auto-incremented values
    let color_enum = EnumBuilder::new("Color", 1)
        .auto_member("RED")    // 0
        .auto_member("GREEN")  // 1
        .auto_member("BLUE")   // 2
        .auto_member("YELLOW") // 3
        .build()?;
    println!("Created enum 'Color' with ordinal {}", color_enum.ordinal());

    // Create an array of integers
    let int_array = builders::array_type(builders::int32(), 10).build()?;
    println!("Created int[10] array with ordinal {}", int_array.ordinal());

    // Create a pointer to int
    let int_ptr = builders::pointer_type(builders::int32()).build()?;
    println!("Created int* pointer with ordinal {}", int_ptr.ordinal());

    // Create a struct with enum and array fields
    let complex_struct = builders::struct_type("ComplexData")
        .field("status", status_enum.clone())
        .field("color", color_enum.clone())
        .field("values", int_array.clone())
        .field("next", int_ptr.clone())
        .build()?;
    println!(
        "Created struct 'ComplexData' with ordinal {}",
        complex_struct.ordinal()
    );

    // Create an array of structs
    let struct_array = ArrayBuilder::new(complex_struct.clone(), 5).build()?;
    println!(
        "Created ComplexData[5] array with ordinal {}",
        struct_array.ordinal()
    );

    // Create a pointer to the struct (for linked list scenarios)
    let struct_ptr = PointerBuilder::new(complex_struct.clone()).build()?;
    println!(
        "Created ComplexData* pointer with ordinal {}",
        struct_ptr.ordinal()
    );

    // Create a nested struct that uses pointers and arrays
    let linked_node = builders::struct_type("LinkedNode")
        .field("data", int_array)
        .field("status", status_enum)
        .field("next", struct_ptr)
        .build()?;
    println!(
        "Created struct 'LinkedNode' with ordinal {}",
        linked_node.ordinal()
    );

    // Database is automatically saved and closed when idb goes out of scope
    
    println!("\n✨ Successfully created all advanced types!");
    println!("\nYou can now use these types to annotate your binary in IDA Pro.");
    
    Ok(())
}