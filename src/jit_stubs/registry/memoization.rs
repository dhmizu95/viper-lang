//! Memoization JIT stub registration

use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::Module;

pub fn register_memoization_stubs(ee: &ExecutionEngine, module: &Module) {
    register_stubs!(ee, module, [
        // ARC key creation functions (1-8 parameters)
        "arc_key_create1" => super::super::memoization::arc_key_create1_stub,
        "arc_key_create2" => super::super::memoization::arc_key_create2_stub,
        "arc_key_create3" => super::super::memoization::arc_key_create3_stub,
        "arc_key_create4" => super::super::memoization::arc_key_create4_stub,
        "arc_key_create5" => super::super::memoization::arc_key_create5_stub,
        "arc_key_create6" => super::super::memoization::arc_key_create6_stub,
        "arc_key_create7" => super::super::memoization::arc_key_create7_stub,
        "arc_key_create8" => super::super::memoization::arc_key_create8_stub,
        
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

        // Backward compatibility wrappers
        "vp_tuple_create1" => super::super::memoization::vp_tuple_create1_stub,
        "vp_tuple_create2" => super::super::memoization::vp_tuple_create2_stub,
    ]);
}
