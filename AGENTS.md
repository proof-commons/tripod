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
- Meson graph and command wiring is tested by the mocked contract lane
  (`scripts/test-meson-mock.sh`): it configures one disposable build under
  `build/mocks/` with `-Dmock_mode=true` and simulates only the TeX
  toolchain via `execwrap --mock-child`, so it needs meson+ninja but no
  TeX. `build/mocks/` is the sole allowed exception to the no-parallel-
  build-directory rule; it is git-ignored and rebuilt from scratch each
  run. Do NOT add tests that invoke the real TeX toolchain, configure a
  second production build directory, or build the document twice — put
  semantic behaviour in Rust unit tests and graph/wiring in the mock lane.
  Byte reproducibility remains a separate manual/release check
  (`scripts/check-document-reproducibility.sh`), not an ordinary CI lane.
- Repository source, tests, Meson definitions, scripts, TeX, and
  `.latexmkrc` are executable. Do not run an untrusted contribution in a
  credential-bearing development environment; use an external secretless
  VM or sandbox. The repository does not establish its own isolation
  boundary (ADR-015).
