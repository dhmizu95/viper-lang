//! Register a single JIT stub function
macro_rules! register_stub {
    ($ee:expr, $mod:expr, $func:literal, $stub:expr) => {
        if let Some(func) = $mod.get_function($func) {
            $ee.add_global_mapping(&func.as_global_value(), $stub as *const () as usize);
        }
    };
}

/// Register multiple JIT stub functions at once
macro_rules! register_stubs {
    ($ee:expr, $mod:expr, [$($func:literal => $stub:expr),* $(,)?]) => {
        $(
            register_stub!($ee, $mod, $func, $stub);
        )*
    };
}
