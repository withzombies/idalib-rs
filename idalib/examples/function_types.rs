use idalib::idb::IDB;
use idalib::types::{builders, CallingConvention, FunctionBuilder, TypeBuilder};
use idalib::IDAError;

fn main() -> Result<(), IDAError> {
    // Open a binary for analysis
    let binary_path = std::env::args()
        .nth(1)
        .ok_or_else(|| IDAError::ffi_with("Usage: function_types <binary_path>"))?;

    let mut idb = IDB::open(&binary_path)?;

    println!("🔨 Building function prototypes in IDA database...\n");

    // =============================================
    // Simple Function Types
    // =============================================
    println!("📝 Creating simple function types:");
    
    // void function with no parameters
    let void_func = builders::function_type()
        .build()?;
    println!("  Created void function (ordinal {})", void_func.ordinal());
    
    // Function returning int with no parameters
    let get_value_func = builders::function_type()
        .returns(builders::int32())
        .build()?;
    println!("  Created int get_value() (ordinal {})", get_value_func.ordinal());
    
    // Function with parameters and return value
    let add_func = builders::function_type()
        .returns(builders::int32())
        .param("a", builders::int32())
        .param("b", builders::int32())
        .build()?;
    println!("  Created int add(int a, int b) (ordinal {})", add_func.ordinal());

    // =============================================
    // Calling Conventions
    // =============================================
    println!("\n📞 Functions with different calling conventions:");
    
    // Cdecl function
    let cdecl_func = builders::function_type()
        .returns(builders::int32())
        .param("arg1", builders::int32())
        .param("arg2", builders::int32())
        .calling_convention(CallingConvention::Cdecl)
        .build()?;
    println!("  Created cdecl function (ordinal {})", cdecl_func.ordinal());
    
    // Stdcall function
    let stdcall_func = builders::function_type()
        .returns(builders::int32())
        .param("hWnd", builders::uint32())
        .param("uMsg", builders::uint32())
        .param("wParam", builders::uint32())
        .param("lParam", builders::uint32())
        .calling_convention(CallingConvention::Stdcall)
        .build()?;
    println!("  Created stdcall WndProc (ordinal {})", stdcall_func.ordinal());
    
    // Fastcall function
    let fastcall_func = builders::function_type()
        .returns(builders::uint64())
        .param("a", builders::uint64())
        .param("b", builders::uint64())
        .calling_convention(CallingConvention::Fastcall)
        .build()?;
    println!("  Created fastcall function (ordinal {})", fastcall_func.ordinal());

    // =============================================
    // Variadic Functions
    // =============================================
    println!("\n🔢 Creating variadic functions:");
    
    // printf-like function
    let printf_func = builders::function_type()
        .returns(builders::int32())
        .param("format", builders::pointer_type(builders::char()).build()?)
        .vararg(true)
        .calling_convention(CallingConvention::Cdecl)
        .build()?;
    println!("  Created printf-like function (ordinal {})", printf_func.ordinal());
    
    // Custom logging function with varargs
    let log_func = builders::function_type()
        .returns(builders::void())
        .param("level", builders::int32())
        .param("file", builders::pointer_type(builders::char()).build()?)
        .param("line", builders::int32())
        .param("format", builders::pointer_type(builders::char()).build()?)
        .vararg(true)
        .build()?;
    println!("  Created logging function with varargs (ordinal {})", log_func.ordinal());

    // =============================================
    // Function Attributes
    // =============================================
    println!("\n⚡ Functions with special attributes:");
    
    // Noreturn function (like exit or abort)
    let exit_func = builders::function_type()
        .param("status", builders::int32())
        .noreturn()
        .build()?;
    println!("  Created noreturn exit function (ordinal {})", exit_func.ordinal());
    
    // Pure function (no side effects)
    let strlen_func = builders::function_type()
        .returns(builders::uint64())
        .param("str", builders::pointer_type(builders::char()).build()?)
        .pure_func()
        .build()?;
    println!("  Created pure strlen function (ordinal {})", strlen_func.ordinal());
    
    // Static function
    let static_helper = builders::function_type()
        .returns(builders::int32())
        .param("data", builders::pointer_type(builders::void()).build()?)
        .static_func()
        .build()?;
    println!("  Created static helper function (ordinal {})", static_helper.ordinal());

    // =============================================
    // Member Functions (C++)
    // =============================================
    println!("\n🎭 Creating C++ member functions:");
    
    // First, create a simple class structure
    let my_class = builders::struct_type("MyClass")
        .field("data", builders::int32())
        .field("count", builders::uint32())
        .build()?;
    
    // Virtual member function
    let virtual_method = builders::function_type()
        .returns(builders::void())
        .hidden_param("this", builders::pointer_type(my_class.clone()).build()?)
        .param("value", builders::int32())
        .virtual_func()
        .calling_convention(CallingConvention::Thiscall)
        .build()?;
    println!("  Created virtual member function (ordinal {})", virtual_method.ordinal());
    
    // Const member function
    let const_getter = builders::function_type()
        .returns(builders::int32())
        .hidden_param("this", builders::pointer_type(my_class.clone()).build()?)
        .const_func()
        .calling_convention(CallingConvention::Thiscall)
        .build()?;
    println!("  Created const getter function (ordinal {})", const_getter.ordinal());
    
    // Constructor
    let constructor = builders::function_type()
        .returns(builders::pointer_type(my_class.clone()).build()?)
        .hidden_param("this", builders::pointer_type(my_class.clone()).build()?)
        .param("initial_value", builders::int32())
        .constructor()
        .calling_convention(CallingConvention::Thiscall)
        .build()?;
    println!("  Created constructor (ordinal {})", constructor.ordinal());
    
    // Destructor
    let destructor = builders::function_type()
        .hidden_param("this", builders::pointer_type(my_class).build()?)
        .destructor()
        .calling_convention(CallingConvention::Thiscall)
        .build()?;
    println!("  Created destructor (ordinal {})", destructor.ordinal());

    // =============================================
    // Function Pointers
    // =============================================
    println!("\n➡️ Creating function pointer types:");
    
    // Create a callback function type
    let callback_func = builders::function_type()
        .returns(builders::int32())
        .param("event", builders::uint32())
        .param("data", builders::pointer_type(builders::void()).build()?)
        .build()?;
    
    // Create a function pointer to the callback
    let callback_ptr = builders::function_pointer(callback_func.clone()).build()?;
    println!("  Created callback function pointer (ordinal {})", callback_ptr.ordinal());
    
    // Create a struct with function pointer field
    let event_handler = builders::struct_type("EventHandler")
        .field("id", builders::uint32())
        .field("callback", callback_ptr.clone())
        .field("user_data", builders::pointer_type(builders::void()).build()?)
        .build()?;
    println!("  Created EventHandler struct with function pointer (ordinal {})", event_handler.ordinal());
    
    // Function that takes a function pointer as parameter
    let register_handler = builders::function_type()
        .returns(builders::bool())
        .param("handler", callback_ptr)
        .param("priority", builders::int32())
        .build()?;
    println!("  Created register_handler function (ordinal {})", register_handler.ordinal());

    // =============================================
    // Complex Function Types
    // =============================================
    println!("\n🔮 Creating complex function types:");
    
    // Create a FILE structure (simplified)
    let file_struct = builders::struct_type("FILE")
        .field("fd", builders::int32())
        .field("flags", builders::uint32())
        .field("buffer", builders::pointer_type(builders::uint8()).build()?)
        .field("buffer_size", builders::uint64())
        .build()?;
    
    // fopen function
    let fopen_func = builders::function_type()
        .returns(builders::pointer_type(file_struct.clone()).build()?)
        .param("filename", builders::pointer_type(builders::char()).build()?)
        .param("mode", builders::pointer_type(builders::char()).build()?)
        .build()?;
    println!("  Created fopen function (ordinal {})", fopen_func.ordinal());
    
    // fread function with complex parameters
    let fread_func = builders::function_type()
        .returns(builders::uint64())  // size_t
        .param("ptr", builders::pointer_type(builders::void()).build()?)
        .param("size", builders::uint64())
        .param("count", builders::uint64())
        .param("stream", builders::pointer_type(file_struct).build()?)
        .build()?;
    println!("  Created fread function (ordinal {})", fread_func.ordinal());
    
    // Signal handler function type
    let sighandler_t = builders::function_type()
        .param("sig", builders::int32())
        .build()?;
    
    // signal() function that returns a function pointer
    let signal_func_ptr = builders::function_pointer(sighandler_t.clone()).build()?;
    let signal_func = builders::function_type()
        .returns(signal_func_ptr.clone())
        .param("sig", builders::int32())
        .param("handler", signal_func_ptr)
        .build()?;
    println!("  Created signal() function returning function pointer (ordinal {})", signal_func.ordinal());

    // =============================================
    // Platform-specific Functions
    // =============================================
    println!("\n💻 Creating platform-specific function types:");
    
    // Windows API function
    let message_box = builders::function_type()
        .returns(builders::int32())
        .param("hWnd", builders::pointer_type(builders::void()).build()?)
        .param("lpText", builders::pointer_type(builders::char()).build()?)
        .param("lpCaption", builders::pointer_type(builders::char()).build()?)
        .param("uType", builders::uint32())
        .calling_convention(CallingConvention::Stdcall)
        .build()?;
    println!("  Created MessageBox Windows API function (ordinal {})", message_box.ordinal());
    
    // Linux syscall-like function
    let syscall_func = builders::function_type()
        .returns(builders::int64())
        .param("number", builders::int64())
        .vararg(true)
        .build()?;
    println!("  Created syscall function (ordinal {})", syscall_func.ordinal());

    // Database is automatically saved and closed when idb goes out of scope
    println!("\n✅ All function type features demonstrated successfully!");
    println!("\n📋 Function Types Created:");
    println!("  • Simple functions with parameters and return values");
    println!("  • Functions with different calling conventions (cdecl, stdcall, fastcall)");
    println!("  • Variadic functions for flexible argument lists");
    println!("  • Special function attributes (noreturn, pure, static)");
    println!("  • C++ member functions with virtual, const, and constructor/destructor support");
    println!("  • Function pointers for callbacks and event handlers");
    println!("  • Complex function signatures with structures");
    println!("  • Platform-specific functions (Windows API, Linux syscalls)");
    println!("\n💡 These function prototypes can now be applied to functions in your binary!");
    
    Ok(())
}