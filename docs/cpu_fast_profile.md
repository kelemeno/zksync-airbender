# Experimental CPU fast profile (base layer)

A benchmark experiment for proving small batches quickly on a CPU-only machine. On an Apple M4 Pro (64 GB), a warm base proof of a 1–5 transfer custom STF batch takes about:
- 4 s and 13 GB unsigned;
- 20 s and 36 GB with secp256k1 ECDSA via the bigint delegation.

Stock v3 takes about 148 s and 65 GB for the same batch. All proofs pass `cli verify`.

**Not reviewed.** Sizes, ROM size and WHIR parameters are a performance experiment. The security level setting is unchanged (Sec100) and query counts follow this repo's documented rule, but the Airbender team has not reviewed any of it. This profile is **base layer only**: the recursion verifier programs (1.8–3.5 MB) do not fit the 512 KB ROM.

## Changes
| Change | Where |
|---|---|
| Buffered artifact JSON I/O: 23–37 s → ~1 s per proof | `prover_pipeline/src/lib.rs` (`serialize_to_file`, `deserialize_from_file`) |
| Warm prover: setups, setup commitments and twiddles cached per program binary across proofs in one process (`cli prove-batch`) | `program_prover/src/warm.rs`, memo sites in `program_prover/src/unrolled.rs` |
| ROM 4 MB → 512 KB, which shrinks every family's decoder table from 2^20 to 2^17 rows | `common_constants/src/rom.rs`, `riscv_common/src/lds/*/memory.x` |
| jump/branch/slt uses the 2^7-row `ConditionalJmpBranchSltUnified` table plus one `U16GetSign` lookup, instead of the 2^22-row `ConditionalJmpBranchSlt` | `cs/src/gkr_circuits/jump_branch_slt_family/circuit.rs` |
| Chunk sizes at their floors (lookup tables + decoder must fit one trace): 2^19 for add/sub, jump/branch/slt, mul/div, word ld/st, inits/teardowns, BLAKE2 and BLAKE2-G; 2^20 for shift; 2^21 for bigint and keccak; 2^22 for byte ld/st | `circuit_defs/**/src/lib.rs` |
| Fast WHIR schedule: 16-bit PoW per round, intermediate LDE ×16/×64/×256/…, queries = ceil(1.2 × (100 − pow) / rate_bits), new 2^19 entry. The shipped table is kept as `config_for_100_bits_under_pessimistic_conjecture_original` | `prover/src/gkr/prover_config/example_configs.rs` |
| Delegation memory caps committed in the verifier's `DELEGATION_CIRCUITS_SETUP_PARAMS` order (it was type-sorted, which disagrees with the verifier when BLAKE2-G is used together with bigint or keccak) | `program_prover/src/unrolled.rs`, `circuit_defs/trace_and_split/src/lib.rs` |
| Regenerated artifacts: circuit layouts and witness functions, `cs/compiled_circuits/*`, generated verifiers, delegation setup caps | `circuit_defs/**/generated`, `cs/compiled_circuits`, `verifier/src/generated`, `circuit_defs/setups/generated` |

The bigint lookup-index fix is a separate commit on this branch and applies to stock v3 on its own.

## Regenerating after a size or circuit change
`recreate_verifiers.sh` cannot be used: the `cs` layout tests hard-code 2^24. Instead:
1. Run `cargo test --lib generate` for each changed circuit crate. This writes `generated/layout.json` and `witness_generation_fn.rs`.
2. Copy each `layout.json` to `cs/compiled_circuits/<verifier name>_layout_gkr.json`. inits/teardowns uses `..._layout_no_caches_gkr.json`.
3. Run `cargo test -p verifier_generator --test generate_verifiers -- --exact <names>`.
4. For delegation circuits, also run `cd circuit_defs/setups && cargo test --lib generate_delegation_circuits_artifacts`. It regenerates the hard-coded delegation setup caps.
5. Rebuild: `cargo build --release -p cli`.

Guests must be linked with this tree's `riscv_common` (512 KB ROM memory map).
