//! Decorative randomness (house lights, fire particles), from the engine's
//! unsynced RNG as the Lua widgets' `math.random`.

pub use spring::random::unsynced::float;

/// Screenshot builds restart the unsynced RNG at `seed` before drawing a batch,
/// so the lights and flames in a screenshot are the same every run. Seeding
/// also resets the engine's own unsynced draws, which is why release builds
/// never do it.
pub fn pin_for_screenshots(seed: i32) {
    #[cfg(feature = "e2e")]
    spring::random::unsynced::seed(seed);
    #[cfg(not(feature = "e2e"))]
    let _ = seed;
}
