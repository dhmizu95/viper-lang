sed -i 's/let is_float_list = match \&iter_type {/println!("ITER_TYPE: {:?}", iter_type);\n    let is_float_list = match \&iter_type {/' src/codegen/control_flow/loops.rs
