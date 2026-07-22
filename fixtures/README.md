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

Bytes are not the whole story: a file that hashes correctly can still be nonsense. So both halves
of `scenario/` are **discovered by walking** and read by the test suite, not merely hashed:

- every `scenario/traps/*.toml` is parsed, validated, and cross-checked against the observations it
  claims to judge;
- every `scenario/replay/*.jsonl` is parsed into its records, and the addresses it carries are
  checked against the synthetic-data rule below: every fact of every observation, and any IPv4 or
  MAC literal appearing in a control record's free text.

What that check does **not** cover, and knowingly: an observation's opaque `raw` payload, and a
hostname written in prose — a machine cannot recognise the second, and the first needs a stated
policy rather than an improvised one. Both are recorded in `deferred-work.md`.

Both walks are recursive and refuse a symlink or a foreign extension — a file the gate hashes and
the suite never reads is a lock with nothing behind it. `README.md` is exempt from both, at any
depth, exactly as it is exempt from the orphan rule above: the two gates must agree about what the
corpus may contain, or documenting a directory turns the test suite red.

## Never real network data

Synthetic values only: RFC 5737 documentation addresses (`192.0.2.0/24`), locally-administered
MACs, invented hostnames. This repository is public, and a real capture would carry MACs,
hostnames and the topology of someone's home. That is disqualifying, not a preference (D19).
