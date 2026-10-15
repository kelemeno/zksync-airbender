//! Benchmark experiment: a per-thread "warm prover" cache. Values that depend only on the
//! program binary and circuit sizes (circuit setups, setup commitments, twiddles) are built
//! on the first proof and reused by later proofs in the same process (e.g. `cli prove-batch`).
//! Entries are leaked on purpose: a proving server keeps them resident for its lifetime.
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static MAP: RefCell<HashMap<String, &'static dyn Any>> = RefCell::new(HashMap::new());
}

/// Returns the cached value for `key`, building it with `f` on first use.
pub fn memo<T: 'static>(key: String, f: impl FnOnce() -> T) -> &'static T {
    if let Some(v) = MAP.with(|m| m.borrow().get(&key).copied()) {
        return v.downcast_ref::<T>().expect("warm cache type mismatch");
    }
    let v: &'static T = Box::leak(Box::new(f()));
    MAP.with(|m| m.borrow_mut().insert(key, v as &'static dyn Any));
    v
}

/// FNV-1a over the program image, used to key per-binary entries.
pub fn hash_words(parts: &[&[u32]]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for p in parts {
        for w in p.iter() {
            for b in w.to_le_bytes() {
                h ^= b as u64;
                h = h.wrapping_mul(0x100000001b3);
            }
        }
        h ^= 0xff;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}
