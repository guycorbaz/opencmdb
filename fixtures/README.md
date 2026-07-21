# fixtures/

**These files are a SPEC, not test data.**

They live at the workspace root, outside every crate, on purpose (D56). A file under `tests/`
is read as the property of the test, and the first reflex of someone refactoring the engine is
to adjust it until the red goes away. **A red repairable by editing the spec is not a gate — it
is a negotiation** (D45).

At the root, editing a file here is a commit that says *"I am changing the spec"*, not *"I am
fixing a test"*. Review it that way.

## Layout

| Directory | Holds | Rots? |
|---|---|---|
| `scenario/` | synthetic traps, written to trap a named case — they prove the **engine** | **No.** They are right or wrong. |
| `capture/` | real, version-tagged, dated source payloads — they prove the **parser** | **Yes.** A re-capture job diffs them against the live schema. |

The split is not tidying. It is the domain of definition of a destructive tool: the re-capture
job must be structurally unable to reach `scenario/`, or the day a source changes its schema it
would offer to "update" a synthetic trap and **rewrite the truth to make the gate pass**.

## The lock

Every artefact is listed in `MANIFEST.toml` with its sha256, and `cargo xtask ci` checks **both
directions** — the corpus is frozen only when both hold:

- **Edited** — a listed artefact whose bytes changed is RED. The single repair is a deliberate
  bump, which reads in review as *"I am changing the spec"*.
- **Added** — a file present here but absent from the manifest is RED. Without this the guarantee
  would only be *"listed files are unchanged"*, which is a different and much weaker claim: a new
  trap file would be neither hashed nor noticed.

Exempt from the second rule, deliberately and narrowly: `MANIFEST.toml` itself (a lock cannot list
itself) and `README.md` files (prose about the corpus, not artefacts of it).

Bytes are not the whole story: a trap file that hashes correctly can still be nonsense. Every
`scenario/traps/*.toml` is therefore discovered and parsed by the test suite — validated, and
cross-checked against the observations it claims to judge.

## Never real network data

Synthetic values only: RFC 5737 documentation addresses (`192.0.2.0/24`), locally-administered
MACs, invented hostnames. This repository is public, and a real capture would carry MACs,
hostnames and the topology of someone's home. That is disqualifying, not a preference (D19).
