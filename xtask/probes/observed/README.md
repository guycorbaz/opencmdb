# `observed-immutable` probe corpus — story 6.3

Each file here is planted, one at a time, under a scratch `crates/` and run through
**`gate_observed_immutable` end to end**. Its verdict is pinned in `OBSERVED_PROBES`
(`xtask/src/main.rs`) and asserted in **both directions**, because a gate that silently WIDENS is
a gate whose stated limits have gone stale.

The verdicts are **located** — a file *and a line* — not booleans. Story 5.12 learned why the hard
way: it shipped an offset→line map counting characters where the caller indexed bytes, and **no
boolean probe could see it**, because the write still reddened, only at the wrong line.
*A pinned boolean proves THAT a gate fires and never WHERE.*

## The eleven that must RED

| probe | the shape it carries |
|---|---|
| `o01_plain_update.rs` | the plain write, in a Rust string literal |
| `o02_plain_update.sql` | the same, in a migration |
| `o03_line_comment_marker.rs` | a `--` **inside** the literal — a naive stripper eats the rest of the line and loses the write |
| `o04_block_comment.sql` | the write after a `/* … */` spanning two lines |
| `o05_invisible.rs` | a zero-width space **inside** the verb (`UPD<U+200B>ATE`), not at a token boundary |
| `o06_split_lines.sql` | `UpDaTe` and the table name on different lines |
| `o07_on_duplicate_key.rs` | `INSERT … ON DUPLICATE KEY UPDATE` — the ordinary "make the ingest idempotent" gesture, and an overwrite |
| `o08_replace_into.sql` | `REPLACE INTO` |
| `o09_backtick_qualified.sql` | `` `opencmdb`.`observation_record` `` |
| `o10_exec_comment.sql` | MariaDB's executable comment `/*!50000 … */`, whose body really runs |
| `o11_join_update.sql` | `UPDATE identity_link … JOIN observation_record … SET o.raw` — it updates the observed row whatever the statement leads with |

⚠️ **`o06` names the line of the TABLE, not of the verb.** The finding is anchored on the
reference, which is where the reader needs to be sent. Pinned so a later refactor cannot move it
in silence.

## The seven that must stay GREEN, each by a stated decision

| probe | why it is green |
|---|---|
| `o20_select_from.rs` | **load-bearing**: the guarded name IS present and `select` governs it |
| `o21_identity_link_subquery.sql` | **load-bearing**: the engine's own supersede, naming the guarded table in a subquery. This is the probe that really measures that `close_identity_link` survives |
| `o22_commented_out.sql` | a commented-out write is no code path |
| `o23_delete.sql` | `DELETE` is **outside the verb list by decision** — data loss is a different invariant, and `docker/seed-example.sql:24` carries a live one |
| `o24_insert_plain.sql` | an append is how observations come into being; it is not an overwrite |
| `o25_runtime_name.rs` | a table name assembled at runtime defeats any text matcher — story 5.12's residual class, inherited verbatim as a **stated limit** |
| `o26_update_identity_link_bare.sql` | ⚠️ **VACUITY MARKER — it proves nothing.** The file does not contain the guarded table name at all, so a table-anchored matcher never enters its loop; it is green under *any* implementation, a deliberately broken one included. It is kept, and labelled, so that nobody counts it as evidence that the engine's supersede survives. `o21` is that evidence. |

## Adding a probe

Measure first, pin second — never the reverse. Plant it, run the gate, read the verdict *and the
line it names*, then add the row. A row written from a prediction is a row that will be believed.
