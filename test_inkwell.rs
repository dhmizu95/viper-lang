use inkwell::context::Context;
use inkwell::targets::{InitializationConfig, Target};
use inkwell::OptimizationLevel;

fn main() {
    Target::initialize_native(&InitializationConfig::default()).unwrap();
    let ctx = Context::create();
    let m1 = ctx.create_module("m1");
    let ee = m1.create_jit_execution_engine(OptimizationLevel::None).unwrap();
    
    let m2 = ctx.create_module("m2");
    ee.add_module(&m2).unwrap();
    println!("Added module successfully!");
}
