//! Autogenerated weights for frame_system
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2021-08-04, STEPS: `[50, ]`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("heiko"), DB CACHE: 128

// Executed Command:
// ./target/release/parallel
// benchmark
// --chain=heiko
// --execution=wasm
// --wasm-execution=compiled
// --pallet=frame_system
// --extrinsic=*
// --steps=50
// --repeat=20
// --raw
// --output=./runtime/heiko/src/weights//frame_system.rs

#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::all)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for frame_system.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> frame_system::WeightInfo for WeightInfo<T> {
    fn remark(_b: u32) -> Weight {
        (1_683_000 as Weight)
    }
    fn remark_with_event(b: u32) -> Weight {
        (0 as Weight)
            // Standard Error: 0
            .saturating_add((1_000 as Weight).saturating_mul(b as Weight))
    }
    fn set_heap_pages() -> Weight {
        (2_418_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn set_storage(i: u32) -> Weight {
        (0 as Weight)
            // Standard Error: 1_000
            .saturating_add((847_000 as Weight).saturating_mul(i as Weight))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(i as Weight)))
    }
    fn kill_storage(i: u32) -> Weight {
        (798_000 as Weight)
            // Standard Error: 2_000
            .saturating_add((643_000 as Weight).saturating_mul(i as Weight))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(i as Weight)))
    }
    fn kill_prefix(p: u32) -> Weight {
        (11_012_000 as Weight)
            // Standard Error: 3_000
            .saturating_add((1_084_000 as Weight).saturating_mul(p as Weight))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(p as Weight)))
    }
}
