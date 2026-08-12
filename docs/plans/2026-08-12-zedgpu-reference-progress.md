# TDD Progress Journal

- Task: Evidence-backed idiomatic Zed GPUI reference and observation ledger
- Created: 2026-08-12 13:42:43Z
- Updated: 2026-08-12 14:04:53Z
- Current Phase: Refactor
- Status: complete

## Sessions

### 2026-08-12 13:44:19Z — Inventory and evidence boundary

#### Tests Written

- Defined a documentation acceptance check: every idiomatic claim must point to directly inspected source evidence.
- Defined a ledger integrity check: graph-only discoveries do not count as parsed files.
- Defined a separation check: prescriptive patterns belong in `idiomatic-reference-file-zedgpu.md`; descriptive internals and uncertain findings belong in `zedgpu-observations.md`.

#### Red

- The requested reference and observation ledger did not exist.
- No source revision, evidence rule, or parsed-file inventory had been recorded.

#### Green

- Created both requested artifacts with explicit scope and evidence boundaries.
- Recorded the Zed source revision and the graph-level inventory of `gpui`, `ui`, `gpui_tokio`, and `gpui_macos`.
- Added an empty parsed-file ledger that will only be populated after direct source reads.

#### Refactor

- Kept graph orientation separate from parsed-file evidence so later readers can tell discovery from verification.
- Kept the idiomatic reference intentionally skeletal until claims have source support.

#### Test Results

- Initial existence check was RED: both requested files were absent.
- Creation checkpoint is GREEN: both files now exist and have distinct responsibilities.

#### Implementation Progress

- Completed repository and crate-level orientation.
- Completed research artifact initialization.
- Source-level lifecycle/state/rendering inspection has not started.

#### Current Focus

- Trace the smallest complete application lifecycle and the entity/render update loop.

#### Next Steps

1. Read and ledger minimal examples plus their framework API definitions.
2. Trace entity updates, notifications, observation, subscriptions, and task ownership.
3. Append non-prescriptive runtime observations after each source batch.

#### Context Notes

- Source repository: `/Users/amuldotexe/Desktop/oss-read-only/zed-gpui`.
- Source revision: `6bd93fc3195242834f4999f3b3daab294df6b253` on `main`.
- The source checkout has a pre-existing untracked `crates/gpui/.code-map/`; do not modify or treat it as evidence.

#### Performance / Metrics

- Graph-scoped crates: 4.
- Directly parsed source files: 0.
- Finalized idiomatic patterns: 0.

### 2026-08-12 13:46:00Z — Lifecycle, ownership, and render pass

#### Tests Written

- Added a source-triangulation check for lifecycle patterns: framework overview, API implementation, and minimal usage must agree.
- Added a lifetime check for subscriptions: document both drop cancellation and explicit detachment semantics.
- Added a render identity check: distinguish entity-backed `Render` from stateless `RenderOnce` and record the state-collision caveat.

#### Red

- The mental model had no verified ownership or invalidation contract.
- Subscription and callback lifetimes were ambiguous.
- The reference did not distinguish view-level declarative composition from custom element work.

#### Green

- Verified the application → window → root entity → render lifecycle.
- Verified `App` ownership, typed entity handles, context-gated reads/updates, `notify/observe`, and `emit/subscribe`.
- Verified subscription drop/detach semantics and callback helper capture behavior.
- Verified view identity and cached subtree behavior.
- Appended 13 directly inspected files/spans to the observation ledger.

#### Refactor

- Separated prescriptive rules from implementation mechanics such as effect queues and initial draw behavior.
- Classified low-level `Element` work as an escalation path rather than the default UI register.

#### Test Results

- Lifecycle triangulation: GREEN across README, `Application::run`/`App::open_window`, and minimal examples.
- Subscription lifetime check: GREEN from `Subscription` implementation and ownership example.
- Render identity check: GREEN from `view.rs` documentation and implementation.

#### Implementation Progress

- Lifecycle/state/rendering research pass complete.
- Evidence ledger contains 13 source files.
- Six idiomatic classifications are ready to promote into the final reference after adjacent input/task evidence is collected.

#### Current Focus

- Trace task cancellation, weak entity updates across await points, actions, event propagation, focus, and accessibility.

#### Next Steps

1. Read async APIs and representative background/foreground task usage.
2. Read action, interaction, focus, tab-stop, and accessibility sources plus examples.
3. Check the UI component layer for repeated composition conventions.

#### Context Notes

- `Context::listener` captures weak self; `Context::processor` captures strong self to return a value.
- Emission is queued into the pending effect cycle.
- Entity view identity scopes descendant element state; duplicate sibling identity is unsafe.

#### Performance / Metrics

- Directly parsed source files: 13.
- Source-backed idiomatic classifications: 6.
- Open research categories: async/tasks, input/focus/a11y, styling/components, lists/custom elements, testing, platform boundary.

### 2026-08-12 13:51:11Z — Interaction, UI layer, lists, and testing pass

#### Tests Written

- Added a cancellation check: distinguish owned tasks, detached tasks, background work, and Tokio-bridged work.
- Added an interaction check: verify action and ordinary-event propagation defaults separately.
- Added an accessibility completeness check: stable identity, focus tracking, tab traversal, role, label, value/state, and accessible actions.
- Added a component-policy check: compare raw GPUI primitives against the Zed `ui` crate’s semantic components, theme tokens, spacing, and tooltip conventions.
- Added a visual-test check: verify tests operate on a rendered tree and explicitly advance detached asynchronous work.

#### Red

- Async cancellation and post-await entity access were undocumented.
- Action propagation, ordinary event propagation, and default prevention could easily be conflated.
- The initial draft had no evidence-backed policy for semantic components, spacing, colors, focus, accessibility, lists, or testing.

#### Green

- Verified foreground/background executor boundaries, weak entity capture, cancellation-on-drop, detachment, error-aware detachment, and Tokio cancellation propagation.
- Verified actions, key contexts, focus tracking, tab traversal, event propagation defaults, and accessibility identity.
- Verified the Zed UI layer’s component-first, semantic-color, dynamic-spacing, and tooltip conventions.
- Verified declarative versus custom-element boundaries, uniform versus variable-height list choices, and retained scroll handles.
- Verified deterministic GPUI test contexts and action, keyboard, mouse, prompt, file-selection, event-stream, resize, and property-test helpers.
- Expanded the parsed ledger to 45 unique files (47 rows because two files gained additional inspected spans).

#### Refactor

- Moved platform/runtime mechanics and caveats into the observation ledger instead of presenting them as application recipes.
- Converted API facts into decision rules only where framework documentation and representative usage agree.
- Kept advanced canvas/custom-element behavior behind an explicit escalation boundary.

#### Test Results

- Cancellation/lifetime check: GREEN across context spawning, executor, async-context, image-loading example, and Tokio adapter.
- Interaction check: GREEN across action registry, window dispatch implementation, tab-stop example, and accessibility example.
- Component-policy check: GREEN across UI prelude, component catalog, buttons, list items, tooltips, colors, and spacing.
- Visual-test check: GREEN across example tests, the test attribute runtime, and app/visual test contexts.

#### Implementation Progress

- Core lifecycle, state, rendering, async work, interaction, focus, accessibility, semantic components, lists, and testing are source-backed.
- The observation ledger contains every directly parsed file and span from this pass.
- The reference outline is ready for promotion after four targeted gaps are closed.

#### Current Focus

- Close targeted gaps in reusable text input, typed globals, fluent style/interaction helpers, and platform/macOS boundaries.

#### Next Steps

1. Inspect `ui_input` and the custom `InputHandler` boundary.
2. Inspect typed globals plus author-facing styling/interaction traits.
3. Inspect platform selection and selected macOS internals, then promote stable application idioms into the reference.

#### Context Notes

- GPUI action handlers stop bubbling by default; ordinary event handlers propagate by default.
- A rendered focus handle and stable element IDs are part of correctness, not optional metadata.
- The `ui` crate encodes Zed product consistency that raw GPUI primitives do not provide automatically.
- Custom elements and canvases are intentionally advanced escape hatches.

#### Performance / Metrics

- Directly parsed source files: 45 unique files.
- Parsed ledger rows: 47.
- Source-backed idiomatic classifications: 14.
- Open research categories: reusable input, globals, fluent styling/interaction, platform/macOS boundary.

### 2026-08-12 13:54:53Z — Input, globals, composition, and macOS boundary pass

#### Tests Written

- Added a text-input escalation check: high-level field usage must be separated from custom IME implementation.
- Added a global-state scope check: app-wide state must not be presented as a replacement for entity-local state.
- Added an identity check: stateful interaction and accessibility must follow stable element identity.
- Added a platform-boundary check: application authoring must stop at GPUI abstractions unless implementing the backend itself.

#### Red

- The draft could have implied that `ui_input::InputField` works without the editor subsystem.
- Typed globals lacked an access-control and scope warning.
- Fluent conditionals, styling, and the transition from ordinary to stateful interaction had not been traced to their defining traits.
- The relationship between `gpui_platform`, AppKit windows, frame pacing, and Metal rendering was not explicit.

#### Green

- Verified that `InputField` is editor-backed, factory-initialized, focus-aware, single-line, masked/validated, and event-capable.
- Verified the complete custom entity input adapter and platform IME contract, including UTF-16 and marked-text responsibilities.
- Verified typed `Global` read/update helpers and the framework’s recommended private-wrapper restriction pattern.
- Verified `FluentBuilder`, the complete `Styled` trait, central `div` role, and the ID-gated stateful interaction boundary.
- Verified cross-platform application selection plus selected macOS AppKit, display-link, and Metal seams.
- Expanded the parsed ledger to 60 unique files (62 rows because two core files have multiple independently recorded spans).

#### Refactor

- Classified `ui_input` as a Zed-integrated dependency choice, not a bare-GPUI default.
- Kept IME protocol implementation and all macOS renderer/frame-pacing mechanics in the advanced/internal category.
- Turned globals into a scoped policy with explicit encapsulation, rather than a convenience recommendation.

#### Test Results

- Text-input escalation check: GREEN across `ui_input`, editor factory setup, entity adapter, and platform IME contract.
- Global-state scope check: GREEN from `global.rs` documentation and its typed helper design.
- Identity/composition check: GREEN across `FluentBuilder`, `Styled`, and `InteractiveElement`/`StatefulInteractiveElement`.
- Platform-boundary check: GREEN from `gpui_platform`, `gpui_macos`, window display-link subscription, and Metal scene consumption.

#### Implementation Progress

- All planned research categories now have direct source evidence.
- Sixty unique source files have been directly parsed and recorded.
- The source-backed pattern catalog is ready to write; only line-anchor collection and final overstatement audits remain.

#### Current Focus

- Promote classified idioms into the final reference with concise recipes, consequences, failure modes, and pinned source links.

#### Next Steps

1. Collect exact source line anchors for every promoted pattern.
2. Replace the reference skeleton with the complete pattern catalog and examples.
3. Audit every local source link, ledger path, and prescriptive claim.

#### Context Notes

- Zed’s reusable `InputField` requires editor initialization; a small standalone GPUI app may choose a narrower custom component instead of pulling in editor.
- Stateful interactivity follows `.id(...)`; stable identity is a functional requirement for state and accessibility.
- The native macOS pipeline is already abstracted behind `gpui_platform::application()` and platform-neutral scenes.

#### Performance / Metrics

- Directly parsed source files: 60 unique files.
- Parsed ledger rows: 62.
- Source-backed idiomatic classifications: 18.
- Open research categories: none; audit work remains.

### 2026-08-12 14:03:31Z — Pattern promotion and reference drafting

#### Tests Written

- Added a coverage check requiring the reference to address every completed research category.
- Added a source-link check requiring every prescriptive section to cite local pinned evidence.
- Added a snippet audit for known API-shape hazards: action-aware tooltip constructors, background-to-foreground handoff, rendered-tree action dispatch, typed global helper imports, and current-window semantics.
- Added an optimization restraint check for cached views and custom elements.

#### Red

- The idiomatic reference was only a skeleton with “Research in progress” placeholders.
- Several initially drafted snippets used plausible but incorrect API shapes for action tooltips, async context re-entry, and visual-test action dispatch.
- Assets, declarative animations, and cached-view constraints were researched but missing from the pattern catalog.

#### Green

- Replaced the skeleton with a 20-pattern reference covering startup, ownership, invalidation/events, lifetimes, rendering, semantic components, fluent composition, identity, actions, focus, accessibility, async work, lists, input, globals, assets/animation, windows, testing, custom elements/caching, and the macOS boundary.
- Corrected snippets against directly inspected signatures and examples.
- Added a decision table, failure-mode table, pre-commit checklist, pinned revision warning, and 79 local source links.
- Promoted asset/animation and measured cached-view guidance into the reference and observation classifications.

#### Refactor

- Kept recipes concise while attaching consequences and escalation boundaries to each pattern.
- Used one platform-neutral startup path and relegated AppKit, display-link, and Metal details to boundary evidence.
- Marked code fragments as focused recipes rather than claiming that every fragment is a standalone crate.

#### Test Results

- Research-category coverage: GREEN; all planned categories are represented.
- Initial snippet audit: RED for three API-shape issues, then GREEN after source-aligned corrections.
- Source-link target existence: GREEN in the first audit pass.
- Markdown fence balance: GREEN in the first audit pass.

#### Implementation Progress

- The reference content is complete and source-backed.
- The observation ledger contains 62 parsed-span rows across 60 unique files.
- Final mechanical validation and worktree review remain.

#### Current Focus

- Run the final link/anchor, Markdown structure, ledger integrity, whitespace, and repository-status checks.

#### Next Steps

1. Re-run all documentation validation after the final asset/caching additions.
2. Inspect the final diff/status for unrelated changes.
3. Close the progress journal with exact verification results.

#### Context Notes

- Action-aware button tooltips use `Tooltip::for_action_title(...)` when a tooltip closure is required.
- Visual-test action dispatch should follow the rendered focus-handle path used in GPUI’s own example.
- Cached views require a definite size and reliable notification-driven invalidation.

#### Performance / Metrics

- Directly parsed source files: 60 unique files.
- Parsed ledger rows: 62.
- Reference pattern sections: 20.
- Local pinned source links before final audit: 79.

### 2026-08-12 14:04:53Z — Final documentation verification

#### Tests Written

- Validated whitespace and unfinished-work marker absence in both requested artifacts.
- Validated Markdown fence balance and sequential numbered pattern headings.
- Validated every absolute source link, its line anchor, and every relative document link.
- Validated sequential ledger numbering, source-path existence, and unique-file count.
- Validated repository scope and checked the read-only source checkout for accidental writes.

#### Red

- The first unfinished-work check treated the ordinary UI term “placeholder” as a work marker and also matched a historical “Research in progress” sentence inside this append-only journal.
- The first Ruby line-anchor script used `filter_map`, which is unavailable in the installed Ruby version.

#### Green

- Narrowed the gate to actual unfinished-work markers in the two deliverable artifacts while retaining the historical journal entry unchanged.
- Rewrote the compatibility check with `map(...).compact` and reran the full audit successfully.
- Confirmed 38 balanced code-fence markers, 80 valid source line anchors, 2 valid relative links, 62 sequential ledger rows, 60 unique ledger files, and 20 sequential pattern sections.
- Confirmed zero trailing-whitespace matches and exactly three new files in the working repository.
- Confirmed the source checkout still has only its pre-existing untracked `crates/gpui/.code-map/` directory.

#### Refactor

- Corrected the current-window evidence link to the exact `App::with_window` documentation.
- Added the missing typed-global trait import to its recipe.
- Added source-backed asset/animation guidance and the definite-size cached-view constraint.

#### Test Results

- `git diff --check`: PASS for tracked changes; explicit trailing-whitespace scan covers the three untracked deliverables and also passed.
- Unfinished-work markers in deliverables: PASS, zero matches.
- Markdown fence balance: PASS, 38 markers.
- Source line anchors: PASS, 80 of 80 resolve within existing files.
- Relative links: PASS, 2 of 2 resolve.
- Ledger integrity: PASS, rows 1–62 sequential, 62 paths present, 60 unique files.
- Pattern numbering: PASS, sections 1–20 sequential.
- Worktree scope: PASS, only the reference, observation ledger, and progress journal are new.

#### Implementation Progress

- Requested reference file is complete.
- Requested continuous observation ledger is complete.
- TDD research/progress journal is complete and resumable.
- No source code or read-only Zed checkout files were modified.

#### Current Focus

- Handoff the completed source-backed documentation artifacts.

#### Next Steps

1. Use the reference as the implementation checklist for the GPUI application.
2. Re-run the evidence audit when the pinned Zed revision changes.
3. Add new parsed files to the observation ledger before promoting future idioms.

#### Context Notes

- The artifact filename retains the requested `zedgpu` spelling; the document uses the canonical framework name GPUI.
- The most important dependency caveat is that `ui_input::InputField` requires the editor factory.
- The most important backend boundary is that feature code should remain above `gpui_platform`.

#### Performance / Metrics

- Final reference: 667 lines before this journal-only checkpoint, 3,876 words.
- Final observations before the audit entry: 200 lines, 4,457 words.
- Directly parsed source files: 60 unique files.
- Final verified source anchors: 80.
