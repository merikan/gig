# gig

A Rust CLI that clones repos into a predictable `root-dir/host/owner/repo` workspace, and recognizes repos already cloned there instead of re-cloning or clobbering them.

## Language

**Destination**:
The on-disk path `get` computes for a URL - `root-dir` joined with the URL's host and path segments, mirrored verbatim. Distinct from the URL itself: a destination is a filesystem location, a URL is where a repo comes from.

**Already-cloned**:
A destination that already contains a `.git` subdirectory. `get` treats this as "nothing to do" by default, or a pull target with `--pull` - never a clone target.

**Occupied destination**:
A destination that already exists on disk but is *not* already-cloned (no `.git` subdirectory) - a stray file, an empty directory, or unrelated contents. `get` refuses to clone into it and errors out rather than touching it.
_Avoid_: "existing destination" (ambiguous - both already-cloned and occupied destinations "exist"; use the specific term for which case is meant).

**Candidate destination**:
A destination path considered during the non-destructive existing-clone search - `get` probes the default destination and every declared category's destination for a URL, without touching disk, before settling on the one destination it will act on. Distinct from Destination: a URL has exactly one destination but may have several candidate destinations.

**Default category**:
A declared category marked `--default`, used to route a clone when no declared category's pattern matched the URL - instead of falling back to bare `root-dir`. Independent of a category's patterns: a category can match its own patterns *and* be the default, catching only what nothing else (including itself) matched. At most one category is default at a time. See [ADR 0004](docs/adr/0004-default-category-as-orthogonal-flag.md).
