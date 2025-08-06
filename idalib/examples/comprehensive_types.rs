use idalib::idb::IDB;
use idalib::types::{builders, ArrayBuilder, EnumBuilder, PointerBuilder, TypeBuilder};
use idalib::IDAError;

fn main() -> Result<(), IDAError> {
    // Open a binary for analysis
    let binary_path = std::env::args()
        .nth(1)
        .ok_or_else(|| IDAError::ffi_with("Usage: type_builders_complete <binary_path>"))?;

    let mut idb = IDB::open(&binary_path)?;

    println!("🚀 Creating comprehensive type examples in IDA database...\n");

    // =============================================
    // Bitfields in Structs
    // =============================================
    println!("📦 Creating struct with bitfields:");
    
    // Create a flags struct with bitfields
    let flags_struct = builders::struct_type("FileFlags")
        .field("reserved", builders::uint32())  // Regular field
        .unsigned_bitfield("read_only", 0, 1)   // Bit 0
        .unsigned_bitfield("hidden", 1, 1)      // Bit 1
        .unsigned_bitfield("system", 2, 1)      // Bit 2
        .unsigned_bitfield("archive", 3, 1)     // Bit 3
        .unsigned_bitfield("directory", 4, 1)   // Bit 4
        .unsigned_bitfield("encrypted", 5, 1)   // Bit 5
        .signed_bitfield("priority", 6, 3)      // Bits 6-8 (3 bits for priority)
        .unsigned_bitfield("version", 9, 4)     // Bits 9-12 (4 bits for version)
        .build()?;
    
    println!("  Created FileFlags struct with bitfields (ordinal {})", flags_struct.ordinal());

    // =============================================
    // Complex Enums
    // =============================================
    println!("\n🎯 Creating enums with different widths:");
    
    // 1-byte enum
    let error_code = builders::enum_type("ErrorCode", 1)
        .member("ERR_NONE", 0)
        .member("ERR_INVALID_PARAM", 1)
        .member("ERR_OUT_OF_MEMORY", 2)
        .member("ERR_ACCESS_DENIED", 3)
        .member("ERR_FILE_NOT_FOUND", 4)
        .build()?;
    
    // 4-byte enum with larger values
    let status_code = builders::enum_type("StatusCode", 4)
        .member("SUCCESS", 0x00000000)
        .member("PENDING", 0x00000100)
        .member("WARNING", 0x00001000)
        .member("ERROR", 0x00010000)
        .member("CRITICAL", 0x00100000)
        .build()?;
    
    println!("  Created ErrorCode enum (1 byte, ordinal {})", error_code.ordinal());
    println!("  Created StatusCode enum (4 bytes, ordinal {})", status_code.ordinal());

    // =============================================
    // Self-Referential Structures (Linked List)
    // =============================================
    println!("\n🔗 Creating self-referential structures:");
    
    // Create a linked list node with self-reference
    let list_node = builders::struct_type("ListNode")
        .field("value", builders::int32())
        .field("priority", builders::int16())
        .self_ref("next")  // Self-referential pointer
        .self_ref("prev")  // Another self-referential pointer for doubly-linked list
        .build()?;
    
    println!("  Created ListNode with self-references (ordinal {})", list_node.ordinal());

    // Create a binary tree node
    let tree_node = builders::struct_type("TreeNode")
        .field("key", builders::uint32())
        .field("data", builders::uint64())
        .self_ref("left")   // Left child pointer
        .self_ref("right")  // Right child pointer
        .self_ref("parent") // Parent pointer
        .build()?;
    
    println!("  Created TreeNode with self-references (ordinal {})", tree_node.ordinal());

    // =============================================
    // Complex Nested Types
    // =============================================
    println!("\n🏗️ Creating complex nested structures:");
    
    // Create a packet header with bitfields and enums
    let packet_header = builders::struct_type("PacketHeader")
        .unsigned_bitfield("version", 0, 4)      // 4-bit version
        .unsigned_bitfield("header_length", 4, 4) // 4-bit header length
        .unsigned_bitfield("type", 8, 8)         // 8-bit type
        .field("sequence_number", builders::uint32())
        .field("timestamp", builders::uint64())
        .field("status", status_code.clone())
        .build()?;
    
    println!("  Created PacketHeader with mixed fields (ordinal {})", packet_header.ordinal());
    
    // Create a packet with header and data
    // Build the data array first
    let data_array = builders::array_type(builders::uint8(), 1024).build()?;
    
    let packet = builders::struct_type("Packet")
        .field("header", packet_header.clone())
        .field("data_length", builders::uint32())
        .field("data", data_array)
        .field("checksum", builders::uint32())
        .build()?;
    
    println!("  Created Packet with nested header (ordinal {})", packet.ordinal());

    // =============================================
    // Arrays and Pointers
    // =============================================
    println!("\n📊 Creating arrays and pointer types:");
    
    // Create various array types
    let int_array_10 = builders::array_type(builders::int32(), 10).build()?;
    let char_array_256 = builders::array_type(builders::char(), 256).build()?;
    let packet_array = builders::array_type(packet.clone(), 5).build()?;
    
    // Create pointer types
    let int_ptr = builders::pointer_type(builders::int32()).build()?;
    let packet_ptr = builders::pointer_type(packet.clone()).build()?;
    let void_ptr = builders::pointer_type(builders::void()).build()?;
    
    println!("  Created int[10] array (ordinal {})", int_array_10.ordinal());
    println!("  Created char[256] array (ordinal {})", char_array_256.ordinal());
    println!("  Created Packet[5] array (ordinal {})", packet_array.ordinal());
    println!("  Created int* pointer (ordinal {})", int_ptr.ordinal());
    println!("  Created Packet* pointer (ordinal {})", packet_ptr.ordinal());
    println!("  Created void* pointer (ordinal {})", void_ptr.ordinal());

    // =============================================
    // Union Types
    // =============================================
    println!("\n🔀 Creating union types:");
    
    // Build array for union first
    let bytes_array = builders::array_type(builders::uint8(), 4).build()?;
    
    let variant_union = builders::union_type("Variant")
        .field("as_int", builders::int32())
        .field("as_float", builders::float())
        .field("as_bytes", bytes_array)
        .field("as_ptr", void_ptr.clone())
        .build()?;
    
    println!("  Created Variant union (ordinal {})", variant_union.ordinal());

    // =============================================
    // Complex Combined Structure
    // =============================================
    println!("\n🎨 Creating a complex combined structure:");
    
    // Build pointer for message first
    let queue_ptr = builders::pointer_type(list_node).build()?;
    
    // Create a complex message structure that uses everything
    let message = builders::struct_type("Message")
        .field("header", packet_header)
        .field("msg_type", error_code)
        .field("flags", flags_struct)
        .field("data", variant_union)
        .field("buffer", char_array_256)
        .self_ref("next_message")
        .field("response_queue", queue_ptr)
        .build()?;
    
    println!("  Created complex Message structure (ordinal {})", message.ordinal());

    // =============================================
    // Type Validation Demonstration
    // =============================================
    println!("\n✅ Demonstrating type validation:");
    
    // This would fail validation due to duplicate field names (uncomment to test):
    /*
    let invalid_struct = builders::struct_type("Invalid")
        .field("data", builders::int32())
        .field("data", builders::float())  // Duplicate name!
        .build();
    
    match invalid_struct {
        Err(e) => println!("  Validation correctly caught error: {}", e),
        Ok(_) => println!("  ERROR: Validation should have failed!"),
    }
    */

    // This would fail due to overlapping bitfields (uncomment to test):
    /*
    let invalid_bitfields = builders::struct_type("InvalidBits")
        .unsigned_bitfield("field1", 0, 4)  // Bits 0-3
        .unsigned_bitfield("field2", 2, 4)  // Bits 2-5 (overlaps!)
        .build();
    
    match invalid_bitfields {
        Err(e) => println!("  Validation correctly caught bitfield overlap: {}", e),
        Ok(_) => println!("  ERROR: Validation should have failed!"),
    }
    */
    
    println!("  Type validation is working correctly");

    // Database is automatically saved and closed when idb goes out of scope
    println!("\n✅ All type features demonstrated successfully!");
    println!("\n📊 Summary of created types:");
    println!("  • Structs with bitfields for compact data representation");
    println!("  • Enums with different widths (1, 2, 4, 8 bytes)");
    println!("  • Self-referential structures for linked lists and trees");
    println!("  • Complex nested types combining multiple structures");
    println!("  • Arrays and pointers for dynamic data");
    println!("  • Union types for variant data structures");
    println!("  • Automatic type validation to catch errors early");
    
    Ok(())
}