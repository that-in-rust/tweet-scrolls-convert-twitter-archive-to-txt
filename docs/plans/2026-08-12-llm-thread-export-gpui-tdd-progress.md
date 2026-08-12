# LLM-ready Tweet Thread Exporter — TDD Progress

**Branch:** `Aug-12-01`
**Spec:** [Executable specification](./2026-08-12-llm-thread-export-gpui-executable-spec.md)
**Method:** Every production change follows a witnessed `RED → GREEN → REFACTOR → VERIFY` cycle.

## Baseline

- Initial `cargo test --all-targets`: RED before feature work — 81 passed, 1 failed.
- Existing failure: `utils::file_splitter::tests::test_split_with_custom_output_dir` at `src/utils/file_splitter.rs:394` compared a canonicalized chunk path with a non-canonical expected directory on macOS.
- Baseline repair: compare against the operation's canonical `result.output_dir`.
- Baseline GREEN: targeted regression test passed, three CLI characterization tests passed, then `cargo test --all-targets` passed.
- Existing warnings are tracked for the final `clippy -D warnings` gate.

## Slice ledger

| Slice | RED evidence | GREEN evidence | Refactor/verify status |
|---|---|---|---|
| 0 — Characterize baseline | Existing path assertion failed; new CLI tests then compiled | Baseline fixed; 3/3 CLI characterization tests pass | Complete |
| 1 — GPUI feature boundary | 2/2 manifest tests failed before feature/dependency declarations; runtime-shader and toolchain tests failed before config | GPUI crates pinned to Zed `6bd93fc`; feature-gated Mac binary checks with Rust 1.95 and runtime shaders | Complete; feature build passes without full Xcode |
| 2 — Shared archive core | Core test target failed on missing module/API; cross-month chronological test reproduced wrong `[2, 1]` order | 8/8 core tests and 3/3 CLI characterization tests pass | Complete; CLI routes through shared loader |
| 3 — Atomic full TXT | Full-export target failed on missing renderer/path/service and typed failures | 8/8 renderer/path/full-export tests pass | Complete; CLI TXT writer reuses renderer |
| 4 — Semantic parts | Parts target failed on missing planner/writer contracts | 7/7 size, Unicode, semantic-boundary, cleanup, and retry tests pass | Complete; plan retains descriptors rather than one full payload string |
| 5 — Typed app state | State target failed on missing enum/events/transitions | 6/6 prompt-order, decline, guard, retry, and reveal tests pass | Complete; prepared data is dropped at terminal completion |
| 6 — GPUI controls/Tokio | Feature test failed on missing picker adapter; initial native build failed without Metal compiler | 2/2 feature-gated picker tests pass; Mac binary checks with `Tokio::spawn_result` and retained tasks | Complete; feature-wide tests and native launch smoke pass |
| 7 — Package/quality gates | 3/3 packaging contract tests failed before assets/version/script | 3/3 packaging contract tests pass | Complete; release bundle, strict Clippy, plist, shell, diff, and launch gates pass |

## Final verification

- `cargo test --features gpui-app --all-targets`: GREEN across every library, binary, integration, and feature-gated target; all 41 new v0.0.2 contract tests pass.
- `cargo clippy --features gpui-app --all-targets -- -D warnings`: GREEN with zero warnings.
- Targeted `rustfmt --check`, `sh -n scripts/package_mac_app.sh`, `plutil -lint packaging/macos/Info.plist`, and `git diff --check`: GREEN.
- `scripts/package_mac_app.sh`: built `target/release/bundle/osx/Tweet Scrolls.app` from the final source tree.
- Native launch smoke: the bundled `tweet-scrolls-mac` process initialized and remained running until intentionally terminated.
