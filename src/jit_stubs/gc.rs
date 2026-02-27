// GC module stubs for JIT

pub extern "C" fn vp_gc_collect() {
    // In ARC system, this is a no-op
    // Mainly for statistics and compatibility
}

pub extern "C" fn vp_gc_disable() {
    // Placeholder - ARC doesn't have disable
}

pub extern "C" fn vp_gc_enable() {
    // Placeholder - ARC doesn't have enable
}

pub extern "C" fn vp_gc_is_enabled() -> i64 {
    1 // Always enabled in ARC
}

pub extern "C" fn vp_gc_get_count() -> i64 {
    0 // No collection cycles in ARC
}

pub extern "C" fn vp_gc_get_total_freed() -> i64 {
    0 // Would track in real implementation
}

pub extern "C" fn vp_gc_get_memory_usage() -> i64 {
    0 // Would track in real implementation
}

pub extern "C" fn vp_gc_set_threshold(_threshold: i64) {
    // Placeholder for future threshold-based collection
}

pub extern "C" fn vp_gc_get_threshold() -> i64 {
    0 // No threshold currently
}

pub extern "C" fn vp_gc_get_stats() -> *mut i8 {
    let stats = "GC Stats: collections=0, enabled=yes";
    let c_str = std::ffi::CString::new(stats).unwrap();
    c_str.into_raw()
}

pub extern "C" fn vp_gc_print_stats() {
    println!("GC Statistics:");
    println!("  Collections: 0");
    println!("  Enabled: yes");
    println!("  Total freed: 0 bytes");
}

pub extern "C" fn vp_gc_reset_stats() {
    // Nothing to reset in stub
}

pub extern "C" fn vp_gc_set_debug(_enabled: i64) {
    // Placeholder for debug mode
}

pub extern "C" fn vp_gc_run_finalizers() -> i64 {
    0 // ARC handles finalization automatically
}

pub extern "C" fn vp_gc_get_object_count() -> i64 {
    0 // Would track in real implementation
}

pub extern "C" fn vp_gc_get_pending_count() -> i64 {
    0 // No pending finalizers in stub
}

pub extern "C" fn vp_gc_break_cycles() -> i64 {
    0 // Placeholder for cycle detection
}
