//! Core JIT stub registration - Memory, GC, and Tagged Int functions

use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::Module;

pub fn register_core_stubs(ee: &ExecutionEngine, module: &Module) {
    // Memory functions (low-level allocation, not GC-managed)
    register_stubs!(ee, module, [
        "vp_malloc" => super::super::memory::vp_malloc,
        "vp_free" => super::super::memory::vp_free,
    ]);

    // GC functions (ARC system - these are no-ops for compatibility)
    register_stubs!(ee, module, [
        "vp_gc_collect" => super::super::gc::vp_gc_collect,
        "vp_gc_disable" => super::super::gc::vp_gc_disable,
        "vp_gc_enable" => super::super::gc::vp_gc_enable,
        "vp_gc_is_enabled" => super::super::gc::vp_gc_is_enabled,
        "vp_gc_get_count" => super::super::gc::vp_gc_get_count,
        "vp_gc_get_total_freed" => super::super::gc::vp_gc_get_total_freed,
        "vp_gc_get_memory_usage" => super::super::gc::vp_gc_get_memory_usage,
        "vp_gc_set_threshold" => super::super::gc::vp_gc_set_threshold,
        "vp_gc_get_threshold" => super::super::gc::vp_gc_get_threshold,
        "vp_gc_get_stats" => super::super::gc::vp_gc_get_stats,
        "vp_gc_print_stats" => super::super::gc::vp_gc_print_stats,
        "vp_gc_reset_stats" => super::super::gc::vp_gc_reset_stats,
        "vp_gc_set_debug" => super::super::gc::vp_gc_set_debug,
        "vp_gc_run_finalizers" => super::super::gc::vp_gc_run_finalizers,
        "vp_gc_get_object_count" => super::super::gc::vp_gc_get_object_count,
        "vp_gc_get_pending_count" => super::super::gc::vp_gc_get_pending_count,
        "vp_gc_break_cycles" => super::super::gc::vp_gc_break_cycles,
    ]);

    // Reference counting functions
    register_stubs!(ee, module, [
        "vp_retain" => super::super::lists::vp_retain_stub,
        "vp_retain_local" => super::super::lists::vp_retain_local_stub,
        "vp_release" => super::super::lists::vp_release_stub,
    ]);

    // Tagged Int runtime functions
    register_stubs!(ee, module, [
        "tagged_int_add" => super::super::tagged_int::tagged_int_add,
        "tagged_int_sub" => super::super::tagged_int::tagged_int_sub,
        "tagged_int_mul" => super::super::tagged_int::tagged_int_mul,
        "tagged_int_div" => super::super::tagged_int::tagged_int_div,
        "tagged_int_mod" => super::super::tagged_int::tagged_int_mod,
        "tagged_int_pow" => super::super::tagged_int::tagged_int_pow,
        "tagged_int_neg" => super::super::tagged_int::tagged_int_neg,
        "tagged_int_eq" => super::super::tagged_int::tagged_int_eq,
        "tagged_int_lt" => super::super::tagged_int::tagged_int_lt,
        "tagged_int_gt" => super::super::tagged_int::tagged_int_gt,
        "tagged_int_cmp" => super::super::tagged_int::tagged_int_cmp,
        "tagged_int_from_i64" => super::super::tagged_int::tagged_int_from_i64,
        "tagged_int_from_str" => super::super::tagged_int::tagged_int_from_str,
        "tagged_int_to_str" => super::super::tagged_int::tagged_int_to_str,
        "tagged_int_print" => super::super::tagged_int::tagged_int_print,
        "tagged_int_free" => super::super::tagged_int::tagged_int_free,
        "tagged_int_bitand" => super::super::tagged_int::tagged_int_bitand,
        "tagged_int_bitor" => super::super::tagged_int::tagged_int_bitor,
        "tagged_int_bitxor" => super::super::tagged_int::tagged_int_bitxor,
        "tagged_int_lshift" => super::super::tagged_int::tagged_int_lshift,
        "tagged_int_rshift" => super::super::tagged_int::tagged_int_rshift,
    ]);
}
