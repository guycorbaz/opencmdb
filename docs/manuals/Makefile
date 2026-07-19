# Build the opencmdb manuals with LuaLaTeX.
# The shared style (common/opencmdb-manual.sty) is put on TEXINPUTS so each manual
# can `\usepackage{opencmdb-manual}` regardless of its own subdirectory.
#
#   make            # build both manuals
#   make user       # build only the User Manual
#   make admin      # build only the Administrator Manual
#   make clean      # remove LaTeX build artifacts (keeps the PDFs)
#   make distclean  # also remove the PDFs

export TEXINPUTS := $(CURDIR)/common:$(TEXINPUTS)

LATEXMK := latexmk -lualatex -interaction=nonstopmode -halt-on-error -cd

.PHONY: all user admin clean distclean
all: user admin

user:
	$(LATEXMK) user-manual/user-manual.tex

admin:
	$(LATEXMK) admin-manual/admin-manual.tex

clean:
	$(LATEXMK) -c -cd user-manual/user-manual.tex
	$(LATEXMK) -c -cd admin-manual/admin-manual.tex

distclean:
	$(LATEXMK) -C -cd user-manual/user-manual.tex
	$(LATEXMK) -C -cd admin-manual/admin-manual.tex
