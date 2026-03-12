//! Memoization Runtime Support - LRU Cache and Unbounded Cache
//!
//! Provides runtime functions for memoization decorators:
//! - @lru_cache(maxsize=N) - LRU eviction policy
//! - @cache - Unbounded cache (maxsize=None)

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::BasicType;
use inkwell::values::{FunctionValue, GlobalValue, PointerValue};

/// Memoization runtime functions
pub struct MemoizationFunctions<'ctx> {
    pub lru_cache_create: FunctionValue<'ctx>,
    pub lru_cache_get: FunctionValue<'ctx>,
    pub lru_cache_set: FunctionValue<'ctx>,
    pub lru_cache_destroy: FunctionValue<'ctx>,
    pub cache_create: FunctionValue<'ctx>,
    pub cache_get: FunctionValue<'ctx>,
    pub cache_set: FunctionValue<'ctx>,
    pub cache_destroy: FunctionValue<'ctx>,
    pub tuple_create1: FunctionValue<'ctx>,
    pub tuple_create2: FunctionValue<'ctx>,
}

/// Declare memoization runtime functions
pub fn declare_memoization_functions<'ctx>(
    context: &'ctx Context,
    module: &mut Module<'ctx>,
) -> Result<MemoizationFunctions<'ctx>, String> {
    let i8_ptr_type = context.ptr_type(inkwell::AddressSpace::default());
    let i64_type = context.i64_type();
    let void_type = context.void_type();

    // LRU Cache functions
    let lru_cache_create_type = i8_ptr_type.fn_type(&[i64_type.into()], false);
    let lru_cache_create = module.add_function("vp_lru_cache_create", lru_cache_create_type, None);

    let lru_cache_get_type = i64_type.fn_type(&[i8_ptr_type.into(), i8_ptr_type.into(), context.i32_type().ptr_type(inkwell::AddressSpace::default()).into()], false);
    let lru_cache_get = module.add_function("vp_lru_cache_get", lru_cache_get_type, None);

    let lru_cache_set_type = void_type.fn_type(
        &[i8_ptr_type.into(), i8_ptr_type.into(), i64_type.into(), i64_type.into()],
        false,
    );
    let lru_cache_set = module.add_function("vp_lru_cache_set", lru_cache_set_type, None);

    let lru_cache_destroy_type = void_type.fn_type(&[i8_ptr_type.into()], false);
    let lru_cache_destroy = module.add_function("vp_lru_cache_destroy", lru_cache_destroy_type, None);

    // Unbounded cache functions
    let cache_create_type = i8_ptr_type.fn_type(&[], false);
    let cache_create = module.add_function("vp_cache_create", cache_create_type, None);

    let cache_get_type = i64_type.fn_type(&[i8_ptr_type.into(), i8_ptr_type.into(), context.i32_type().ptr_type(inkwell::AddressSpace::default()).into()], false);
    let cache_get = module.add_function("vp_cache_get", cache_get_type, None);

    let cache_set_type = void_type.fn_type(
        &[i8_ptr_type.into(), i8_ptr_type.into(), i64_type.into(), i64_type.into()],
        false,
    );
    let cache_set = module.add_function("vp_cache_set", cache_set_type, None);

    let cache_destroy_type = void_type.fn_type(&[i8_ptr_type.into()], false);
    let cache_destroy = module.add_function("vp_cache_destroy", cache_destroy_type, None);

    // Tuple creation for cache keys (single arg, multi-arg)
    let tuple_create1_type = i8_ptr_type.fn_type(&[i64_type.into()], false);
    let tuple_create1 = module.add_function("vp_tuple_create1", tuple_create1_type, None);

    let tuple_create2_type = i8_ptr_type.fn_type(&[i64_type.into(), i64_type.into()], false);
    let tuple_create2 = module.add_function("vp_tuple_create2", tuple_create2_type, None);

    Ok(MemoizationFunctions {
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
    let global_name = format!("__memo_cache_{}_{}", if is_lru { "lru" } else { "unbounded" }, func_name);

    let global = module.add_global(i8_ptr_type, Some(inkwell::AddressSpace::default()), &global_name);
    global.set_initializer(&i8_ptr_type.const_null());
    // Note: linkage set during optimization

    global.as_pointer_value()
}
