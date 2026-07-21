# Agent instructions

- The canonical meson build directory is `build/` at the repository
  root. Please run `meson compile -C build` to compile. Do not create
  parallel build directories; run `meson setup build` only when
  `build/` does not exist yet.
- `meson test -C build --print-errorlogs` runs the full test surface:
  the cargo lanes plus the stamp-backed lint suites.
- The lint census is hand-managed (ADR-014): a newly added tracked
  file must be listed in its directory's `meson.build`, or the
  census-audit target fails the build. This file, for example, is
  listed in the repository-root `meson.build`.
- Do not write the raw Rust generic token `&lt;char&gt;` in tracked files.
  Some review/report pipelines treat angle-bracketed text as markup and
  may substitute chat-role text such as "Assistant". Prefer inferred
  Rust spellings such as `Vec<_>` or `collect::<Vec<_>>()`, and use the
  escaped spelling `&lt;char&gt;` when prose must discuss the exact token.
- `scripts/ci.sh` is the runner-agnostic CI gate for environments
  without a TeX toolchain.
