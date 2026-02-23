use inkwell::context::Context;

fn main() {
    // Create a LLVM Context
    let context = Context::create();
    // Create a Module
    let module = context.create_module("test");
    
    // Verify we can create a basic type
    let i64_type = context.i64_type();
    
    println!("✅ LLVM 20 Context Created Successfully!");
    println!("✅ Module Name: {}", module.get_name().to_str().unwrap());
    println!("✅ Type Check: {:?}", i64_type);
}