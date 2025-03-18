use crate::{
    context::Context,
    hashing::hash_str,
    trace,
    trait_map::TraitMap,
    context::DataPlugin
};
use rand::{
    distr::{
        uniform::{SampleRange, SampleUniform},
        weighted::{
            WeightedIndex,
            Weight
        }
    },
    prelude::Distribution,
    Rng,
    SeedableRng,
};
use std::any::Any;

// pub struct RngId {
//     idx: usize,
//     seed: u64,
//     name: &'static str,
//     rng: Box<dyn SeedableRng<Seed=u64>>,
// }
pub trait RngId: Any  {
    #![allow(non_upper_case_globals)]
    const new: &'static dyn Fn(u64) -> Self;
    const name: &'static str;
    type RngType: SeedableRng;
    fn rng(&mut self) -> &mut Self::RngType;
}

struct RngPlugin {
    base_seed: u64,
    rng_map  : TraitMap
}

impl RngPlugin {
    fn with_seed(seed : u64) -> Self {
        RngPlugin{
            base_seed: seed,
            rng_map  : TraitMap::new()
        }
    }
    fn clear(&mut self) {
        self.rng_map.clear();
    }

    pub fn get_rng<R: RngId>(&mut self) -> &mut R::RngType {
        if !self.rng_map.contains_key::<R>() {
            let base_seed = self.base_seed;
            let seed_offset = base_seed.wrapping_add(hash_str(R::name));
            self.rng_map.insert(R::new(seed_offset));
        }

        self.rng_map.get_mut::<R>().unwrap().rng()
    }
}

impl DataPlugin for RngPlugin {
    #[allow(non_upper_case_globals)]
    const new: &'static dyn Fn() -> Self = &|| {
        RngPlugin{
            base_seed: 0,
            rng_map: TraitMap::new()
        }
    };
}

/// Gets a mutable reference to the random number generator associated with the given
/// `RngId`.
// This is a private free function so that it's not leaked to the public API.
fn get_rng<R: RngId>(context: &mut Context) -> &mut R::RngType {
    let rng_container = context
        .get_data_container_mut::<RngPlugin>();

    rng_container.get_rng::<R>()
}

pub trait ContextRandomExt {
    fn init_random(&mut self, base_seed: u64);

    /// Gets a random sample from the random number generator associated with the given
    /// `RngId` by applying the specified sampler function. If the Rng has not been used
    /// before, one will be created with the base seed you defined in `set_base_random_seed`.
    /// Note that this will panic if `set_base_random_seed` was not called yet.
    fn sample<R: RngId + 'static, T>(
        &mut self,
        sampler: impl FnOnce(&mut R::RngType) -> T,
    ) -> T;

    /// Gets a random sample from the specified distribution using a random number generator
    /// associated with the given `RngId`. If the Rng has not been used before, one will be
    /// created with the base seed you defined in `set_base_random_seed`.
    /// Note that this will panic if `set_base_random_seed` was not called yet.
    fn sample_distr<R: RngId + 'static, T>(
        &mut self,
        distribution: impl Distribution<T>,
    ) -> T
    where
        R::RngType: Rng;

    /// Gets a random sample within the range provided by `range`
    /// using the generator associated with the given `RngId`.
    /// Note that this will panic if `set_base_random_seed` was not called yet.
    fn sample_range<R: RngId + 'static, S, T>(&mut self, range: S) -> T
    where
        R::RngType: Rng,
        S: SampleRange<T>,
        T: SampleUniform;

    /// Gets a random boolean value which is true with probability `p`
    /// using the generator associated with the given `RngId`.
    /// Note that this will panic if `set_base_random_seed` was not called yet.
    fn sample_bool<R: RngId + 'static>(&mut self, p: f64) -> bool
    where
        R::RngType: Rng;

    /// Draws a random entry out of the list provided in `weights`
    /// with the given weights using the generator associated with the
    /// given `RngId`.  Note that this will panic if
    /// `set_base_random_seed` was not called yet.
    fn sample_weighted<R: RngId + 'static, T>(&mut self, weights: &[T]) -> usize
    where
        R::RngType: Rng,
        T: Clone + Default + SampleUniform + for<'a> std::ops::AddAssign<&'a T> + PartialOrd + Weight;
}

impl ContextRandomExt for Context {
    /// Initializes the `RngPlugin` data container to store rngs as well as a base
    /// seed. Note that rngs are created lazily when `get_rng` is called.
    fn init_random(&mut self, base_seed: u64) {
        trace!("initializing random module");
        let rng_container = self.get_data_container_mut::<RngPlugin>();
        rng_container.base_seed = base_seed;

        // Clear any existing Rngs to ensure they get re-seeded when `get_rng` is called
        rng_container.clear();
    }

    fn sample<R: RngId + 'static, T>(
        &mut self,
        sampler: impl FnOnce(&mut R::RngType) -> T,
    ) -> T {
        let rng = get_rng::<R>(self);
        sampler(rng)
    }

    fn sample_distr<R: RngId + 'static, T>(
        &mut self,
        distribution: impl Distribution<T>,
    ) -> T
    where
        R::RngType: Rng,
    {
        let rng = get_rng::<R>(self);
        distribution.sample::<R::RngType>(rng)
    }

    fn sample_range<R: RngId + 'static, S, T>(&mut self, range: S) -> T
    where
        R::RngType: Rng,
        S: SampleRange<T>,
        T: SampleUniform,
    {
        self.sample::<R, T>(|rng| rng.random_range(range))
    }

    fn sample_bool<R: RngId + 'static>(&mut self, p: f64) -> bool
    where
        R::RngType: Rng,
    {
        self.sample::<R, bool>(|rng| rng.random_bool(p))
    }

    fn sample_weighted<R: RngId + 'static, T>(&mut self, weights: &[T]) -> usize
    where
        R::RngType: Rng,
        T: Clone + Default + SampleUniform + for<'a> std::ops::AddAssign<&'a T> + PartialOrd + Weight,
    {
        let index = WeightedIndex::new(weights).unwrap();
        let rng = get_rng::<R>(self);
        index.sample(rng)
    }
}


#[macro_export]
macro_rules! define_rng {
    ($random_id:ident) => {
        struct $random_id{
            rng: $crate::rand::rngs::StdRng,
        }

        impl $crate::random::RngId for $random_id {
            #![allow(non_upper_case_globals)]
            // TODO(ryl8@cdc.gov): This is hardcoded to StdRng; we should replace this
            type RngType = $crate::rand::rngs::StdRng;
            const name: &'static str = &stringify!($random_id);
            const new: &'static dyn Fn(u64) -> Self = &|seed| {
                use $crate::rand::SeedableRng;
                Self {
                    rng: $crate::rand::rngs::StdRng::seed_from_u64(seed),
                }
            };

            fn rng(&mut self) -> &mut Self::RngType {
                &mut self.rng
            }
        }
    };
    ($random_id:ident, $rng_type:ty) => {
        struct $random_id{
            rng: $rng_type,
        }

        impl $crate::random::RngId for $random_id {
            #![allow(non_upper_case_globals)]
            // TODO(ryl8@cdc.gov): This is hardcoded to StdRng; we should replace this
            type RngType = $rng_type;
            const name: &'static str = &stringify!($random_id);
            const new: &'static dyn Fn(u64) -> Self = &|seed| {
                use $crate::rand::SeedableRng;
                Self {
                    rng: <$rng_type>::seed_from_u64(seed),
                }
            };

            fn rng(&mut self) -> &mut Self::RngType {
                &mut self.rng
            }
        }
    };
    ($random_id:ident, $rng_type:ty, $seed:literal) => {
        struct $random_id{
            rng: $rng_type,
        }

        impl $crate::random::RngId for $random_id {
            #![allow(non_upper_case_globals)]
            // TODO(ryl8@cdc.gov): This is hardcoded to StdRng; we should replace this
            type RngType = $rng_type;
            const name: &'static str = &stringify!($random_id);
            const new: &'static dyn Fn(u64) -> Self = &|_| {
                use $crate::rand::SeedableRng;
                Self {
                    rng: <$rng_type>::seed_from_u64($seed),
                }
            };

            fn rng(&mut self) -> &mut Self::RngType {
                &mut self.rng
            }
        }
    };
}
#[allow(unused_imports)]
pub use define_rng;

#[cfg(test)]
mod test {
    use crate::context::{Context, DataPlugin};
    use crate::random::ContextRandomExt;
    use rand::RngCore;
    use rand::{distr::weighted::WeightedIndex, prelude::Distribution};

    define_rng!(FooRng);
    define_rng!(BarRng);

    #[test]
    fn get_rng_basic() {
        let mut context = Context::new();
        context.init_random(42);

        assert_ne!(
            context.sample::<FooRng, _>(RngCore::next_u64),
            context.sample::<FooRng, _>(RngCore::next_u64)
        );
    }

    #[test]
    fn multiple_rng_types() {
        let mut context = Context::new();
        context.init_random(42);

        assert_ne!(
            context.sample::<FooRng, _>(RngCore::next_u64),
            context.sample::<BarRng, _>(RngCore::next_u64)
        );
    }

    #[test]
    fn reset_seed() {
        let mut context = Context::new();
        context.init_random(42);

        let run_0 = context.sample::<FooRng, _>(RngCore::next_u64);
        let run_1 = context.sample::<FooRng, _>(RngCore::next_u64);

        // Reset with same seed, ensure we get the same values
        context.init_random(42);
        assert_eq!(run_0, context.sample::<FooRng, _>(RngCore::next_u64));
        assert_eq!(run_1, context.sample::<FooRng, _>(RngCore::next_u64));

        // Reset with different seed, ensure we get different values
        context.init_random(88);
        assert_ne!(run_0, context.sample::<FooRng, _>(RngCore::next_u64));
        assert_ne!(run_1, context.sample::<FooRng, _>(RngCore::next_u64));
    }

    struct SamplerData(WeightedIndex<f64>);
    impl DataPlugin for SamplerData{
        const new: &'static dyn Fn() -> Self = &||{
            let wi = WeightedIndex::new(vec![1.0]).unwrap();
            SamplerData(wi)
        };
    }

    #[test]
    fn sampler_function_closure_capture() {
        let mut context = Context::new();
        context.init_random(42);
        // Initialize weighted sampler
        let wi = WeightedIndex::new(vec![1.0, 2.0]).unwrap();
        *context.get_data_container_mut() = SamplerData(wi.clone());

        let n_samples = 3000;
        let mut zero_counter = 0;
        for _ in 0..n_samples {
            let sample = context.sample::<FooRng, _>(|rng| wi.sample(rng));
            if sample == 0 {
                zero_counter += 1;
            }
        }
        assert!((zero_counter - 1000_i32).abs() < 50);
    }

    #[test]
    fn sample_distribution() {
        let mut context = Context::new();
        context.init_random(42);

        // Initialize weighted sampler
        let wi = WeightedIndex::new(vec![1.0, 2.0]).unwrap();
        *context.get_data_container_mut::<SamplerData>() = SamplerData(wi.clone());

        let n_samples = 3000;
        let mut zero_counter = 0;
        for _ in 0..n_samples {
            let sample = context.sample_distr::<FooRng, usize>(&wi);
            if sample == 0 {
                zero_counter += 1;
            }
        }
        assert!((zero_counter - 1000_i32).abs() < 50);
    }

    #[test]
    fn sample_range() {
        let mut context = Context::new();
        context.init_random(42);
        let result = context.sample_range::<FooRng, _, i32>(0..10);
        assert!((0..10).contains(&result));
    }

    #[test]
    fn sample_bool() {
        let mut context = Context::new();
        context.init_random(42);
        let _r: bool = context.sample_bool::<FooRng>(0.5);
    }

    #[test]
    fn sample_weighted() {
        let mut context = Context::new();
        context.init_random(42);
        let r: usize = context.sample_weighted::<FooRng, _>(&[0.1, 0.3, 0.4]);
        assert!(r < 3);
    }
}
