//! Memoization Runtime Support - LRU Cache and Unbounded Cache
//!
//! Provides runtime functions for memoization decorators:
//! - @lru_cache(maxsize=N) - LRU eviction policy
//! - @cache - Unbounded cache (maxsize=None)
//!
//! Uses ARC (Automatic Reference Counting) for memory management.

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::{FunctionValue, PointerValue};

/// Memoization runtime functions
pub struct MemoizationFunctions<'ctx> {
    // ARC key creation functions (supports 1-8 parameters)
    pub arc_key_create1: FunctionValue<'ctx>,
    pub arc_key_create2: FunctionValue<'ctx>,
    pub arc_key_create3: FunctionValue<'ctx>,
    pub arc_key_create4: FunctionValue<'ctx>,
    pub arc_key_create5: FunctionValue<'ctx>,
    pub arc_key_create6: FunctionValue<'ctx>,
    pub arc_key_create7: FunctionValue<'ctx>,
    pub arc_key_create8: FunctionValue<'ctx>,

    // LRU Cache functions
    pub lru_cache_create: FunctionValue<'ctx>,
    pub lru_cache_get: FunctionValue<'ctx>,
    pub lru_cache_set: FunctionValue<'ctx>,
    pub lru_cache_destroy: FunctionValue<'ctx>,

    // Unbounded Cache functions
    pub cache_create: FunctionValue<'ctx>,
    pub cache_get: FunctionValue<'ctx>,
    pub cache_set: FunctionValue<'ctx>,
    pub cache_destroy: FunctionValue<'ctx>,

    // Backward compatibility (wrappers around arc_key_create)
    pub tuple_create1: FunctionValue<'ctx>,
    pub tuple_create2: FunctionValue<'ctx>,
}

/// Declare memoization runtime functions
pub fn declare_memoization_functions<'ctx>(
    context: &'ctx Context,
    module: &mut Module<'ctx>,
) -> crate::codegen::Result<MemoizationFunctions<'ctx>> {
    let i8_ptr_type = context.ptr_type(inkwell::AddressSpace::default());
    let i64_type = context.i64_type();
    let i32_type = context.i32_type();
    let void_type = context.void_type();
    // LLVM 15+: use context.ptr_type() instead of i32_type.ptr_type()
    let i32_ptr_type = context.ptr_type(inkwell::AddressSpace::default());

    // ARC key creation functions (return ARCCacheKey*)
    let arc_key_create1_type = i8_ptr_type.fn_type(&[i64_type.into()], false);
    let arc_key_create1 = module.add_function("arc_key_create1", arc_key_create1_type, None);

    let arc_key_create2_type = i8_ptr_type.fn_type(&[i64_type.into(), i64_type.into()], false);
    let arc_key_create2 = module.add_function("arc_key_create2", arc_key_create2_type, None);

    let arc_key_create3_type =
        i8_ptr_type.fn_type(&[i64_type.into(), i64_type.into(), i64_type.into()], false);
    let arc_key_create3 = module.add_function("arc_key_create3", arc_key_create3_type, None);

    let arc_key_create4_type = i8_ptr_type
        .fn_type(&[i64_type.into(), i64_type.into(), i64_type.into(), i64_type.into()], false);
    let arc_key_create4 = module.add_function("arc_key_create4", arc_key_create4_type, None);

    let arc_key_create5_type = i8_ptr_type.fn_type(
        &[i64_type.into(), i64_type.into(), i64_type.into(), i64_type.into(), i64_type.into()],
        false,
    );
    let arc_key_create5 = module.add_function("arc_key_create5", arc_key_create5_type, None);

    let arc_key_create6_type = i8_ptr_type.fn_type(
        &[
            i64_type.into(),
            i64_type.into(),
            i64_type.into(),
            i64_type.into(),
            i64_type.into(),
            i64_type.into(),
        ],
        false,
    );
    let arc_key_create6 = module.add_function("arc_key_create6", arc_key_create6_type, None);

    let arc_key_create7_type = i8_ptr_type.fn_type(
        &[
            i64_type.into(),
            i64_type.into(),
            i64_type.into(),
            i64_type.into(),
            i64_type.into(),
            i64_type.into(),
            i64_type.into(),
        ],
        false,
    );
    let arc_key_create7 = module.add_function("arc_key_create7", arc_key_create7_type, None);

    let arc_key_create8_type = i8_ptr_type.fn_type(
        &[
            i64_type.into(),
            i64_type.into(),
            i64_type.into(),
            i64_type.into(),
            i64_type.into(),
            i64_type.into(),
            i64_type.into(),
            i64_type.into(),
        ],
        false,
    );
    let arc_key_create8 = module.add_function("arc_key_create8", arc_key_create8_type, None);

    // LRU Cache functions (updated signatures for ARCCacheKey*)
    let lru_cache_create_type = i8_ptr_type.fn_type(&[i64_type.into()], false);
    let lru_cache_create = module.add_function("vp_lru_cache_create", lru_cache_create_type, None);

    let lru_cache_get_type = i64_type.fn_type(
        &[
            i8_ptr_type.into(),  // cache
            i8_ptr_type.into(),  // key (ARCCacheKey*)
            i32_ptr_type.into(), // found
            i32_ptr_type.into(), // is_bigint
        ],
        false,
    );
    let lru_cache_get = module.add_function("vp_lru_cache_get", lru_cache_get_type, None);

    let lru_cache_set_type = void_type.fn_type(
        &[
            i8_ptr_type.into(), // cache
            i8_ptr_type.into(), // key (ARCCacheKey*)
            i64_type.into(),    // value
            i32_type.into(),    // is_bigint
        ],
        false,
    );
    let lru_cache_set = module.add_function("vp_lru_cache_set", lru_cache_set_type, None);

    let lru_cache_destroy_type = void_type.fn_type(&[i8_ptr_type.into()], false);
    let lru_cache_destroy =
        module.add_function("vp_lru_cache_destroy", lru_cache_destroy_type, None);

    // Unbounded cache functions (updated signatures for ARCCacheKey*)
    let cache_create_type = i8_ptr_type.fn_type(&[], false);
    let cache_create = module.add_function("vp_cache_create", cache_create_type, None);

    let cache_get_type = i64_type.fn_type(
        &[i8_ptr_type.into(), i8_ptr_type.into(), i32_ptr_type.into(), i32_ptr_type.into()],
        false,
    );
    let cache_get = module.add_function("vp_cache_get", cache_get_type, None);

    let cache_set_type = void_type.fn_type(
        &[i8_ptr_type.into(), i8_ptr_type.into(), i64_type.into(), i32_type.into()],
        false,
    );
    let cache_set = module.add_function("vp_cache_set", cache_set_type, None);

    let cache_destroy_type = void_type.fn_type(&[i8_ptr_type.into()], false);
    let cache_destroy = module.add_function("vp_cache_destroy", cache_destroy_type, None);

    // Backward compatibility wrappers
    let tuple_create1_type = i8_ptr_type.fn_type(&[i64_type.into()], false);
    let tuple_create1 = module.add_function("vp_tuple_create1", tuple_create1_type, None);

    let tuple_create2_type = i8_ptr_type.fn_type(&[i64_type.into(), i64_type.into()], false);
    let tuple_create2 = module.add_function("vp_tuple_create2", tuple_create2_type, None);

    Ok(MemoizationFunctions {
        arc_key_create1,
        arc_key_create2,
        arc_key_create3,
        arc_key_create4,
        arc_key_create5,
        arc_key_create6,
        arc_key_create7,
        arc_key_create8,
        lru_cache_create,
        lru_cache_get,
        lru_cache_set,
        lru_cache_destroy,
        cache_create,
        cache_get,
        cache_set,
        cache_destroy,
        tuple_create1,
        tuple_create2,
    })
}

/// Create a global cache pointer for a memoized function
pub fn create_cache_global<'ctx>(
    context: &'ctx Context,
    module: &mut Module<'ctx>,
    func_name: &str,
    is_lru: bool,
) -> PointerValue<'ctx> {
    let i8_ptr_type = context.ptr_type(inkwell::AddressSpace::default());
    let global_name =
        format!("__memo_cache_{}_{}", if is_lru { "lru" } else { "unbounded" }, func_name);

    let global =
        module.add_global(i8_ptr_type, Some(inkwell::AddressSpace::default()), &global_name);
    global.set_initializer(&i8_ptr_type.const_null());
    // Note: linkage set during optimization

    global.as_pointer_value()
}
