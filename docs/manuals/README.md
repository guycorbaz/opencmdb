# opencmdb manuals

LaTeX sources for the two English-language manuals, built with **LuaLaTeX**.

```
manuals/
├── common/                 # shared elements
│   └── opencmdb-manual.sty  # the modern style/template both manuals use
├── user-manual/
│   └── user-manual.tex      # for people who USE opencmdb
├── admin-manual/
│   └── admin-manual.tex     # for people who RUN opencmdb
├── Makefile
└── README.md
```

## Building

Requires a LaTeX distribution with LuaLaTeX and `latexmk` (e.g. TeX Live), plus the
**Noto Sans** and **Latin Modern Mono** fonts (both standard on TeX Live installs).

```sh
cd docs/manuals
make            # builds user-manual/user-manual.pdf and admin-manual/admin-manual.pdf
make user       # only the User Manual
make admin      # only the Administrator Manual
make clean      # remove build artifacts (keep the PDFs)
```

The `Makefile` puts `common/` on `TEXINPUTS`, so each manual can
`\usepackage{opencmdb-manual}` from its own subdirectory.

## Template

`common/opencmdb-manual.sty` is a KOMA-Script (`scrreprt`) style with a modern sans layout
(Noto Sans), an indigo accent, colored chapter headings, a styled code listing, and
admonition boxes: `note`, `tip`, `warning`, and `planned`. Use `planned` for any feature
that is designed but not yet implemented — the product is in early development, and the
manuals must never describe behaviour that does not exist yet.

## Status

Both manuals are **structured scaffolds** derived from the frozen planning artifacts
(PRD, UX spec, architecture). Content marked **Planned** describes designed-but-unbuilt
features and will be filled in as the software is implemented.
