//! Memoization JIT stub registration

use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::Module;

pub fn register_memoization_stubs(ee: &ExecutionEngine, module: &Module) {
    register_stubs!(ee, module, [
        // LRU Cache functions
        "vp_lru_cache_create" => super::super::memoization::vp_lru_cache_create_stub,
        "vp_lru_cache_get" => super::super::memoization::vp_lru_cache_get_stub,
        "vp_lru_cache_set" => super::super::memoization::vp_lru_cache_set_stub,
        "vp_lru_cache_destroy" => super::super::memoization::vp_lru_cache_destroy_stub,
        
        // Unbounded Cache functions
        "vp_cache_create" => super::super::memoization::vp_cache_create_stub,
        "vp_cache_get" => super::super::memoization::vp_cache_get_stub,
        "vp_cache_set" => super::super::memoization::vp_cache_set_stub,
        "vp_cache_destroy" => super::super::memoization::vp_cache_destroy_stub,
        
        // Tuple creation for cache keys
        "vp_tuple_create1" => super::super::memoization::vp_tuple_create1_stub,
        "vp_tuple_create2" => super::super::memoization::vp_tuple_create2_stub,
    ]);
}
