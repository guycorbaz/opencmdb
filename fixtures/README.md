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
| `scenario/` | synthetic traps, written to trap a named case — they prove the **engine** (`scenario/wire/` is the one exception: a synthetic wire SPEC that proves the **parser**, run by Epic 11's harness — it lives here rather than in `capture/` because it is a spec, not a rotting capture; story 4.18) | **No.** They are right or wrong. |
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

Bytes are not the whole story: a file that hashes correctly can still be nonsense. So the
`replay/` and `traps/` halves of `scenario/` are **discovered by walking** and read by the test
suite, not merely hashed (`scenario/wire/` sits outside the walks — its dedicated shape test is
its reader and its privacy coverage; see its README):

- every `scenario/traps/*.toml` is parsed, validated, and cross-checked against the observations it
  claims to judge — and its RAW TEXT, header comments included, is read before TOML parsing and
  checked against the synthetic-data rule below. Comments have to be checked before the parser
  throws them away, which is why the scan reads bytes rather than parsed values;
- every `scenario/replay/*.jsonl` is parsed into its records, and the addresses it carries are
  checked against the synthetic-data rule below: every fact of every observation, an observation's
  opaque `raw` payload, and any IPv4 or MAC literal appearing in a control record's free text.

What that check does **not** cover, and knowingly: the free author-typed `vendor` and `peer_port`
strings of the `OuiVendor` and `Uplink` facts; every `README.md`, exempt at any depth by the rule
below — this file included; IPv6 literals, which no branch attempts; and several address notations
the tokenizer cannot read (zero-padded IPv4, the Cisco dotted and bare MAC forms, an address glued
to a hexdigit). All of those are recorded in `deferred-work.md` with an owner.

One is recorded WITHOUT an owner, because it is not mechanically closable: a hostname written in
prose. A machine cannot tell an invented name from a captured one, so no rule is waiting to be
written — that hole is held by review or not at all.

Honesty about coverage runs the other way too: `raw` is scanned by the same rule as everything
else, but exactly ONE committed observation carries a non-null `raw` and it holds no address, so
today that leg proves nothing about the corpus. It is a rule with a permanent test behind it, not
a claim that anything was checked.

Both walks are recursive and refuse a symlink or a foreign extension — a file the gate hashes and
the suite never reads is a lock with nothing behind it. Neither descends into a dot-entry: tooling
scratch such as `.claude/` is not corpus, and without the skip the first file a tool wrote there
would have made the suite accuse the corpus of a defect it does not have. The cost is named — a
`.hidden.toml` would no longer be seen, which is acceptable because the corpus never hides an
artefact and `MANIFEST.toml` lists every one by its visible name. `README.md` is exempt from both,
at any depth, exactly as it is exempt from the orphan rule above: the two gates must agree about
what the corpus may contain, or documenting a directory turns the test suite red.

## Never real network data

Synthetic values only: RFC 5737 documentation addresses (`192.0.2.0/24`), locally-administered
UNICAST MACs, invented hostnames. This repository is public, and a real capture would carry MACs,
hostnames and the topology of someone's home. That is disqualifying, not a preference (D19).

"Locally administered" is necessary and not sufficient: a MULTICAST MAC is refused whatever its
U/L bit says, because a multicast address names no interface and its bytes can come from a real
one — an IPv6 solicited-node address such as `33:33:ff:xx:xx:xx` embeds real interface-identifier
bytes while wearing a set U/L bit. One narrow exception is admitted by name and 5-octet exact: the
IANA VRRP IPv4 virtual-router block `00:00:5e:00:01:xx`, a protocol address identical on every
VRRP deployment with that VRID — the MAC analog of an RFC 5737 address.
