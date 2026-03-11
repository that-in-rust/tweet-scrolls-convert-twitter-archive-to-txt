# Decisions L1 - Tweet-Scrolls Tauri Desktop App (v200)

**Version:** 1.0
**Date:** 2026-03-11
**Status:** Approved

---

## Section: Objectives

### Objective 01: Prove to the world we can build a high-quality, superb Tauri Mac app

**Why This Matters:**
- Mac native apps command premium positioning
- Tauri + Rust = small bundle size (2-5MB) vs Electron (150MB+)
- Native performance and privacy-first architecture
- Demonstrates engineering excellence in Rust + modern frontend

**Success Criteria:**
- App feels native and responsive on macOS
- Bundle size under 10MB (without LLM model)
- Cold start under 2 seconds
- Beautiful, intuitive UI that delights users
- 4.5+ star rating if released on App Store

**Key Risks:**
- Tauri learning curve for frontend integration
- Cross-platform considerations for future Windows/Linux support

---

### Objective 02: Build the most differentiated Twitter Archive Analysis tool possible

**Differentiation Pillars:**
1. **Local LLM Integration** - Users can chat with their archive naturally
2. **CPU Optimization** - Maximize local compute, minimize cloud dependency
3. **Privacy-First Architecture** - Data never leaves the user's machine
4. **Rich Visualizations** - Beautiful charts, network graphs, timelines
5. **Complete Archive Support** - Handle 100GB+ archives efficiently

**Why This Matters:**
- Competitors require cloud upload (privacy risk)
- No native Mac app exists in this space
- LLM integration for personal data is novel and valuable
- Power users want local processing, not SaaS subscriptions

**Success Criteria:**
- Process 100MB archives in under 30 seconds
- Support all Twitter archive file types (tweets, DMs, likes, followers, etc.)
- Enable natural language queries via local LLM
- Generate actionable insights and beautiful visualizations

---

## Section: Architecture Decisions

### Decision 001: Use TursoDB (libSQL) as Embedded RDBMS

**Context:**
Need a fast, embedded database for querying processed archive data. Options considered:
- SQLite (traditional)
- TursoDB / libSQL (SQLite-compatible, edge-optimized)
- DuckDB (analytics-focused)
- Custom index files (current approach)

**Decision:**
Use **TursoDB (libSQL)** as the embedded database.

**Rationale:**
1. **GTM Leverage:** Turso is used by CodeMogger and other successful tools - provides ecosystem familiarity and potential partnership opportunities
2. **SQLite-Compatible:** Easy migration path, familiar SQL API
3. **Edge-Optimized:** Designed for local-first, offline-capable applications
4. **Rust Bindings:** Good Rust support via `libsql` crate
5. **Performance:** Optimized for read-heavy workloads (our use case)
6. **Future-Proof:** Can sync to Turso cloud for paid tier if desired

**Implementation:**
```rust
// Cargo.toml
libsql = "0.9" // Turso/libSQL Rust bindings

// Schema design
// - tweets table (id, text, created_at, engagement_metrics, ...)
// - dm_conversations table (conversation_id, messages_json, ...)
// - mentions table (user_id, username, mention_count, ...)
// - analytics_cache table (query_hash, result_json, timestamp)
```

**Trade-offs:**
- (+) Better GTM story with Turso ecosystem
- (+) Future sync capabilities for premium tier
- (+) Active development and community
- (-) Slightly more complex than vanilla SQLite
- (-) Additional dependency

---

### Decision 002: Single-ZIP Input - Users Only Provide Archive ZIP

**Context:**
How should users provide their Twitter data? Options:
1. Require extracted folder with specific file structure
2. Accept only the ZIP file, extract internally
3. Accept both ZIP and extracted folder

**Decision:**
Users **only provide the Twitter archive ZIP file**. All extraction, parsing, and processing happens automatically.

**Rationale:**
1. **Simplicity:** One drag-drop action = zero friction onboarding
2. **Privacy:** All extraction happens locally, no user intervention needed
3. **Error Prevention:** Users can't accidentally select wrong folders
4. **Delight Moment:** Drop ZIP, see progress bar, get insights

**User Experience:**
```
┌─────────────────────────────────────────────────────────┐
│                                                         │
│     📂 Drop your Twitter archive ZIP here              │
│                                                         │
│     or click to browse                                  │
│                                                         │
│     ─────────────────────────────────────────────────   │
│     ✓ Your data stays on your Mac                      │
│     ✓ No cloud upload, ever                            │
│     ✓ Process locally with full privacy                │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

**Processing Pipeline:**
```
ZIP Input → Extract to temp → Parse all .js files → Build TursoDB → Clean temp → Ready
```

**Privacy Guarantees:**
1. Extraction to app sandbox only
2. No network calls during processing
3. Database stored in user's Application Support folder
4. User can delete all data from app settings
5. Clear data retention: "Your data, your machine, your control"

**Implementation Details:**
- Use Rust `zip` crate for extraction
- Process in streaming fashion for large archives
- Show real-time progress (file by file)
- Handle multi-part archives (direct-messages-part1.js, etc.)

**Trade-offs:**
- (+) Maximum user simplicity
- (+) Privacy-first by design
- (+) Error-proof input handling
- (-) Larger temp storage during extraction
- (-) Must handle all ZIP edge cases

---

## Section: Technical Decisions

### Decision 003: Frontend Framework - React + TypeScript + shadcn/ui

**Context:**
Need a modern, maintainable frontend for Tauri app.

**Decision:**
Use **React + TypeScript** with **shadcn/ui** component library.

**Rationale:**
1. shadcn/ui provides beautiful, accessible components
2. Native-feeling design that matches macOS aesthetics
3. Large ecosystem and community support
4. Good Tauri integration examples exist

---

### Decision 004: Local LLM - Ollama Integration (Primary), Bundled Fallback

**Context:**
How to enable LLM-powered analysis of archives?

**Decision:**
- **Primary:** Detect and use Ollama if installed (localhost:11434)
- **Fallback:** Bundle a small model via llama.cpp for basic queries

**Rationale:**
1. Ollama is popular among Mac power users (our target)
2. User controls model choice (Llama 3.2, Mistral, etc.)
3. No bundle bloat for users who don't need LLM features
4. Fallback ensures feature works for all users

---

### Decision 005: Visualization - Recharts + D3.js

**Context:**
Need beautiful, performant charts and graphs.

**Decision:**
- **Recharts** for standard charts (bar, line, area)
- **D3.js** for custom network graphs (mention circles)

**Rationale:**
1. Recharts is React-native, declarative, easy to use
2. D3.js enables custom visualizations like twitter-circle style network graphs
3. Both perform well with large datasets

---

## Section: Reference Research

### Patterns from twitter-circle (sankalp1999)

**Learnings:**
1. **Weighted Scoring** - Recency-weighted interaction scores for ranking
2. **User ID Mapping** - Extract mentions from tweets.js to build ID→username map
3. **DM Processing** - Multi-part DM file handling
4. **D3.js Circles** - Concentric visualization of network
5. **Chart.js Graphs** - Monthly message counts over time

**Improvements We'll Make:**
1. No Puppeteer/scraping - pure local processing
2. Native Mac app instead of web server
3. LLM integration for natural queries
4. Embedded database for fast queries
5. No external dependencies at runtime

---

## Appendix: File Locations

| Item | Path |
|------|------|
| Research Notes | `/docs/v200research01.md` |
| Reference Repo | `/reference_repos/twitter-circle/` (gitignored) |
| Real Data | `/REALDATA/` (gitignored) |
| This Document | `/docs/DecisionsL1.md` |

---

*Document created for Tweet-Scrolls v200 Tauri Desktop App initiative.*
*Next: Create detailed technical design documents (L2 decisions).*
