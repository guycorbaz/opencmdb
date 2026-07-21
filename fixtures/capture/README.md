# fixtures/capture/

Real payloads captured from a live source. They prove the **parser**, never the engine.

**They rot**, and that is expected: a device moves, a lease expires, a firmware renames a field.
Each capture is therefore tagged with the source application's version and its capture date, and
a re-capture job diffs the recorded schema against the live one.

Two rules that are not negotiable:

1. **When the re-capture job is written, it must be structurally unable to reach
   `../scenario/`** — not by a check that carelessness can bypass, but by a signature that does
   not permit the fault: the tool is to hold `capture/` as a module constant and never take a
   path parameter (D56). *No such tool exists yet, and nothing in the code enforces this today;
   it is a requirement on the story that introduces it (Epic 11), recorded here so it is not
   discovered afterwards.*
2. **Nothing here may carry real private data into a public repository.** Captures must be
   scrubbed — addresses, MACs, hostnames — before they are committed (D19).

Empty until the first real connector exists (Epic 11).
