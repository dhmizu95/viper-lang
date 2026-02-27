// Random module stubs for JIT - Phase 2
// PCG64 PRNG implementation

use std::cell::RefCell;

thread_local! {
    static PCG_STATE: RefCell<(u64, u64)> = RefCell::new((0, 0));
    static PCG_INITIALIZED: RefCell<bool> = RefCell::new(false);
}

fn pcg_init(seed: u64, seq: u64) {
    PCG_STATE.with(|state| {
        let mut s = state.borrow_mut();
        s.0 = 0;
        s.1 = (seq << 1) | 1;
        
        let old_state = s.0;
        s.0 = old_state.wrapping_mul(6364136223846793005).wrapping_add(s.1);
        
        s.0 = old_state.wrapping_add(seed);
        let old_state = s.0;
        s.0 = old_state.wrapping_mul(6364136223846793005).wrapping_add(s.1);
    });
    PCG_INITIALIZED.with(|init| *init.borrow_mut() = true);
}

fn pcg_next() -> u64 {
    PCG_STATE.with(|state| {
        let mut s = state.borrow_mut();
        let old_state = s.0;
        s.0 = old_state.wrapping_mul(6364136223846793005).wrapping_add(s.1);
        
        let xorshifted = (((old_state >> 18) ^ old_state) >> 27) as u32;
        let rot = (old_state >> 59) as u32;
        ((xorshifted >> rot) | (xorshifted << (rot.wrapping_neg() & 31))) as u64
    })
}

fn ensure_initialized() {
    PCG_INITIALIZED.with(|init| {
        if !*init.borrow() {
            let seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64;
            pcg_init(seed, 1);
        }
    });
}

#[no_mangle]
pub extern "C" fn vp_random_random() -> f64 {
    ensure_initialized();
    let r = pcg_next();
    (r >> 11) as f64 * (1.0 / 9007199254740992.0)
}

#[no_mangle]
pub extern "C" fn vp_random_randint(a: i64, b: i64) -> i64 {
    ensure_initialized();
    
    let (min, max) = if a > b { (b, a) } else { (a, b) };
    let range = (max - min + 1) as u64;
    let r = pcg_next();
    
    min + (r % range) as i64
}

#[no_mangle]
pub extern "C" fn vp_random_seed(seed: i64) {
    pcg_init(seed as u64, 1);
    PCG_INITIALIZED.with(|init| *init.borrow_mut() = true);
}

#[no_mangle]
pub extern "C" fn vp_random_seed_secure() {
    // Use random bytes from OS
    if let Ok(random_bytes) = std::fs::read("/dev/urandom") {
        if random_bytes.len() >= 16 {
            let seed = u64::from_ne_bytes([
                random_bytes[0], random_bytes[1], random_bytes[2], random_bytes[3],
                random_bytes[4], random_bytes[5], random_bytes[6], random_bytes[7],
            ]);
            let seq = u64::from_ne_bytes([
                random_bytes[8], random_bytes[9], random_bytes[10], random_bytes[11],
                random_bytes[12], random_bytes[13], random_bytes[14], random_bytes[15],
            ]);
            pcg_init(seed, seq);
            PCG_INITIALIZED.with(|init| *init.borrow_mut() = true);
            return;
        }
    }
    // Fallback to time-based seed
    vp_random_seed(std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as i64);
}

#[no_mangle]
pub extern "C" fn vp_random_choice(list: *mut std::ffi::c_void) -> i64 {
    // Simplified: would need ViperList integration
    // For now, return 0
    let _ = list;
    0
}

#[no_mangle]
pub extern "C" fn vp_random_shuffle(list: *mut std::ffi::c_void) {
    // Simplified: would need ViperList integration
    let _ = list;
}

#[no_mangle]
pub extern "C" fn vp_random_uniform(a: f64, b: f64) -> f64 {
    a + vp_random_random() * (b - a)
}

#[no_mangle]
pub extern "C" fn vp_random_gauss() -> f64 {
    // Box-Muller transform
    thread_local! {
        static HAS_SPARE: RefCell<bool> = RefCell::new(false);
        static SPARE: RefCell<f64> = RefCell::new(0.0);
    }

    // Check if we have a spare value
    let has_spare = HAS_SPARE.with(|h| *h.borrow());
    if has_spare {
        HAS_SPARE.with(|h| *h.borrow_mut() = false);
        return SPARE.with(|s| *s.borrow());
    }

    let mut u: f64;
    let mut v: f64;
    let mut s: f64;

    loop {
        u = vp_random_random() * 2.0 - 1.0;
        v = vp_random_random() * 2.0 - 1.0;
        s = u * u + v * v;
        if s < 1.0 && s > 0.0 {
            break;
        }
    }

    let multiplier = (-2.0 * s.ln() / s).sqrt();
    let spare = v * multiplier;

    SPARE.with(|s| *s.borrow_mut() = spare);
    HAS_SPARE.with(|h| *h.borrow_mut() = true);
    
    u * multiplier
}

#[no_mangle]
pub extern "C" fn vp_random_normal(mean: f64, stddev: f64) -> f64 {
    mean + stddev * vp_random_gauss()
}

#[no_mangle]
pub extern "C" fn vp_random_exp(lambd: f64) -> f64 {
    if lambd <= 0.0 {
        return 0.0;
    }
    -(1.0 - vp_random_random()).ln() / lambd
}

#[no_mangle]
pub extern "C" fn vp_random_sample(list: *mut std::ffi::c_void, k: i64) -> *mut std::ffi::c_void {
    // Simplified: would need ViperList integration
    let _ = list;
    let _ = k;
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn vp_random_bool(probability: f64) -> i64 {
    if vp_random_random() < probability { 1 } else { 0 }
}

#[no_mangle]
pub extern "C" fn vp_random_get_state() -> i64 {
    PCG_STATE.with(|state| state.borrow().0 as i64)
}

#[no_mangle]
pub extern "C" fn vp_random_set_state(state: i64) {
    PCG_STATE.with(|s| s.borrow_mut().0 = state as u64);
    PCG_INITIALIZED.with(|init| *init.borrow_mut() = true);
}

#[no_mangle]
pub extern "C" fn vp_random_is_initialized() -> i64 {
    PCG_INITIALIZED.with(|init| if *init.borrow() { 1 } else { 0 })
}

#[no_mangle]
pub extern "C" fn vp_random_getrandbits(k: i64) -> i64 {
    ensure_initialized();
    if k <= 0 {
        return 0;
    }
    if k >= 64 {
        return pcg_next() as i64;
    }
    let mask = (1u64 << k) - 1;
    (pcg_next() & mask) as i64
}

#[no_mangle]
pub extern "C" fn vp_random_randbytes(n: i64) -> *mut i8 {
    ensure_initialized();
    if n <= 0 {
        return std::ptr::null_mut();
    }
    
    let mut bytes = Vec::with_capacity(n as usize);
    for _ in 0..n {
        bytes.push((pcg_next() & 0xFF) as i8);
    }
    
    bytes.as_mut_ptr()
}
