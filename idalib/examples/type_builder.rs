use std::env;
use std::path::Path;

use idalib::idb::IDB;
use idalib::types::{PrimitiveType, StructBuilder, TypeBuilder, builders};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup: get binary path from args
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <binary>", args[0]);
        std::process::exit(1);
    }

    let binary_path = Path::new(&args[1]);
    if !binary_path.exists() {
        eprintln!("Error: File '{}' not found", binary_path.display());
        std::process::exit(1);
    }

    // Open the IDA database
    let mut idb = IDB::open(binary_path)?;
    println!("✨ Opened database for: {}", binary_path.display());

    // Test primitive type creation
    println!("\n📊 Testing Primitive Types");
    
    let int32_type = PrimitiveType::Int32.to_type()?;
    println!("  • int32 type (ordinal: {})", int32_type.ordinal());
    
    let uint64_type = PrimitiveType::UInt64.to_type()?;
    println!("  • uint64 type (ordinal: {})", uint64_type.ordinal());
    
    let float_type = PrimitiveType::Float.to_type()?;
    println!("  • float type (ordinal: {})", float_type.ordinal());

    // Test struct creation
    println!("\n🏗️ Creating Simple Struct");
    let point_struct = builders::struct_type("Point")
        .field("x", builders::int32())
        .field("y", builders::int32())
        .build()?;
    
    println!("  ✓ Created struct 'Point' (ordinal: {})", point_struct.ordinal());
    if let Some(name) = point_struct.name() {
        println!("    Name verified: {}", name);
    }

    // Create a more complex struct
    println!("\n🔧 Creating Complex Struct");
    let data_struct = builders::struct_type("DataBlock")
        .field("id", builders::uint64())
        .field("size", builders::uint32())
        .field("value", builders::double())
        .field("flags", builders::uint8())
        .build()?;
    
    println!("  ✓ Created struct 'DataBlock' (ordinal: {})", data_struct.ordinal());

    // Create a union
    println!("\n🔀 Creating Union");
    let variant_union = builders::union_type("Variant")
        .field("as_int", builders::int32())
        .field("as_float", builders::float())
        .field("as_uint", builders::uint32())
        .build()?;
    
    println!("  ✓ Created union 'Variant' (ordinal: {})", variant_union.ordinal());

    // List all types
    println!("\n📋 Types in Database");
    let types = idb.types();
    println!("  📊 Total types in database: {}", types.len());

    // Database is automatically closed when idb goes out of scope
    drop(idb);
    println!("\n✅ Database closed successfully");

    Ok(())
}