# Zed GPUI Research Observations

This is the append-oriented research ledger for `idiomatic-reference-file-zedgpu.md`.
It records every source file and span inspected, plus useful implementation details that are descriptive rather than prescriptive.

## Evidence boundary

- Source repository: `/Users/amuldotexe/Desktop/oss-read-only/zed-gpui`
- Source revision: `6bd93fc3195242834f4999f3b3daab294df6b253`
- Source branch at inspection: `main`
- Reference rule: an idiom enters the reference only after it is verified in source and supported by either framework API design plus usage, or repeated high-quality usage.
- Observation rule: backend mechanics, incidental implementation choices, uncertain conventions, and future investigation leads stay here.
- A ledger row means the named source span was directly read. Graph search results alone do not count as parsed files.

## Graph orientation — 2026-08-12

Before reading individual files, the codebase-memory graph was scoped to the four relevant crates.

- `crates/gpui`: 6,220 graph nodes across 88 Rust files. Dominant seams are entity/context updates, render/layout/paint, div interactivity, focus/key dispatch, tasks, and subscriptions.
- `crates/ui`: 2,365 graph nodes across 108 Rust files. Dominant seams are flex/stack composition, labels, buttons, lists, menus, tooltips, and shared styling.
- `crates/gpui_tokio`: a deliberately small Tokio interoperability bridge. Its graph centers on initialization, handle access, spawning, and cancellation/result handoff.
- `crates/gpui_macos`: 885 graph nodes across 17 Rust files. It is primarily platform implementation evidence, not an application-authoring API surface.

High fan-in around `Entity::update_in`, `Window::spawn`, `Subscription::detach`, context event emission, and `IntoElement` conversion identifies lifecycle ownership as the most important area to verify carefully.

## Parsed-file ledger

| # | Source file | Spans or symbols read | Why inspected | Classification | Notes |
|---:|---|---|---|---|---|
| 1 | `crates/gpui/README.md` | 1–98 | Framework-authored big picture and startup guidance | Idiom evidence | GPUI explicitly describes itself as hybrid immediate/retained mode with three registers: entities, views, and low-level elements. It warns that the pre-1.0 API changes often. |
| 2 | `crates/gpui/src/_ownership_and_data_flow.rs` | 1–140 | Canonical ownership, observation, and event flow | Idiom evidence | `App` owns entity state; typed handles require a context to read/update; `notify/observe` is state invalidation while `emit/subscribe` is typed event delivery. |
| 3 | `crates/gpui/src/gpui.rs` | 1–346 | Public surface, context traits, exports | API evidence | `AppContext` and `VisualContext` define the shared context boundary. The crate exposes separate entity, view, element, style, input, task, and window systems. |
| 4 | `crates/gpui/src/prelude.rs` | 1–9 | Intended application imports | Idiom evidence | The crate explicitly recommends importing `gpui::prelude::*`; it exposes the composition traits needed for fluent element building. |
| 5 | `crates/gpui/src/app.rs` | `Application::run` 226–236; `App::open_window` 1236–1269 | Lifecycle implementation | API/internal evidence | `run` enters the platform loop and gives the launch callback `&mut App`; `open_window` requires a `Render` root entity and forces an initial draw before returning. |
| 6 | `crates/gpui/src/app/context.rs` | 1–330; 730–810 | Entity-local callbacks, invalidation, events, async ownership | Idiom evidence | `Context<T>` dereferences to `App`, captures weak self for listeners/observers, and documents that spawned tasks must be held or detached. |
| 7 | `crates/gpui/src/subscription.rs` | 130–190 | Subscription cancellation semantics | Idiom evidence | `Subscription` is `#[must_use]`; drop invokes unsubscribe, while `detach` removes the drop callback so delivery continues until participating entities die. |
| 8 | `crates/gpui/src/executor.rs` | 1–120 | Executor split and error-aware detachment | Idiom evidence | Foreground tasks are local/main-thread; background tasks require `Send`. `TaskExt::detach_and_log_err` preserves error visibility for detached fallible work. |
| 9 | `crates/gpui/src/view.rs` | 1–380 | View identity, render conversion, subtree caching | Idiom/API evidence | `Entity<T: Render>` has retained identity; `RenderOnce` is stateless. Entity identity scopes element state, and notification controls cached subtree reuse. |
| 10 | `crates/gpui/examples/scrollable.rs` | 1–72 | Minimal application and declarative styling | Usage evidence | Shows `application().run`, `open_window`, root `cx.new`, `Render`, fluent `div`, and explicit scroll IDs. |
| 11 | `crates/gpui/examples/ownership_post.rs` | 1–50 | Compact typed-event example | Usage evidence | Confirms `EventEmitter`, `subscribe`, `emit`, handle reads, and detached entity-scoped subscription in one flow. |
| 12 | `crates/gpui/examples/on_window_close_quit.rs` | 1–97 | Actions, focus, multi-window lifetime | Usage evidence | Shows action declaration/binding/handling, focus tracking, explicit application quit after the last window, and detached app-lifetime listener. |
| 13 | `crates/gpui/examples/uniform_list.rs` | 1–65 | Virtualized list entry point | Usage evidence | Uses a stable list ID and `cx.processor` to regain entity state inside the range renderer. |
| 14 | `crates/gpui/src/app/async_context.rs` | 1–535 | Entity/window access across await points | Idiom/API evidence | Async contexts regain synchronous access through `update`; entity handles are weak by default in entity-scoped tasks, and missing entities/windows surface as typed errors. |
| 15 | `crates/gpui/src/app/context.rs` | 650–725, in addition to row 6 | Entity-scoped spawning | Idiom evidence | `spawn` delegates to `spawn_in`; the callback receives a weak entity plus async app/window context, and dropping the returned task cancels it unless detached. |
| 16 | `crates/gpui/src/app.rs` | 1770–1810; 1870–1915; 2160–2220; 2370–2410, in addition to row 5 | Background work, global action handlers, window lookup | API evidence | Background work requires `Send + 'static`; app action listeners live outside the element tree; entity-to-window lookup means “most recently rendered,” not ownership. |
| 17 | `crates/gpui/src/window.rs` | 480–710; 2160–2225; 2310–2355; 2515–2565; 2885–2915; 6025–6095 | Focus, actions, refresh, and event propagation | Idiom/API evidence | Action handlers stop bubbling by default, ordinary event handlers propagate by default, `prevent_default` is separate, and focus traversal is explicit. |
| 18 | `crates/gpui/src/action.rs` | 1–458 | Action declarations, registry, dispatch | Idiom/API evidence | Unit actions use `actions!`; payload actions derive `Action`; action names are namespaced and key bindings resolve through context. |
| 19 | `crates/gpui/src/_accessibility.rs` | 1–295 | Accessibility identity and actions | Idiom/API evidence | Stable global element IDs preserve node identity; duplicate IDs in a frame are invalid; accessible actions are distinct from GPUI actions. |
| 20 | `crates/gpui/examples/a11y.rs` | 1–266 | Accessible control construction | Usage evidence | Demonstrates roles, labels, values, state, focus, tab stops, and accessible action handlers together. |
| 21 | `crates/gpui/examples/tab_stop.rs` | 1–214 | Keyboard focus traversal | Usage evidence | Shows `FocusHandle`, `Focusable`, tracked focus, tab-stop declarations, and explicit next/previous focus actions. |
| 22 | `crates/gpui/examples/animation.rs` | 1–134 | Frame animation and asset-backed rendering | Usage evidence | Animation is declarative via element extensions; asset loading is configured on the application and used during render. |
| 23 | `crates/gpui_tokio/src/gpui_tokio.rs` | 1–100 | Tokio interoperability and cancellation | Idiom/API evidence | The bridge initializes or accepts a runtime handle; dropping the GPUI task aborts the Tokio join handle and task failure crosses the bridge as a result. |
| 24 | `crates/gpui/examples/image_loading.rs` | 1–226 | Asynchronous asset/network loading | Usage evidence | Long-lived view state owns its loading task; background work returns data and foreground `update` mutates the view and notifies. |
| 25 | `crates/ui/src/ui.rs` | 1–20 | Zed UI public surface | API evidence | Zed’s UI layer reexports components, prelude, styles, traits, and utilities over GPUI. |
| 26 | `crates/ui/src/prelude.rs` | 1–35 | Intended Zed application imports | Idiom evidence | The source says this prelude is intended for almost all files using the UI crate and combines GPUI composition with semantic UI types. |
| 27 | `crates/ui/src/component_prelude.rs` | 1–6 | Component-author import boundary | Idiom evidence | Component implementations get a smaller GPUI/UI prelude distinct from the broader application prelude. |
| 28 | `crates/ui/src/components.rs` | 1–85 | Component catalog and public exports | API evidence | The UI layer exposes buttons, labels, lists, menus, tables, tooltips, toggles, editors, and other semantic building blocks. |
| 29 | `crates/ui/src/styles.rs` | 1–18 | Semantic style surface | API evidence | Shared color, elevation, spacing, platform, and unit tokens are centralized. |
| 30 | `crates/ui/src/traits.rs` | 1–8 | Shared component traits | API evidence | Component behavior is composed from traits such as clickable, disableable, fixed-width, selectable, and toggleable. |
| 31 | `crates/ui/src/components/button/button.rs` | 1–648 | Canonical button composition | Idiom evidence | Buttons derive accessible names from visible labels, expose key bindings, set roles, and suppress click behavior while disabled. |
| 32 | `crates/ui/src/components/button/button_like.rs` | 1–190; 470–900 | Shared interaction shell and tooltip policy | Idiom evidence | `ButtonLike` is intentionally an escape hatch; source warns unconstrained use can make UI inconsistent, and nearly all interactables should expose tooltips. |
| 33 | `crates/ui/src/components/stack.rs` | 1–15 | Flex composition helpers | Idiom evidence | `h_flex` and `v_flex` provide the standard row/column starting point. |
| 34 | `crates/ui/src/styles/spacing.rs` | 1–54 | Density-aware spacing | Idiom evidence | `DynamicSpacing` centralizes compact/default/comfortable density and explicitly tells callers not to derive spacing from density manually. |
| 35 | `crates/ui/src/styles/units.rs` | 1–29 | Rem conversion policy | Idiom evidence | Rem conversion is centralized around the UI font size instead of arbitrary local scaling. |
| 36 | `crates/ui/src/styles/color.rs` | 1–242 | Semantic color policy | Idiom evidence | Semantic colors preserve meaning across themes; `Color::Custom` is strongly discouraged because it severs that relationship. |
| 37 | `crates/ui/src/components/list/list_item.rs` | 1–583 | Canonical selectable row | Idiom evidence | List items compose slots, selected/disabled/hover state, semantic colors, role, label, checked state, and active-descendant accessibility. |
| 38 | `crates/ui/src/components/tooltip.rs` | 1–291 | Text, action, and focus-aware tooltips | Idiom evidence | Action tooltips derive current key bindings; focus-aware tooltips avoid competing with keyboard focus. |
| 39 | `crates/component_preview/examples/component_preview.rs` | 1–156 | Standalone Zed UI initialization | Usage evidence | Configures assets, fonts, settings, theme loading, and UI font setup before constructing the root view. |
| 40 | `crates/markdown/examples/markdown_as_child.rs` | 1–113 | Smaller standalone Zed UI application | Usage evidence | Confirms assets/theme setup and root-view construction outside the full editor application. |
| 41 | `crates/gpui/src/element.rs` | 1–240 | Low-level element contract | Idiom/API evidence | Custom elements own imperative request-layout, prepaint, and paint phases; the framework recommends ordinary components unless manual layout/paint is needed. |
| 42 | `crates/gpui/src/elements/uniform_list.rs` | 1–250; 600–730 | Uniform virtual list mechanics | Idiom/API evidence | The list renders visible ranges, measures a representative item, and exposes a reusable scroll handle whose state belongs with the view. |
| 43 | `crates/gpui/examples/list_example.rs` | 1–170 | Variable-height list usage | Usage evidence | `ListState` owns item production/measurement and is rendered with the `list` element; this is distinct from uniform-height virtualization. |
| 44 | `crates/gpui/examples/painting.rs` | 1–260 | Manual drawing boundary | Internal/advanced evidence | Canvas painting exposes bounds in prepaint and draws paths in paint; it is useful for specialized rendering, not a default component pattern. |
| 45 | `crates/gpui/examples/testing.rs` | 1–552 | End-to-end GPUI test examples | Idiom evidence | Demonstrates deterministic executor draining, action/keyboard/mouse simulation, prompt/file mocks, event streams, resize, and property iterations. |
| 46 | `crates/gpui/src/test.rs` | 1–180 | GPUI test attribute/runtime contract | API evidence | `#[gpui::test]` supplies test contexts, supports async tests and iterations, and runs on GPUI’s deterministic test executor. |
| 47 | `crates/gpui/src/app/test_context.rs` | 1–640; 730–990 | Test context and visual-driving APIs | Idiom/API evidence | Side effects flush after synchronous updates; visual tests require a rendered window; helper APIs simulate input and expose notifications/events. |
| 48 | `crates/ui_input/src/input_field.rs` | 1–272 | Zed-integrated reusable text field | Idiom evidence | Wraps a single-line editor with labels, placeholder, icon, validation, masking, tab/focus support, semantic styling, and tooltip-backed password reveal. |
| 49 | `crates/ui_input/src/ui_input.rs` | 1–45 | Editor abstraction and initialization seam | API/internal evidence | The crate depends on the editor through an erased factory, exposes editor operations/events, and cannot live in the lower-level `ui` crate. |
| 50 | `crates/editor/src/editor.rs` | 380–405 | `ui_input` factory initialization | Usage/internal evidence | Editor initialization installs a factory that constructs a single-line editor and erases it behind the `ui_input` interface. |
| 51 | `crates/gpui/src/global.rs` | 1–75 | Typed application-global state | Idiom/API evidence | `Global` is a marker for type-safe app state; source recommends private wrapper types and restricted accessors when access should be constrained. |
| 52 | `crates/gpui/src/util.rs` | 1–140 | Fluent conditional composition | Idiom evidence | `map`, `when`, `when_else`, `when_some`, and `when_none` keep conditional construction inside fluent element chains. |
| 53 | `crates/gpui/src/styled.rs` | 1–904 | Author-facing styling trait | Idiom/API evidence | `Styled` is the opt-in Tailwind-like surface for layout, sizing, overflow, typography, color, borders, grids, and debug outlines. |
| 54 | `crates/gpui/src/elements/div.rs` | 1–580; 700–1420; 1660–1785; 2360–2960 | Central container and interactivity traits | Idiom/API/internal evidence | `div` is the central reusable container; IDs upgrade it to stateful interaction/a11y; focus, key contexts, capture/bubble handlers, hover styles, hitboxes, and click synthesis are composed here. |
| 55 | `crates/gpui/src/input.rs` | 1–233 | Entity-backed platform text input adapter | Advanced API evidence | Custom text systems implement UTF-16 selection/composition/geometry behavior, wrap it in `ElementInputHandler`, and register it during element paint. |
| 56 | `crates/gpui/src/platform.rs` | 1390–1465; 1635–1815 | Platform input bridge contract | Internal/advanced evidence | The platform-facing interface mirrors IME selection, marked text, replacement, candidate-window geometry, and printable-key routing; this is not needed for ordinary form fields. |
| 57 | `crates/gpui_platform/src/gpui_platform.rs` | 1–180 | Cross-platform application selection | Idiom/platform evidence | `application()` selects the native backend by target; on macOS it constructs `MacPlatform`, so authoring code normally stays platform-neutral. |
| 58 | `crates/gpui_macos/src/gpui_macos.rs` | 1–137 | macOS module/export boundary | Internal evidence | The crate hides dispatcher, AppKit window, display-link, clipboard, and renderer modules and publicly exposes `MacPlatform`. |
| 59 | `crates/gpui_macos/src/platform.rs` | 1–125; 620–685; 1230–1265 | macOS application/window adapter | Internal evidence | GPUI windows are translated to AppKit windows; unsupported native anchored popups deliberately fall back to in-window popovers. |
| 60 | `crates/gpui_macos/src/window.rs` | 620–710 | Native window frame pacing seam | Internal evidence | Visible windows subscribe to a display-specific frame source and stop it when hidden/tearing down; display reconfiguration can temporarily yield no screen. |
| 61 | `crates/gpui_macos/src/display_link.rs` | 1–285 | CVDisplayLink lifetime and fan-out | Internal evidence | One immortal display link per display feeds per-window dispatch sources; the design avoids unsafe CoreVideo teardown races and coalesces vsync on the main queue. |
| 62 | `crates/gpui_macos/src/metal_renderer.rs` | 80–190; 420–490; 1200–1275 | Metal render/present boundary | Internal evidence | A scene is lowered to a drawable and command buffer; presentation differs for transaction-backed windows, while test support can render offscreen/read textures. |

## Non-idiomatic internals and future-use notes

### Repository state

- The source checkout was on `main` at the revision above.
- `crates/gpui/.code-map/` was already untracked when repository state was inspected; this research does not treat it as source evidence or modify it.

### Runtime and ownership details

- `App::open_window` performs an initial draw before returning. The implementation comment says this is needed to avoid returning a never-rendered window on Windows; application code should not infer that opening a window is a lazy, render-free operation.
- `Context::listener` captures a weak entity and silently ignores callbacks after the entity is gone. `Context::processor` captures a strong entity because it must return a value. That is an internal lifetime difference worth remembering when choosing helpers, but the idiomatic rule is expressed in terms of callback shape.
- `observe` and `subscribe` also capture the observing entity weakly. Their callbacks return `false` internally once it can no longer be upgraded, allowing GPUI to remove dead subscribers.
- Emitted events are placed into `App::pending_effects`; delivery is effect-cycle based rather than a direct nested call from `Context::emit`.
- A view entity’s ID scopes its descendant element-state namespace. Rendering the same view identity as siblings at the same tree position can collide internal `use_state` and scroll state; nesting is safe because parent paths scope the ID.
- `AnyView::cached` can recycle the previous rendered subtree until the backing entity notifies, while `Window::refresh` bypasses that cache.
- `AppContext::with_window` uses an entity’s most recently rendered window. This is a convenience boundary, not proof that an entity permanently belongs to one window.
- The framework crate is `#![warn(missing_docs)]` but also explicitly says GPUI is pre-1.0 and often breaks between versions. Pin the source revision when treating these notes as authoritative.

### Async and executor details

- Entity-scoped `Context::spawn` passes a weak entity handle on purpose. This prevents a task from keeping the view alive accidentally; callers must handle failed upgrades after an await point.
- The foreground executor is local to the application thread. It is the correct place to re-enter GPUI state, but CPU-heavy or blocking work would stall UI progress.
- Background tasks require `Send + 'static`. They cannot directly retain non-thread-safe GPUI view state, which makes the data handoff boundary explicit.
- A `Task` is a cancellation handle: dropping it cancels unfinished work. Detaching intentionally transfers that lifetime to the executor. `detach_and_log_err` is preferable for fallible fire-and-forget work because it preserves error visibility.
- The Tokio adapter is small by design. A GPUI task awaits a Tokio join handle; dropping the GPUI side aborts Tokio work. This is cancellation propagation, not merely executor conversion.
- Async app and window contexts deliberately require `update` to access mutable UI state. `as_mut` panics in async contexts because carrying an ordinary mutable `App` reference across suspension would be unsound.

### Action, focus, and accessibility mechanics

- Action handling follows capture/bubble dispatch through the rendered element tree. Element action handlers stop propagation by default; `cx.propagate()` opts back into bubbling.
- Ordinary input event handlers have the opposite default: they propagate until `cx.stop_propagation()` is called.
- `Window::prevent_default()` currently controls default focus behavior on mouse down. It is independent of propagation and should not be documented as an event-stopping shortcut.
- `Window::on_action` registers during paint and is lower-level than an element’s `.on_action(...)`; framework comments tell component authors to prefer element handlers.
- A `FocusHandle` is retained state. Declaring a node focusable without tracking that handle on the rendered interactive root disconnects programmatic focus state from the element tree.
- Tab traversal is opt-in through tab-stop/tab-index metadata; examples bind Tab and Shift-Tab to explicit next/previous focus actions.
- Accessibility identity is based on stable global element IDs. Reusing the same global ID twice in a frame is a bug; retaining an ID across frames tells the accessibility tree it is the same node.
- `text!` derives an ID from its source location. Repeated mapped calls at one source location need an additional ID scope such as `.with_id(index)` to avoid duplicate accessibility identities.
- Accessible actions and GPUI command actions are separate mechanisms. `on_click` automatically exposes an accessible click action, while custom accessible operations must be registered deliberately.

### Zed UI layer details

- The `ui` crate is an opinionated semantic layer, not merely a bag of GPUI aliases. It combines shared component behavior, theme colors, density-aware spacing, typography, and accessibility defaults.
- `ButtonLike` exists for unusual controls, but its own documentation warns that using it freely can produce inconsistent Zed UI. This supports a “semantic component first” rule.
- `Button` and `ListItem` place accessibility metadata on the same actionable node that owns click behavior. Moving roles/labels to a decorative wrapper would change the accessible interaction target.
- Action-aware tooltips resolve the active key binding dynamically. Hard-coding shortcut text would go stale as the keymap or context changes.
- `DynamicSpacing` is not a convenience wrapper around `ui_density`; it is the policy boundary. Its source explicitly prohibits manually deriving values from density.
- `Color::Custom` is intentionally available but strongly discouraged. It is an escape hatch for externally sourced colors, not a normal theme choice.

### Element, list, and test internals

- The declarative element tree and its callbacks are rebuilt and dropped each frame. Retained application state belongs in entities, element state keyed by IDs, focus/scroll handles, or other explicit retained handles—not in callback captures assumed to persist.
- Custom elements execute request-layout, prepaint, and paint in sequence. Hitboxes are inserted during prepaint so later-painted siblings can win hit testing.
- A custom element’s accessibility node only participates when the element has an ID. Manual painting without stable identity therefore needs an explicit accessibility design.
- `uniform_list` assumes uniform item height and measures a representative item. Variable-height content belongs in `list`/`ListState`, where measurement policy is explicit.
- GPUI visual tests interact with the last rendered element tree. Dispatching an action against a window that has not rendered the relevant handler cannot validate the UI path.
- The test executor is deterministic and single-threaded. Awaiting external I/O can park it indefinitely; tests should mock external systems or use parking only with a deliberate reason.
- Synchronous `TestAppContext` updates flush queued effects automatically, while detached asynchronous effects require executor progress such as `run_until_parked`.

### Text input, globals, and fluent composition

- `ui_input::InputField` is the high-level Zed field, but it is editor-backed. Its `OnceLock` factory must be installed by editor initialization before `InputField::new`; using it in a small standalone GPUI binary pulls in the editor layer deliberately.
- `InputField` owns an erased editor handle, implements `Focusable`, and applies tab configuration to the editor’s focus handle during render. Its visible field shell tracks that configured handle.
- Masked input has a semantic icon button and tooltip, validation changes both border and explanatory label, and setter methods route into the embedded editor rather than duplicating text state.
- The lower-level `EntityInputHandler`/`ElementInputHandler` path is an IME protocol implementation: selection and replacement ranges are UTF-16, marked text must be represented, and bounds drive the system candidate window. Treating it as a simple `String` callback would be incorrect.
- `Global` is type-safe, but it is still process-wide mutable state. The framework documentation explicitly suggests a private global wrapper plus restricted accessor API where universal read/write access is too broad.
- `FluentBuilder` conditionals are generic construction helpers. Their high fan-in across GPUI/Zed supports using them to keep render trees linear instead of breaking chains into mutable temporary variables.
- Calling `.id(...)` on an `InteractiveElement` produces a `Stateful<Self>`, which is why stateful interaction and accessibility methods are only available after identity has been supplied.
- `div` is deliberately the all-in-one low-level container. Zed’s `ui` semantic components layer product rules on top; both statements are compatible, not competing recommendations.
- Assets are installed at the application boundary, while render-time loading uses window/app access and returns futures. The image element owns loading/fallback presentation rather than requiring a parallel view-level loading widget.
- `with_animation` is a declarative style transformation over time. Domain state still belongs in an entity; the animation helper is a frame-production mechanism.
- Cached views require definite sizing because cached layout is driven by the supplied style rather than measured from rendered content. Notification invalidates the backing entity; full window refresh bypasses reuse.

### macOS backend boundary

- `gpui_platform::application()` selects `MacPlatform` under `target_os = "macos"`. Ordinary application code should depend on this selector and GPUI abstractions, not construct AppKit or Metal objects directly.
- `MacPlatform::open_window` translates GPUI `WindowParams` to a `MacWindow`. Native anchored popups are currently rejected so GPUI callers can fall back to in-window popovers.
- macOS frame pacing is visibility-aware and display-specific. Each visible window owns a dispatch source, while a static registry owns one never-released `CVDisplayLink` per display to avoid teardown races.
- CoreVideo callbacks arrive off the main thread and only merge data into a main-queue dispatch source. GPUI/UI state is not mutated in the callback.
- The renderer consumes GPUI’s platform-neutral `Scene`; application components never submit Metal commands. Visual-test support can render offscreen and read textures without making renderer concerns part of the authoring model.
- Display-link lock ordering and immortal display entries solve historical crash classes. They are important internals for maintainers, but not reusable application-level patterns.

### Open investigations

- Collect exact line anchors for every source link that will enter the idiomatic reference.
- Audit whether the final reference overstates any Zed-specific `ui` convention as a GPUI framework invariant.
- Check example snippets for API consistency at the pinned revision without inventing unavailable initialization helpers.

## Classification decisions

- **Promote:** create long-lived mutable application/view state with `cx.new`, access it only through `Entity<T>` plus a context, and mutate through `update`.
- **Promote:** call `cx.notify()` after state changes that must invalidate observers or a rendered subtree; use typed `emit` for semantic events rather than treating notification as an event payload.
- **Promote:** store a `Subscription` when cancellation should follow the owner field’s lifetime; call `.detach()` only when the relationship should survive the local handle and naturally end with the participating entities/application listener set.
- **Promote:** use `cx.listener` for element callbacks that need mutable view state and do not return a value; use `cx.processor` for value-producing callbacks such as virtualized range rendering.
- **Promote:** use `Render` for entity-backed stateful views and `RenderOnce` for owned, stateless component values.
- **Promote with caution:** use low-level custom `Element` implementations only when declarative views cannot meet layout/paint/performance requirements; the framework documentation explicitly positions elements as the imperative register.
- **Promote:** use entity-scoped foreground tasks for async work that returns to UI state, keep the task handle when cancellation should follow the owner, and re-enter GPUI through async-context `update` after awaits.
- **Promote:** use `background_spawn` only for `Send + 'static` work that must leave the UI thread; return plain data and apply it on the foreground context.
- **Promote:** declare commands as namespaced actions, bind keys at app startup, scope bindings with key contexts, and handle them on the focused rendered element where possible.
- **Promote:** retain and render-track `FocusHandle`; make keyboard traversal, roles, labels, values, and stable unique IDs part of the control definition rather than post-hoc decoration.
- **Promote:** prefer `ui` semantic components, theme colors, and `DynamicSpacing` over raw interaction shells, literal colors, and hand-derived density values.
- **Promote:** use `uniform_list` plus a retained `UniformListScrollHandle` for uniform-height virtualization; use `list`/`ListState` when row heights vary.
- **Promote:** test through `#[gpui::test]`, construct a visual context from a rendered window, drive actions/input through GPUI’s test APIs, and explicitly advance detached async work.
- **Promote with dependency note:** use `ui_input::InputField` for Zed-integrated single-line form/search fields after editor initialization; implement `EntityInputHandler` only for a genuinely custom text engine with full IME/UTF-16 behavior.
- **Promote with restraint:** use typed `Global` state for application-wide services or policy, hide the marker type and expose narrow accessors when callers should not have unrestricted mutation, and keep local view state in entities.
- **Promote:** compose render-time variants with `FluentBuilder::{when, when_some, when_none, when_else, map}` and style elements through `Styled`; add an element ID before stateful interaction/accessibility features.
- **Promote:** start desktop applications through `gpui_platform::application()` so native backend selection stays outside feature code; treat AppKit, display-link, and Metal types as backend implementation details.
- **Promote:** install assets on the application, resolve/load them through GPUI, and provide loading/fallback elements; use declarative animation helpers for visual interpolation rather than storing frame-by-frame animation state in the domain model.
- **Promote with measurement:** cache an entity view only when the subtree is expensive, has a definite externally supplied size, and invalidation through `notify` is reliable.

## Final audit — 2026-08-12

- Parsed ledger: 62 source-span rows across 60 unique files.
- Reference: 20 sequential pattern sections and 80 pinned source line anchors.
- All absolute source files and line anchors resolve at the pinned revision.
- Both relative document links resolve.
- Markdown code fences are balanced; no trailing whitespace or unfinished-work markers remain in the two requested artifacts.
- The read-only source checkout remains unchanged apart from its pre-existing untracked `crates/gpui/.code-map/` directory.
