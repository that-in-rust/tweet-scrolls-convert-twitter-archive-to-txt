# v200 Research: GTM Strategy & Tauri Desktop App Vision
**Date:** 2026-03-11
**Status:** Research Complete

---

## Executive Summary

This document captures research on go-to-market (GTM) strategies for Tweet-Scrolls, with a focus on building a Tauri desktop application as the flagship product. The research combines insights from Shreyas Doshi's product thinking principles, MCP (Model Context Protocol) ecosystem analysis, and developer tool distribution patterns.

---

## Part 1: GTM Strategy Research

### 1.1 The Core Problem (Customer Problem Stack Ranking)

**Critical Problem Being Solved:**
> "My Twitter archive is a 50GB JSON dump that's completely unusable. I want to revisit conversations, analyze my patterns, or feed it to an AI—but I can't."

Tweet-Scrolls solves a **high-importance, underserved problem**:
- Twitter/X's API restrictions killed most archive tools
- Users who download their data get raw JSON that's incomprehensible
- Few alternatives exist, and those that do require cloud upload (privacy risk)

### 1.2 Competitive Landscape

| Competitor | Limitation |
|------------|------------|
| Tweet Archivist, Brandwatch | SaaS, requires upload to their servers, expensive ($49-800/mo) |
| Native Twitter Analytics | 90-day limit, no archive support |
| API-based tools | Killed by X's API pricing ($42K/mo for Enterprise) |

**Our Differentiation:**
1. **Privacy-first**: All local processing—data never leaves user's machine
2. **LLM-ready output**: Auto-chunked files designed for AI consumption
3. **Open source**: Free, auditable, trustworthy
4. **Complete archive support**: Handles 50K+ tweets efficiently

### 1.3 GTM Strategy Options (Ranked by Impact)

#### Option A: MCP Server Integration (Highest Impact for AI Ecosystem)
- MCP is becoming the "USB-C for AI" (100M+ monthly downloads)
- Users can say to Claude/ChatGPT: *"Analyze my Twitter archive"* and it just works
- Positions at the center of the AI agent ecosystem
- Creates viral discovery through MCP directories

**MCP Registry Submission Targets:**
- Official MCP Registry (via npm + `mcp-publisher`)
- findmcp.dev (2-min submission)
- mcpserve.com

#### Option B: Multi-Platform CLI Distribution
Expand beyond Cargo:
- PyPI (`pip install tweet-scrolls`)
- npm (`npx tweet-scrolls`)
- Homebrew (macOS)
- AUR (Arch Linux)
- GitHub Releases (pre-built binaries)

#### Option C: LLM Integration Partnerships
- Claude Desktop (via MCP)
- Obsidian (plugin for PKM users)
- Notion (research integration)
- AI note-taking tools (Mem, Reflect)

#### Option D: Content-Led Growth
Content pillars:
1. "What your Twitter archive reveals about you"
2. Technical deep-dives on Rust + Tauri
3. Use case tutorials (AI analysis, legal discovery, personal archiving)

---

## Part 2: Tauri Desktop App Vision

### 2.1 Why Tauri?

**Strategic Advantages:**
- Native Mac app = premium positioning
- Rust backend = reuse existing code
- Small bundle size (2-5MB vs Electron's 150MB+)
- Full filesystem access for local processing
- Perfect for privacy-first positioning

**Market Gap:** No native Mac app exists for Twitter archive exploration.

### 2.2 Technical Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    TAURI APP (Mac Native)                    │
├─────────────────────────────────────────────────────────────┤
│  Frontend (React/Svelte + shadcn/ui + Recharts/D3)          │
│  ┌─────────────┐ ┌─────────────┐ ┌──────────────────────┐   │
│  │ Dashboard   │ │ Thread      │ │ LLM Chat Interface   │   │
│  │ (Charts)    │ │ Explorer    │ │ (Analyze your data)  │   │
│  └─────────────┘ └─────────────┘ └──────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│  Tauri IPC (Rust ↔ TypeScript bridge)                       │
├─────────────────────────────────────────────────────────────┤
│  Rust Backend (Existing code + new features)                │
│  ┌─────────────┐ ┌─────────────┐ ┌──────────────────────┐   │
│  │ Archive     │ │ Thread      │ │ Analytics Engine     │   │
│  │ Parser      │ │ Builder     │ │ (Timeline, Stats)    │   │
│  └─────────────┘ └─────────────┘ └──────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│  Local LLM Integration                                       │
│  ┌─────────────┐ ┌─────────────┐                            │
│  │ Ollama API  │ │ llama.cpp   │ (User's choice)           │
│  │ (localhost) │ │ bundled     │                            │
│  └─────────────┘ └─────────────┘                            │
└─────────────────────────────────────────────────────────────┘
          │
          ▼
    ┌───────────┐
    │ Embedded  │ (SQLite/TursoDB for fast queries)
    │    DB     │
    └───────────┘
```

### 2.3 Key Features

| Feature | Description | User Value |
|---------|-------------|------------|
| **Archive Import** | Drag-drop zip or folder | One-click setup |
| **Timeline Dashboard** | Activity heatmap, posting patterns | Self-awareness |
| **Thread Explorer** | Browse reconstructed conversations | Memory recall |
| **Search Everything** | Full-text search across all data | Find anything |
| **LLM Chat** | "What did I tweet about X in 2024?" | AI-powered insights |
| **Analytics** | Top tweets, engagement trends, word frequency | Growth insights |
| **Export** | Generate reports, shareable summaries | Portability |

### 2.4 Local LLM Integration Options

**Option A: Ollama (Recommended for Mac)**
```bash
ollama pull llama3.2
ollama serve  # localhost:11434
```
- Pros: User controls model choice, no bundle bloat
- Cons: Requires separate installation

**Option B: Bundled llama.cpp**
- Pros: Zero setup, works offline
- Cons: Larger app size (~2-4GB with model)

**Option C: Hybrid**
- Detect Ollama, fallback to bundled mini-model

### 2.5 Visualization Stack

- **Recharts** or **Apache ECharts** (React charts)
- **D3.js** for custom network graphs
- **shadcn/ui** for beautiful native-feeling components
- **Framer Motion** for smooth animations

**Visualization Examples:**
- Activity heatmaps (posting patterns)
- Engagement graphs over time
- Word clouds from tweets
- DM conversation frequency charts
- Top mentions/replies network graph

---

## Part 3: Research Sources

### MCP & AI Ecosystem
- MCP Official Blog: "One Year of MCP" (Nov 2025)
- Model Context Protocol 2025 Guides (multiple sources)
- Microsoft Developer Blog: "Connect Once, Integrate Anywhere"
- MCP Registry quickstart documentation

### Developer Tool GTM
- Unusual VC: "Building GTM for an Open Source Company"
- QC Growth: "Top GTM Strategies for DevTool Companies (2025)"
- Maximize Partners: "Using Open Source as a Funnel"
- Frontlines.io: GTM lessons from Unleash

### Shreyas Doshi Product Thinking
- "Good Product Strategy, Bad Product Strategy"
- Customer Problem Stack Ranking methodology
- "Wow" vs "Table Stakes" feature prioritization

### Tauri & Local LLM
- "Building Local LM Desktop Applications with Tauri" (Medium)
- GitHub: ollama-chat-tauri, tauri-local-lm
- Tauri UI templates and visualization examples

### Twitter Archive Tools Market
- Tweet Archivist comparison articles
- Socialinsider analytics tools review
- MetricsWatch competitor analysis

---

## Part 4: Real Data Analysis

### User Archive Stats (amuldotexe)
- **Account Created:** 2019-05-25
- **tweets.js:** 100MB
- **direct-messages.js:** 100MB
- **like.js:** 68MB
- **direct-message-headers.js:** 67MB
- **Additional files:** followers, grok chats, articles, ad engagements, blocks, mutes, etc.

### Data Structure Observations
- Twitter archives use `window.YTD.<type>.part0 = [...]` format
- Rich metadata: timestamps, edit history, reactions, URLs
- DM structure includes conversationId for threading
- Tweet structure includes full_text, entities, engagement metrics

---

## Next Steps

1. Review reference implementations (twitter-circle repo)
2. Define L1 architecture decisions
3. Plan Tauri app milestones
4. Set up MCP server development

---

*Research compiled for Tweet-Scrolls v200 Tauri Desktop App initiative.*
