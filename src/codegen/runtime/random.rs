//! Random number generation runtime function declarations for Viper code generation

use inkwell::context::Context;
use inkwell::module::Module;

/// Declare random runtime functions
pub fn declare_random_functions<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> crate::codegen::Result<()> {
    let ptr_type = context.ptr_type(inkwell::AddressSpace::default());
    let i64_type = context.i64_type();
    let f64_type = context.f64_type();

    // vp_random_random - Generate random float in [0.0, 1.0)
    // Signature: double vp_random_random(void)
    let random_type = f64_type.fn_type(&[], false);
    module.add_function("vp_random_random", random_type, None);

    // vp_random_randint - Generate random integer in [a, b]
    // Signature: int64_t vp_random_randint(int64_t a, int64_t b)
    let randint_type = i64_type.fn_type(&[i64_type.into(), i64_type.into()], false);
    module.add_function("vp_random_randint", randint_type, None);

    // vp_random_seed - Seed the random number generator
    // Signature: void vp_random_seed(int64_t seed)
    let seed_type = context.void_type().fn_type(&[i64_type.into()], false);
    module.add_function("vp_random_seed", seed_type, None);

    // vp_random_seed_secure - Seed from secure source
    // Signature: void vp_random_seed_secure(void)
    let seed_secure_type = context.void_type().fn_type(&[], false);
    module.add_function("vp_random_seed_secure", seed_secure_type, None);

    // vp_random_choice - Choose random element from list
    // Signature: int64_t vp_random_choice(ViperList* list)
    let choice_type = i64_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_random_choice", choice_type, None);

    // vp_random_shuffle - Shuffle list in place
    // Signature: void vp_random_shuffle(ViperList* list)
    let shuffle_type = context.void_type().fn_type(&[ptr_type.into()], false);
    module.add_function("vp_random_shuffle", shuffle_type, None);

    // vp_random_sample - Sample k unique elements from population
    // Signature: ViperList* vp_random_sample(ViperList* population, int64_t k)
    let sample_type = ptr_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_random_sample", sample_type, None);

    // vp_random_gauss - Gaussian distribution
    // Signature: double vp_random_gauss(double mu, double sigma)
    let gauss_type = f64_type.fn_type(&[f64_type.into(), f64_type.into()], false);
    module.add_function("vp_random_gauss", gauss_type, None);

    // vp_random_uniform - Uniform distribution
    // Signature: double vp_random_uniform(double a, double b)
    let uniform_type = f64_type.fn_type(&[f64_type.into(), f64_type.into()], false);
    module.add_function("vp_random_uniform", uniform_type, None);

    // vp_random_getstate - Get internal state
    // Signature: ViperTuple* vp_random_getstate(void)
    let getstate_type = ptr_type.fn_type(&[], false);
    module.add_function("vp_random_getstate", getstate_type, None);

    // vp_random_setstate - Restore internal state
    // Signature: void vp_random_setstate(ViperTuple* state)
    let setstate_type = context.void_type().fn_type(&[ptr_type.into()], false);
    module.add_function("vp_random_setstate", setstate_type, None);

    Ok(())
}
