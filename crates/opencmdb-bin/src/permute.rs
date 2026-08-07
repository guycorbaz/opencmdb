//! Deterministic permutation sources for story 5.11b's arrival-order measurements.
//!
//! # Why this module exists at all, and why it is measured before it is used
//!
//! Story 5.11b asserts a property that is already true by CONSTRUCTION — `join` returns a
//! `BTreeMap`, `candidates` a `BTreeSet`, the witness is a `min` over a `BTreeSet`, the seen-window
//! a commutative `min`/`max` fold. Nothing downstream is expected to red. That is precisely the
//! condition under which a broken generator is invisible: a shuffle that returns its input makes
//! every consuming test compare a slice with itself and pass, and the whole story ships measuring
//! nothing.
//!
//! Measured at the story's validation, on the story's own code: with the enumerator replaced by
//! *"return the input"*, the consuming tests reddened **only** on their permutation-count
//! assertions — deleting those four lines left all of them GREEN. So this module carries its own
//! guards AND every caller states how many permutations it consumed. One without the other closes
//! nothing.
//!
//! # No new dependency
//!
//! `rand` is in no manifest in this workspace (verified over all four). It appears in `Cargo.lock`
//! only transitively, through `sqlx-postgres` and `surge-ping`. Taking it as a DIRECT dependency
//! would add a crate to a graph `deny.toml` audits, for a job that needs ~10 lines: at corpus scale
//! `n!` is at most 720, so enumeration is EXHAUSTIVE and no RNG is involved at all.

/// Every permutation of `items`, in lexicographic order of the underlying index vector.
///
/// The first element is the IDENTITY, which is a property callers must know rather than discover:
/// sampling `permutations(x).next()` turns an order-independence assertion into `f(x) == f(x)`, a
/// tautology that stays green under a genuinely order-dependent `f`. Measured at the story's
/// validation. A caller that samples must skip element 0 or assert its count.
///
/// Exhaustive by design and cheap where it is used: the committed replay streams carry 3 to 6
/// observations, so `n!` tops out at 720 — measured at 11.5 ms for the largest stream.
///
/// An empty slice yields exactly one permutation, the empty one, which is `0! = 1` and not a
/// degenerate case.
pub fn permutations<T: Clone>(items: &[T]) -> impl Iterator<Item = Vec<T>> {
    let owned = items.to_vec();
    index_permutations(owned.len())
        .into_iter()
        .map(move |order| order.into_iter().map(|i| owned[i].clone()).collect())
}

/// Every permutation of `0..n`, lexicographically, starting at the identity.
///
/// Indices are distinct by construction, so the lexicographic successor walk enumerates exactly
/// `n!` vectors with no duplicates — which is what lets a caller assert the count and have that
/// assertion mean something.
fn index_permutations(n: usize) -> Vec<Vec<usize>> {
    let mut current: Vec<usize> = (0..n).collect();
    let mut all = vec![current.clone()];
    while next_permutation(&mut current) {
        all.push(current.clone());
    }
    all
}

/// Advance `order` to its lexicographic successor in place; `false` when it is the last one.
fn next_permutation(order: &mut [usize]) -> bool {
    if order.len() < 2 {
        return false;
    }
    // The rightmost position whose successor is larger — the pivot the suffix will rotate around.
    let Some(pivot) = (0..order.len() - 1)
        .rev()
        .find(|i| order[*i] < order[i + 1])
    else {
        return false; // fully descending: the last permutation.
    };
    let successor = (pivot + 1..order.len())
        .rev()
        .find(|i| order[*i] > order[pivot])
        .expect("the pivot has a larger element to its right by its own definition");
    order.swap(pivot, successor);
    order[pivot + 1..].reverse();
    true
}

/// The FIXED seed sweep the reference-scale slice is fuzzed over.
///
/// Fixed, never clock-derived: a clock-derived seed makes a test that fails once a month and
/// reproduces never — the anecdote the story's AC4 exists to forbid.
///
/// 🔴 **Its provenance is guarded by [`tests::the_seed_sweep_is_the_fixed_range_it_claims_to_be`]
/// and by nothing else** — in particular NOT by the golden-value test, which pins `shuffled` at a
/// hardcoded seed and never reads this constant. Measured: before that guard existed, swapping this
/// sweep for a `SystemTime::now()`-derived one left the **entire suite green** over three
/// consecutive runs. Every other consumer stays green under a clock-derived sweep too, because
/// eight such seeds still shuffle, still reproduce within one process
/// (`shuffled(x, s) == shuffled(x, s)` holds for every `s`) and still number eight.
///
/// _(This doc claimed the golden-value test was the guard until story 5.11b's code review, where
/// two layers found it refuted by a test eighty lines below it in the same commit.)_
pub const SEED_SWEEP: std::ops::RangeInclusive<u64> = 0..=7;

/// `items` shuffled by a seeded Fisher-Yates, for slices too large to enumerate.
///
/// Deterministic in the seed and in nothing else: it reads no clock, no environment and no global
/// state, so a failure reproduces from the printed seed alone. The generator is a xorshift64 — the
/// modulo below is biased, and that is deliberate and harmless: this needs REPRODUCIBILITY, not
/// statistical quality, and a biased draw still reaches orders the identity does not.
pub fn shuffled<T: Clone>(items: &[T], seed: u64) -> Vec<T> {
    let mut out = items.to_vec();
    let mut state = XorShift64::new(seed);
    for i in (1..out.len()).rev() {
        let j = (state.next_u64() % (i as u64 + 1)) as usize;
        out.swap(i, j);
    }
    out
}

/// A xorshift64. Zero is the recurrence's one fixed point, and [`Self::new`] moves `seed = 0` off it.
///
/// ⚠️ **It does not move EVERY seed off it**, and the doc claimed otherwise until story 5.11b's code
/// review: `new` xors with a constant, so the seed EQUAL to that constant lands on state zero and
/// every draw is then `0`. That seed is `0x9E37_79B9_7F4A_7C15` — the golden-ratio constant, which
/// is exactly the value someone reaching for "a good seed" would reach for.
/// [`tests::the_dead_seed_is_named_and_outside_the_sweep`] pins the behaviour rather than pretending
/// it away: the shuffle still permutes (it degenerates to a fixed rotation), so the failure is
/// SILENT — `the_shuffle_preserves_the_multiset` passes on it.
struct XorShift64(u64);

impl XorShift64 {
    /// Seed the state, folding in an odd constant so that `seed = 0` is not the dead state.
    fn new(seed: u64) -> Self {
        Self(seed ^ 0x9E37_79B9_7F4A_7C15)
    }

    /// The next value, advancing the state.
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// `n!`, as an independent oracle — deliberately NOT computed by the code under test.
    fn factorial(n: usize) -> usize {
        // No `.max(1)`: an empty product is already 1, so the correction was unreachable — and an
        // oracle carrying a dead branch is harder to trust than one without. Caught at the review.
        (1..=n).product()
    }

    /// The enumerator produces exactly `n!` permutations, all distinct, at every size it is used at.
    ///
    /// The count is what every consuming test also asserts: a degenerate enumerator that returns
    /// its input is caught HERE by the count and NOWHERE else by behaviour, because the property
    /// being measured downstream is true by construction.
    #[test]
    fn the_enumerator_is_exhaustive_and_duplicate_free() {
        for n in 0..=6 {
            let items: Vec<usize> = (0..n).collect();
            let all: Vec<Vec<usize>> = permutations(&items).collect();
            assert_eq!(
                all.len(),
                factorial(n),
                "n = {n} must yield n! permutations"
            );
            let distinct: BTreeSet<Vec<usize>> = all.iter().cloned().collect();
            assert_eq!(distinct.len(), all.len(), "n = {n} yielded a duplicate");
            for one in &all {
                let seen: BTreeSet<usize> = one.iter().copied().collect();
                assert_eq!(
                    seen.len(),
                    n,
                    "a permutation must not drop or repeat an element"
                );
            }
        }
    }

    /// 🔴 Element 0 IS the identity — stated as a test because callers must not sample it.
    ///
    /// Measured at validation: sampling one permutation turns the story's AC1 into `join(o) ==
    /// join(o)`, which stays GREEN under a genuinely order-dependent `join`.
    #[test]
    fn the_first_permutation_is_the_identity_and_the_last_is_the_reversal() {
        let items = vec![10, 20, 30, 40];
        let all: Vec<Vec<i32>> = permutations(&items).collect();
        assert_eq!(all[0], items, "element 0 is the identity");
        assert_eq!(
            all[all.len() - 1],
            vec![40, 30, 20, 10],
            "lexicographic order ends reversed"
        );
    }

    /// The enumerator carries the VALUES, not just the shape.
    #[test]
    fn the_enumerator_permutes_the_actual_elements() {
        let all: Vec<Vec<&str>> = permutations(&["a", "b", "c"]).collect();
        assert_eq!(all.len(), 6);
        assert!(all.contains(&vec!["c", "a", "b"]));
        assert!(all.contains(&vec!["b", "c", "a"]));
    }

    /// AC4 guard 1 — the shuffle actually shuffles.
    ///
    /// Over the fixed sweep it produces at least two DISTINCT orders and at least one that is not
    /// the identity. 🔑 Measured: this reds when `shuffled` returns its input, and the story's
    /// AC1–AC3 all stay GREEN — so it closes this hole and no other.
    #[test]
    fn the_shuffle_is_not_the_identity_over_the_seed_sweep() {
        let items: Vec<usize> = (0..16).collect();
        let orders: BTreeSet<Vec<usize>> = SEED_SWEEP.map(|s| shuffled(&items, s)).collect();
        assert!(
            orders.len() >= 2,
            "the sweep must reach at least two distinct orders, reached {}",
            orders.len()
        );
        assert!(
            orders.iter().any(|o| *o != items),
            "every seed in the sweep returned the input unchanged"
        );
    }

    /// A shuffle drops nothing and invents nothing, at every seed in the sweep.
    #[test]
    fn the_shuffle_preserves_the_multiset() {
        let items: Vec<usize> = (0..16).collect();
        for seed in SEED_SWEEP {
            let mut got = shuffled(&items, seed);
            got.sort_unstable();
            assert_eq!(
                got, items,
                "seed {seed} changed the contents, not just the order"
            );
        }
    }

    /// The shuffle is reproducible from its seed alone.
    #[test]
    fn the_shuffle_reproduces_from_its_seed() {
        let items: Vec<usize> = (0..16).collect();
        for seed in SEED_SWEEP {
            assert_eq!(
                shuffled(&items, seed),
                shuffled(&items, seed),
                "seed {seed}"
            );
        }
    }

    /// 🔴 The SWEEP's own values, pinned — the guard a clock-derived seed actually trips on.
    ///
    /// **Measured, and it REFUTES the prediction this story inherited.** The golden-value test
    /// below was said to be what reds when the sweep is seeded from the clock. It is not: it pins
    /// `shuffled` at a HARDCODED seed, so it never reads [`SEED_SWEEP`] at all. Replacing the sweep
    /// with `now()..=now()+7` was measured leaving the ENTIRE suite green — every other consumer
    /// stays green too, because eight clock-derived seeds still shuffle, still reproduce within one
    /// process (`shuffled(x, s) == shuffled(x, s)` for every `s`), and still number eight.
    ///
    /// Reading the constant's VALUES is the only thing that closes it. The two tests are therefore
    /// complementary rather than redundant: this one pins the sweep, the next one pins the
    /// algorithm, and neither substitutes for the other.
    #[test]
    fn the_seed_sweep_is_the_fixed_range_it_claims_to_be() {
        assert_eq!(
            SEED_SWEEP.collect::<Vec<u64>>(),
            vec![0, 1, 2, 3, 4, 5, 6, 7],
            "the sweep must be FIXED — a clock-derived one fails once a month and reproduces never"
        );
    }

    /// ⚠️ The one seed that lands on the recurrence's fixed point, NAMED rather than pretended away.
    ///
    /// `XorShift64::new` xors the seed with `0x9E37_79B9_7F4A_7C15`, so that seed gives state zero
    /// and every draw is `0`. Found at story 5.11b's code review by two layers independently. The
    /// consequence is a SILENT degeneration, not a panic: Fisher-Yates with `j = 0` throughout still
    /// permutes — it produces a fixed rotation — so the multiset and reproducibility guards both
    /// pass on it. This test exists so that the day someone widens the sweep, the failure is a red
    /// here rather than a fuzz test that quietly stopped fuzzing.
    #[test]
    fn the_dead_seed_is_named_and_outside_the_sweep() {
        const DEAD_SEED: u64 = 0x9E37_79B9_7F4A_7C15;
        assert!(
            !SEED_SWEEP.contains(&DEAD_SEED),
            "the sweep must not contain the seed that kills the generator"
        );

        // 🔴 The load-bearing half, and the one that survives a change to the seeding constant:
        // NO seed in the sweep may land on the fixed point. The named constant above pins today's
        // dead seed; this loop pins the property that actually matters, and reds if the fold in
        // `new` is changed so that the dead seed moves INTO the sweep.
        for seed in SEED_SWEEP {
            let mut state = XorShift64::new(seed);
            assert_ne!(
                state.next_u64(),
                0,
                "seed {seed} lands on the recurrence's fixed point and shuffles nothing"
            );
        }

        let items: Vec<usize> = (0..8).collect();
        let degenerate = shuffled(&items, DEAD_SEED);
        assert_ne!(
            degenerate, items,
            "it is a rotation, not the identity — which is why the distinctness guard cannot see it"
        );
        let mut sorted = degenerate.clone();
        sorted.sort_unstable();
        assert_eq!(
            sorted, items,
            "and the multiset survives, which is why THAT guard cannot see it either"
        );
    }

    /// ⚠️ The enumerator's "duplicate-free" claim is about POSITIONS, not values.
    ///
    /// Fed a slice with equal elements it yields `n!` vectors of which only some are distinct —
    /// measured at the review: `[1, 1, 2]` gives 6 permutations and 3 distinct values. That is
    /// correct for what this module is for (it permutes ARRIVAL ORDER, and two observations are
    /// never equal because their `obs_id`s differ), and every caller here passes distinct elements.
    /// It is pinned so the claim is not read more widely than it holds.
    #[test]
    fn duplicate_elements_yield_repeated_permutations() {
        let all: Vec<Vec<i32>> = permutations(&[1, 1, 2]).collect();
        assert_eq!(all.len(), 6, "still 3! by position");
        let distinct: BTreeSet<Vec<i32>> = all.iter().cloned().collect();
        assert_eq!(distinct.len(), 3, "but only 3 distinct by value");
    }

    /// 🔴 AC4 guard 2 — the GOLDEN VALUE, which pins the seed AND the algorithm.
    ///
    /// It pins the ALGORITHM and one `(seed, input)` pair, and it is deliberately independent of
    /// [`SEED_SWEEP`]: it names seed 7 as a literal and never reads the constant. 🔴 **That
    /// independence is exactly why it does NOT guard the sweep's provenance** — that job belongs to
    /// [`the_seed_sweep_is_the_fixed_range_it_claims_to_be`], and the two are complementary rather
    /// than redundant. _(This doc asserted the opposite until story 5.11b's code review, and pointed
    /// at "the test above" positionally, which a later insertion had already made wrong.)_
    ///
    /// What it DOES catch: any change to the Fisher-Yates walk, to the xorshift, or to the seeding
    /// constant.
    ///
    /// The literal below is the measured output of THIS algorithm at THIS seed. Change either and
    /// this test reds, which is the entire point.
    #[test]
    fn a_golden_seed_and_input_pin_one_literal_output() {
        let items: Vec<usize> = (0..8).collect();
        assert_eq!(shuffled(&items, 7), GOLDEN_SEED_7);
    }

    /// The measured output of `shuffled(&(0..8).collect::<Vec<_>>(), 7)`, as a literal.
    const GOLDEN_SEED_7: [usize; 8] = [1, 3, 4, 6, 0, 5, 7, 2];
}
