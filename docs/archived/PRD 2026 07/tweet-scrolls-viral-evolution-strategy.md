# Tweet-Scrolls: Viral Evolution Strategy and Alternate Product Timelines

Date: 2026-07-10

Status: Product strategy exploration, not an implementation commitment

Primary objective: Maximum organic adoption and cultural relevance

Business model constraint: Free and open source. Monetization is not an objective.

Platform constraint: The full Twitter/X archive is commonly a large ZIP and is best processed on a desktop-class machine. Mobile can consume and share outputs, but should not be the primary importer.

## Executive Decision

Tweet-Scrolls should not evolve into a better CSV converter, a generic analytics dashboard, or an AI chat box over tweets.

It should evolve into:

> A private time machine for your internet life.

The desktop application is the vault. It imports the unopened X archive, reconstructs the user's history locally, and creates moments of recognition: eras, turning points, recurring ideas, important people, forgotten work, and memories. The web is the invitation layer. It displays only artifacts that the user deliberately publishes and helps the next person request their archive and install the desktop app.

The recommended first product is a desktop-first, local-first experience called, provisionally, "Your Twitter Eras." In one session it should:

1. Accept the original archive ZIP.
2. Reveal useful results progressively instead of waiting for the entire import.
3. Produce a short, emotionally resonant story about the user's years on Twitter.
4. Let the user search and revisit the evidence behind every claim.
5. Generate beautiful, inspectable, privacy-safe cards or short videos.
6. Put an understated invitation on each artifact: "Made privately from my X archive."

The acquisition wedge is identity and nostalgia. The retention engine is private search, resurfacing, and reuse. The later network effect is a mutual, consent-based "Shared Scroll" between two people. Public community archives are a possible fourth act, not the opening move.

The robust strategy is therefore a barbell:

- Public side: expressive artifacts that travel.
- Private side: a genuinely useful memory vault that earns long-term trust.

Virality without the private utility becomes a once-a-year novelty. Utility without the public artifacts becomes a respected niche tool. The combination can become a durable open-source consumer product.

## Naming Assumption

The prompt says "Shreyas Joshi." I am interpreting this as a reference to product leader Shreyas Doshi and using a lens inspired by his public product-decision questions:

- What is the user need?
- Why is it important?
- What are the expected solutions?
- What are the unexpected solutions?
- What are the known tradeoffs?
- What remains unknown?
- How can we make it known cheaply?

This document applies that lens. It does not claim his endorsement or attempt to imitate his personal voice.

## Premise Check

### Premises that are sound

- A Twitter/X archive contains unusually rich personal history. X says the machine-readable archive can include posts, Direct Messages, media, follows, followers, address-book information, Lists, ad activity, and more.
- Large archives and sensitive DMs strongly favor local processing.
- The current Rust code is useful technical seed capital. It already parses tweets and DMs, reconstructs some threads, calculates timing patterns, and emits text and CSV.
- Open source is an advantage in this category because trust is part of the product, not merely a legal attribute.
- Mobile should not be the initial processing environment. Mobile should be a viewer, sharing surface, or later companion.

### Premises that need correction

- "Desktop or web" is a false binary. The strongest product shape is a desktop vault plus a web distribution layer.
- "Archive viewer" is not enough. X already includes a browser-readable archive, and several third-party tools offer search, viewing, or analytics.
- "More analytics" is not inherently shareable. Counts and charts are useful to the owner but usually boring to everyone else.
- "AI over my archive" is not a differentiated product. It is an enabling capability. Leading with it creates trust, accuracy, cost, and commodity-positioning problems.
- Local processing alone does not make sharing safe. The share boundary needs its own product architecture, redaction model, previews, and provenance.
- The current codebase is not yet ready to make strong anonymization claims. Its anonymization module contains tests that copy IDs unchanged, and the dependency list contains no Blake3 library despite README claims.

### The real fork in the road

The decisive question is:

> What does one user create that makes another person want to request and import their own archive?

That object is not the archive, the dashboard, or the app. It is a "Scroll": a small, expressive, evidence-backed artifact derived from the archive and explicitly approved by its owner.

## Decision Frame

### Desired outcome

Create a free, open-source product that becomes the default way people privately recover, understand, and selectively share their Twitter history.

### What maximum adoption means

Adoption is not GitHub stars, page views, or ZIP uploads. A meaningful adoption event is:

> A person successfully imports an archive, experiences at least one personally meaningful reveal, and either returns to the private vault or creates a safe artifact.

### What maximum virality means

Virality is not a launch spike. A meaningful viral event is:

> A shared artifact causes another person to begin and eventually complete their own archive import.

The full loop is:

~~~mermaid
flowchart TB
    A["User imports archive locally"] --> B["App creates a meaningful private reveal"]
    B --> C["User approves a safe Scroll"]
    C --> D["Scroll travels through X, Bluesky, blogs, or group chats"]
    D --> E["Viewer recognizes a version of their own past"]
    E --> F["Viewer requests archive and installs app"]
    F --> A
~~~

### Hard constraints

- Raw archives and raw DMs do not leave the device by default.
- The app must handle very large ZIPs without loading entire files into memory.
- Every public output must be explicit, previewable, and reversible where hosted.
- The parser must tolerate archive format drift.
- No user account should be required for the private desktop experience.
- The core remains useful without AI or a network connection.
- Open-source users must be able to audit what the app reads, derives, and exports.

### What would count as failure

- The product gets attention but users abandon the archive-request or install flow.
- The first useful result takes so long that users close the app.
- Shared outputs expose DMs, identifiers, location, deleted content, or private accounts unexpectedly.
- Insights feel generic, judgmental, or unverifiable.
- It becomes a one-time Wrapped clone with no reason to return.
- It becomes an expert-only data tool that normal users cannot install.
- A public corpus creates moderation and consent obligations before the project can support them.
- The product depends on a paid X API or a hosted AI service to deliver basic value.

## Current Product Reality

### Assets already present

| Asset | Current evidence | Strategic value |
| --- | --- | --- |
| Local Rust processing | The CLI reads archive files directly and writes local outputs | Strong foundation for a trustworthy desktop core |
| Tweet parsing | tweets.js is parsed into typed records | A base for history, search, and public-post insights |
| Reply grouping | Archived tweets are grouped into thread-like structures | A base for thread rediscovery and creator reuse |
| DM parsing | Direct-message conversations and timestamps are parsed | A rare and emotionally valuable private-memory asset |
| Temporal analysis | Activity density, peak periods, and response times are calculated | A base for eras and relationship timelines |
| CSV and text export | Multiple durable formats are emitted | Useful for interoperability and power users |
| File splitting | Large text outputs can be split for downstream LLM use | Shows awareness of archive scale and portability |

### Product and trust gaps

| Gap | Current evidence | Why it matters |
| --- | --- | --- |
| No direct ZIP import | The CLI expects an extracted archive directory containing tweets.js | The highest-friction step remains with the user |
| Whole-file reads | Tweet and DM pipelines call read_to_string before parsing | Very large archives can create long waits or memory failure |
| No durable local index | There is no SQLite or full-text-search layer | Every rich experience would otherwise repeat expensive parsing |
| No consumer UI | The primary path is a command-line process | This caps adoption to technical users |
| No public/share boundary | Outputs are raw text and CSV | There is no safe, delightful viral object |
| Anonymization claim is inaccurate | The anonymization tests preserve IDs unchanged | A serious trust risk, especially for DMs |
| Tweet timeline is incomplete | RelationshipAnalyzer explicitly skips tweet events | Cross-channel relationship claims would be misleading |
| Incomplete archive coverage | Likes, bookmarks, follower history, and media experiences are not integrated | Many high-value stories are unavailable |
| One known failing test | On 2026-07-10, cargo test had 81 passes and one file-splitter failure | Reliability work is needed before consumer distribution |
| Missing open-source scaffolding | No license, security policy, contribution guide, or code of conduct was found | Adoption and contribution expectations are unclear |

### Critical interpretation

The codebase is a processing engine, not yet a product. That is a favorable position. The difficult domain parsing has begun, but the consumer experience is not locked into the current output model.

The architecture should preserve the Rust parsing work while replacing "files are the product" with:

> Archive in, private model built, meaningful experiences rendered, safe artifacts deliberately exported.

## The User Need Hierarchy

The archive is a means, not the need. Users hire the product for different emotional and practical jobs.

### Level 1: Recovery

"Give me my history in a form I can actually open, search, and keep."

This is functional and broad. It creates trust and baseline adoption but weak virality.

### Level 2: Retrieval

"Help me find the post, thread, person, image, or conversation I half remember."

This creates recurring utility and a strong reason to keep the app installed.

### Level 3: Reflection

"Show me how I changed, what I cared about, and which periods shaped me."

This is the emotional center of the product and the strongest source of shareable identity.

### Level 4: Reuse

"Turn my old work into material I can publish, write from, or migrate elsewhere."

This is especially valuable to creators, writers, researchers, and people leaving X.

### Level 5: Relationship

"Help me remember the people and conversations that mattered, without violating their privacy."

This is highly emotional and potentially networked, but also the highest-risk private-data surface.

### Level 6: Collective memory

"Let our community preserve and explore a shared period of internet history."

This has the largest institutional and network upside, but introduces consent, governance, moderation, and deletion obligations.

## Expert Lenses

### Consumer product and identity lens

People share artifacts that say something legible about who they are. A chart of posts by weekday is information. "I spent 2013 learning in public, 2017 arguing about startups, and 2024 building in Rust" is identity.

Implication: every shareable insight needs a narrative, a visual signature, and evidence the owner can inspect.

### Growth and incentives lens

The archive-request delay breaks a normal instant referral funnel. A viewer may be interested now but receive the archive later.

Implication: acquisition must support a delayed loop:

1. The viewer sees a Scroll.
2. The viewer requests the archive.
3. The viewer installs the desktop app immediately or saves a clear return path.
4. The app can open a demo and explain what will happen.
5. When the ZIP arrives, drag-and-drop completes the journey.

The project should optimize the return path as seriously as the share card.

### Privacy and user-advocacy lens

The most emotionally valuable data is often the least shareable. DMs include another person's words, deleted messages, media, locations, and relationship context. "The user owns the archive" does not mean the user has uncomplicated moral permission to publish everything in it.

Implication: raw DMs are private by default. Public artifacts should start from public posts. Relationship artifacts require extra consent controls and ideally participation from both people.

### Open-source ecosystem lens

An open parser, stable normalized schema, plugin API, and transparent artifact format can create contributor-driven distribution. The project can become infrastructure for tools it never builds itself.

Implication: make import adapters, insight recipes, render themes, and exporters independently extensible.

### Skeptical product lens

The weak premise is that people will do several minutes of settings work, wait for an archive, download a large ZIP, install an unfamiliar desktop app, and then publicly share the result.

The skeptical conclusion is not "do not build it." It is:

- The reveal must be much better than an analytics dashboard.
- Trust must be visible before import, not buried in a privacy policy.
- The first public launch should test artifact desirability before building the full explorer.
- The desktop installer and archive request are part of the product, not documentation chores.

## Chosen Product Thesis

### Positioning

For people with years of life on Twitter/X, Tweet-Scrolls is a private desktop time machine that turns an inaccessible archive into a searchable personal history and a few beautiful stories worth sharing. Unlike upload-based analytics tools, the archive stays on the user's machine and only explicitly approved Scrolls leave it.

### The key product promise

> Your archive stays private. Your memories become useful. You decide what travels.

### The wedge

"Your Twitter Eras" is a guided reveal with five to seven chapters:

1. The beginning: first posts, first recurring topics, earliest surviving media.
2. The eras: periods with distinct vocabulary, people, topics, and posting rhythms.
3. The turning points: changes in interests, work, location only when explicitly stated, or communities.
4. The rediscoveries: old posts or threads that still sound like the person today.
5. The cast: public accounts most present in the user's public conversation history.
6. The signature: recurring words, formats, jokes, links, or creative work.
7. The now: a user-editable interpretation of how their public voice changed.

Every chapter must provide "show me the receipts." The app should distinguish deterministic facts from model-generated interpretation.

### The retention product

After the reveal, the user lands in a local library:

- Instant full-text search.
- Calendar and era navigation.
- Thread and media views.
- "On this day" resurfacing.
- Collections and annotations.
- Exports to Markdown, static HTML, JSON, CSV, and image/video formats.
- Optional semantic search and local AI, never required for basic use.

### The second act

"Shared Scrolls" allow two users to create a mutual artifact:

- Both import their own archive.
- One sends an invitation naming the other person.
- Each sees exactly which derived fields or selected posts would be compared.
- Both approve.
- A shared artifact is generated from the intersection.

Possible outputs include:

- The year we first crossed paths.
- Topics we kept returning to.
- Our public conversation eras.
- A mutually selected set of messages or posts.
- A private two-person replay that is not public by default.

This loop has greater viral potential than a generic invite because the invited person is needed to unlock something personally meaningful.

## Product Principles

### 1. Local is a visible feature

Show a network indicator, a plain-language data map, and an "offline mode" that users can verify. Do not rely on the phrase "local-first" alone.

### 2. Progressive value beats a progress bar

Large imports should reveal safe, cheap facts while deeper indexing continues:

- Archive date range.
- Number of public posts and DMs.
- First surviving post.
- Most active years.
- Earliest media.

The goal is a first meaningful reveal in under 90 seconds for a typical archive, even if the full import takes longer. This is a product target to test, not a current guarantee.

### 3. Receipts before interpretation

Every nontrivial claim links to the underlying posts or events. Model-generated language is labeled as interpretation. Users can edit or discard it.

### 4. Sharing is a compiler boundary

The app should not share screenshots of the private UI. It should compile a new artifact from an allowlisted set of fields. The artifact inspector shows every included text fragment, identifier, timestamp, and media item before export.

### 5. Public posts first, DMs last

The default share studio should not even offer DM content. Enabling it should require a separate flow and stronger warnings. Pair artifacts should favor mutual participation.

### 6. Delight should not become diagnosis

Avoid personality scores, mental-health inference, political labeling, relationship-health scores, or claims about "best friends." These are inaccurate, emotionally risky, and difficult to explain.

### 7. Durable formats are part of the mission

The user should be able to leave Tweet-Scrolls with a useful archive. Local SQLite, JSON, Markdown, static HTML, images, and documented schemas prevent lock-in.

### 8. No account for private value

Accounts are needed only for optional hosted artifacts, synchronization, or community participation. The desktop vault works without login.

### 9. Open source must reduce perceived risk

Source availability alone is insufficient. Signed builds, reproducible build instructions, a threat model, a security policy, test fixtures, and a network-access ledger make openness legible to non-developers.

## User Journeys

### Journey summary

| User | Trigger | Core job | First "aha" | Retention | Viral object |
| --- | --- | --- | --- | --- | --- |
| Nostalgic veteran | Sees a friend's era card | Understand a decade of online life | First post and era transition | On-this-day memories | Personal era story |
| Creator or writer | Needs old material | Search and repurpose past work | Finds a forgotten strong thread | Collections and exports | "From my archive" carousel |
| Private memory keeper | Wants old DMs or relationships | Revisit safely | Restored conversation timeline | Private calendar and people view | Usually none; perhaps a private invite |
| Pair of old friends | Receives a Shared Scroll invite | Reconstruct mutual history | First shared interaction | Pair replay and anniversaries | Mutual-consent Shared Scroll |
| Deactivating user | Plans to leave X | Preserve and migrate | Searchable local copy works | Archive ownership and exports | Exit capsule or static site |
| Researcher or historian | Studies a community or era | Create reproducible corpus | Normalized, queryable data | Saved projects and citations | Opt-in public collection |
| Open-source builder | Encounters unsupported format or idea | Extend the engine | First plugin renders locally | Contributions and plugins | New insight template or importer |

### Journey 1: The nostalgic veteran

Trigger:

They see a visually specific card such as "My Twitter life had five eras" from someone they know.

Flow:

1. The artifact opens well on mobile.
2. It explains that the archive was processed privately.
3. The call to action says "Request your archive" and "Install while you wait."
4. The desktop app opens with a sample archive and a short privacy tour.
5. The user drags the unopened ZIP when it arrives.
6. The first reveal appears while indexing continues.
7. The user explores receipts and edits labels such as "The learning-in-public era."
8. The share studio proposes only public-post artifacts.
9. The user exports a vertical card set or short video and posts it.

Aha moment:

"I had forgotten that this version of me existed."

Viral mechanism:

Recognition plus curiosity. Viewers compare themselves to the person and want their own answer.

Failure risk:

Generic era labels feel like an astrology generator. The cure is evidence, specificity, and user editing.

### Journey 2: The creator or writer

Trigger:

They remember a thread, idea, joke, prediction, or image but cannot find it.

Flow:

1. Import and build a full-text index.
2. Search by phrase, topic, person, date, link domain, or media.
3. Save results into a collection.
4. Turn the collection into Markdown, a newsletter draft, a carousel, or a static page.
5. Cite the original dates and URLs where available.
6. Optionally publish a "From my archive" collection.

Aha moment:

"I already wrote the raw material for this essay six years ago."

Retention mechanism:

Repeated search and reuse, not novelty.

Viral mechanism:

Published collections and creator tutorials show practical value. The artifact is content, not merely an advertisement for the app.

Failure risk:

If search is slow or thread reconstruction is inaccurate, the creator will return to command-line tools or platform search.

### Journey 3: The private memory keeper

Trigger:

They want to recover an old DM conversation, remember when a relationship changed, or revisit messages from someone no longer reachable.

Flow:

1. Import with a clear warning that DMs will remain in the private vault.
2. Choose whether DM text is indexed.
3. Browse conversations by time, participant, and media.
4. See response rhythms and gaps without reductive relationship scores.
5. Add private annotations or hide painful periods.
6. Export an encrypted or local-only memory capsule if desired.

Aha moment:

"I can finally read this history in order, with context."

Retention mechanism:

Private search, anniversaries, and personal curation.

Viral mechanism:

Weak by design. This journey builds trust and word-of-mouth, not public sharing.

Failure risk:

Accidental exposure would damage the entire project. DM features must not be used as growth bait.

### Journey 4: The pair of old friends

Trigger:

One user sees a private prompt: "You and this person appear across eight years. Create a Shared Scroll?"

Flow:

1. User A chooses a person and reviews a proposed data boundary.
2. User A sends an invitation containing no raw message text.
3. User B installs the app and imports their archive.
4. Both users see a preview of the intersection and approve individual items or categories.
5. The app generates a private replay first.
6. They can jointly approve a public version.

Aha moment:

"This is the story of how we became friends, and both of us chose what it says."

Retention mechanism:

Shared anniversaries and private pair collections.

Viral mechanism:

A specific invitation is necessary to unlock the experience. This is a genuine collaborative loop, not a contact-spam mechanic.

Failure risk:

Identity matching, unequal archive completeness, consent withdrawal, and emotionally sensitive content make this unsuitable for version one.

### Journey 5: The person leaving X

Trigger:

They intend to deactivate, have been suspended, or no longer trust the platform.

Flow:

1. A guided "Exit Kit" explains how to request the archive.
2. The app validates archive completeness.
3. It creates a searchable local copy and durable exports.
4. It produces a static personal site or Markdown collection.
5. It maps links and selected content into formats useful for a blog, Bluesky, Mastodon, or an Obsidian vault.
6. It verifies that the user can open the export without Tweet-Scrolls.

Aha moment:

"My history is mine even if my account disappears."

Retention mechanism:

Long-term archive access and periodic export verification.

Viral mechanism:

Exit guides, migration moments, and public static archives create event-driven adoption.

Failure risk:

Migration APIs and policies change. Durable export should remain the core promise.

### Journey 6: The researcher or community historian

Trigger:

They want to study a scene, movement, conference, fandom, or public conversation over time.

Flow:

1. Import one or more archives with explicit owner permission.
2. Normalize into a documented event schema.
3. Create a project with repeatable filters.
4. Export a provenance manifest and queryable dataset.
5. Ask participants to opt specific public posts into a collection.
6. Publish a static or hosted exhibit with removal and correction procedures.

Aha moment:

"This is a living, inspectable record rather than a pile of incompatible ZIPs."

Retention mechanism:

Saved projects, updates, citations, and new contributions.

Viral mechanism:

Each contributor improves the collective artifact and invites peers from the same community.

Failure risk:

Public-post availability is not the same as informed archival consent. Governance must precede scale.

### Journey 7: The open-source builder

Trigger:

They encounter an unsupported archive variant, have an idea for an insight, or want an exporter for another tool.

Flow:

1. Run a synthetic archive fixture.
2. Implement one documented plugin interface.
3. Preview the result in a local development gallery.
4. Add deterministic tests and privacy metadata.
5. Publish through a curated community registry or submit upstream.

Aha moment:

"I added a new experience without touching the private vault internals."

Retention mechanism:

Maintainer relationships, plugin users, and clear contribution ladders.

Viral mechanism:

Themes, importers, and insight recipes bring their own communities.

Failure risk:

An unrestricted plugin can exfiltrate private data. Plugins need permissions, review signals, and visible network capabilities.

## The Viral Object: A Scroll

A Scroll is a deliberately generated derivative, not a screenshot of the private app.

### A strong Scroll has seven properties

1. It is about the user, not about Tweet-Scrolls.
2. A stranger can understand it in under three seconds.
3. It contains at least one surprising and specific observation.
4. It includes enough evidence to feel true.
5. It omits private material by construction.
6. It looks good in a vertical feed, a group chat, and a blog.
7. It leaves a curiosity gap: "What would mine look like?"

### Candidate Scrolls

High-potential:

- My Twitter Eras.
- My first post versus the person I became.
- Twelve years in twelve posts.
- Ideas I kept returning to.
- My public conversation constellation.
- The thread I forgot I wrote.
- My internet hometowns: communities and topics, not inferred physical location.
- Predictions I got right, wrong, or changed my mind about, with manual confirmation.
- My vocabulary migration.
- The links and domains that shaped my years.

Useful but less viral:

- Posting heatmap.
- Post count by year.
- Top hashtags.
- Most-liked posts.
- Export statistics.

Too risky for default sharing:

- Most-messaged DM contacts.
- "Best friend" rankings.
- Relationship health or response-time judgments.
- Deleted or private-account content.
- Location inference.
- Political, medical, sexual, or mental-health classification.
- Screenshots containing unreviewed messages from other people.

### Artifact formats

- Static PNG for universal sharing.
- Vertical carousel for depth.
- Short MP4 or animated WebP for era transitions.
- Static HTML bundle for an interactive story.
- A versioned .scroll package containing only approved derived data, assets, a manifest, and checksums.

The hosted web artifact should be optional. A user can share only an image and still participate in the loop.

## Growth Loops

### Loop 1: Identity artifact

User imports -> receives specific reflection -> shares -> peers compare themselves -> peers import.

This is the first launch loop because it requires only one archive and public-post data.

### Loop 2: Reciprocal unlock

User discovers a meaningful person -> sends a minimal invitation -> second person imports -> both unlock a Shared Scroll -> either or both share.

This can create a higher viral coefficient because value increases with another participant. It must follow, not precede, trust.

### Loop 3: Seasonal ritual

The product offers recurring prompts:

- Account anniversary.
- "On this day."
- End-of-year public voice recap.
- Ten-year retrospective.
- A major platform or community anniversary.

Unlike a single annual Wrapped, the archive supports many eras and anniversaries. The experience should not require fresh API access.

### Loop 4: Creator remix

Writers, designers, and communities publish open insight recipes and visual themes. Their followers install Tweet-Scrolls to use the same lens.

Examples:

- "My learning-in-public years."
- "My open-source origin story."
- "My fandom era."
- "My conference timeline."
- "Posts that became projects."

### Loop 5: Exit event

Platform controversy, policy changes, suspensions, or migration waves create periodic demand for an Exit Kit. Clear guides and durable exports turn those moments into adoption.

### Loop 6: Open-source contribution

An archive schema breaks -> a contributor fixes an adapter -> users of that archive variant arrive -> the contributor is credited -> more maintainers join.

### Loop 7: Community exhibit

A group creates an opt-in collection -> members contribute archives or selected posts -> the exhibit becomes more complete -> missing members are invited.

This is powerful but governance-heavy. It belongs in a later timeline.

## Funnel and Friction Model

The viral loop can be expressed as:

Viral archive activations =

artifact viewers
x artifact-to-CTA rate
x archive-request rate
x return-after-archive-ready rate
x install success
x import success
x meaningful-reveal rate

The weakest factors are likely:

- Archive request and delayed return.
- Desktop install trust.
- Large-file import reliability.
- Willingness to share.

This leads to several product requirements:

- Every artifact must explain the process in one sentence.
- The landing page should offer the official X archive-request instructions before asking for an install.
- The app should be useful in demo mode while the user waits.
- Installers must be signed and easy to verify.
- The ZIP should be accepted unopened.
- An interrupted import must resume.
- The first reveal must not wait for semantic indexing or AI.
- The share flow must feel safer than taking a screenshot.

## Candidate Approaches

### Conventional approach: Archive viewer and analytics dashboard

What it would include:

- Drag-and-drop import.
- Search.
- Tweet-style rendering.
- Charts for posting activity, likes, replies, and top words.
- CSV and PDF exports.

Why it is sensible:

- The value is easy to explain.
- Much of the current Rust logic can feed it.
- Search solves a real recurring problem.
- It is safer and easier than social features.

Why it is insufficient:

- Existing products already cover parts of this space.
- Analytics are rarely emotionally resonant enough to travel.
- It optimizes for the archive owner but contains no strong acquisition loop.
- It risks becoming a polished local utility with a modest ceiling.

Conclusion:

Build the viewer and index as infrastructure, but do not position them as the product.

### Non-obvious approach 1: A personal museum

Blend the archive problem with museums and documentary storytelling. The archive is a collection; the app is a curator; a Scroll is an exhibit.

Useful concepts from museums:

- Exhibits need a thesis rather than a data dump.
- Objects need labels and provenance.
- Visitors need a path through the material.
- Owners should be able to curate, omit, and reinterpret.
- Multiple exhibits can be built from the same collection.

This approach creates emotional value and shareable outcomes while keeping the archive private.

### Non-obvious approach 2: A two-person memory ceremony

Blend the archive with photo-album rituals, friendship anniversaries, and collaborative games. The interesting unit is not "my stats" but "our shared chapter."

Useful concepts:

- Each participant contributes.
- Nothing is unlocked publicly without mutual action.
- Comparing two histories creates specific invitations.
- The experience can live in group chats and reunions, not only public feeds.

This creates the highest organic network potential but requires exceptional consent design.

### Non-obvious approach 3: An open memory protocol

Blend the archive with Git, static-site generators, and durable web-archive formats. Tweet-Scrolls becomes a normalized local history format and a plugin ecosystem.

Useful concepts:

- Stable schemas.
- Content-addressed artifacts.
- Reproducible transforms.
- Portable exports.
- Community-maintained adapters.

This creates infrastructure adoption and long-term resilience, but protocols are not intrinsically viral to consumers.

### Best hybrid

Use the personal museum as the consumer wedge, the local memory protocol as the foundation, and the two-person ceremony as the networked second act.

In shorthand:

> Museum first. Library underneath. Shared ritual next. Public commons later.

## Timeline A: The Personal Internet Museum

### Opening move

Build a desktop application around one exceptional reveal: "Your Twitter Eras." The app imports the original ZIP locally, produces a guided story, and exports safe cards and videos.

### First six weeks

- Separate the parser into a reusable Rust core.
- Build direct ZIP validation and streaming ingest.
- Define a normalized event schema and a local SQLite database.
- Create a synthetic archive fixture with multiple years, threads, media references, and DMs.
- Prototype three deterministic stories:
  - First post versus now.
  - Posting and topic eras.
  - Forgotten public threads.
- Manually test outputs with 10 to 20 volunteer archives, processed on their machines.
- Test whether people save or share the artifacts without being asked.

The team experience:

The work feels like editorial product design as much as engineering. The hard problem is choosing what not to say. Every impressive-looking claim must survive a "show me the receipts" review.

### Months two to four

- Ship a signed private alpha for macOS, Windows, and Linux as feasible.
- Add progressive import, resumability, and a resource estimator.
- Build a story editor so users can rename eras and remove items.
- Add a share inspector and public-post-only default.
- Produce PNG, carousel, and static HTML outputs.
- Launch a simple web gallery using synthetic or explicitly donated artifacts.
- Publish a transparent threat model and network-access policy.

Likely user reaction:

Users enjoy seeing first posts and forgotten threads. Generic top-word cards underperform. User-edited era names are more shareable than automatically generated labels because they feel authored rather than assigned.

### Months five to eight

- Launch around a cultural moment, not merely a version number.
- Seed with a small group of writers, builders, researchers, and long-time Twitter users who process archives privately.
- Make the call to action "Request your archive" rather than "Sign up."
- Add creator export templates and "On this day."
- Let community designers contribute themes without access to raw archives.

Likely system reaction:

The launch gets spikes whenever a recognizable person posts a specific, surprising artifact. The archive delay produces a second, quieter wave days later. Support load clusters around ZIP variants, unsigned installers, disk space, and import duration.

### Months nine to twelve

- Improve search, media browsing, collections, and exports.
- Introduce an annual or account-anniversary ritual.
- Publish a versioned .scroll artifact format.
- Add localization for the request and import journey.
- Measure which story types actually cause downstream imports.

### Long-term shape

Tweet-Scrolls becomes the open-source personal museum for one's public internet history. It is known for emotionally intelligent curation and visible privacy.

### Virality

High initial shareability, medium network effect, strong seasonal spikes.

### Retention

Medium until search, resurfacing, and creator reuse mature; high for creators and memory keepers after they do.

### Likelihood

Medium-high, because the current code and Rust stack align with the foundation and the first viral loop does not need another user.

### Stress points

- Producing insights that are specific without using unreliable AI.
- Supporting archive format drift.
- Cross-platform signing and packaging.
- Avoiding a repetitive Wrapped aesthetic.
- Handling the archive-request delay.

### Inflection points

- If fewer than roughly one in five activated testers voluntarily save a card, the story is not strong enough.
- If users save cards but do not share, the product may still be a good private museum but not a viral artifact engine.
- If artifacts get clicks but imports do not complete, focus shifts from content to trust, request recovery, and installation.
- If search drives frequent returns, Timeline B should be accelerated as the retention layer.

### Regret profile

Low. ZIP ingest, local indexing, provenance, and artifact safety remain useful in every other timeline.

## Timeline B: The Searchable Second Self

### Opening move

Position Tweet-Scrolls as the fastest private way to search, ask questions of, annotate, and reuse one's Twitter history.

### First six weeks

- Build direct ZIP import and a normalized SQLite plus FTS5 index.
- Support filters for date, author, reply status, hashtag, domain, media, and conversation.
- Reconstruct stable local URLs for posts and threads.
- Add collections and Markdown export.
- Test the top 25 retrieval queries with real archive owners.

The team experience:

The work is less theatrical and more reliability-driven. Search quality, indexing speed, schema coverage, and result context dominate. Users report concrete failures, which makes iteration crisp.

### Months two to four

- Add semantic search as an optional local index.
- Add "Ask my archive" with retrieval citations.
- Keep deterministic search as the default and fallback.
- Support optional local models and explicit external-provider plugins.
- Add creator workflows: newsletter draft, thread collection, quote cards, and static pages.
- Import likes and bookmarks where the archive contains them.

Likely user reaction:

Power users love it. General users understand the utility only after they have a retrieval need. Screenshots of surprisingly old ideas and predictions create some organic sharing, but there is no reliable mass ritual.

### Months five to eight

- Add saved searches, tags, notes, and "On this day."
- Integrate exports to Obsidian, plain Markdown folders, and static-site generators.
- Add an agent-readable local API with explicit permissions.
- Build provenance and citation into every AI answer.

### Months nine to twelve

- Become a broader local social-memory workspace.
- Consider other archive sources only after Twitter/X quality is excellent.
- Publish the normalized schema and parser SDK.

### Long-term shape

Tweet-Scrolls becomes a respected personal knowledge tool and local data substrate. It is frequently recommended in creator, research, and data-ownership communities.

### Virality

Medium-low. Sharing is incidental to useful retrieval.

### Retention

High for creators, researchers, and heavy users. Medium for everyone else.

### Likelihood

High for a useful niche product; medium for mass adoption.

### Stress points

- Semantic search can feel magical but produce untrustworthy summaries.
- Local model support increases binary size, hardware variance, and support complexity.
- Broad knowledge-tool positioning competes with mature note and AI products.
- DMs in semantic indexes create a large privacy surface.

### Inflection points

- If over half of activated users run several searches in the first week, search is the core habit.
- If users export collections repeatedly, creator workflows deserve priority.
- If AI answers are used but source links are ignored, the product needs stronger evidence UX.
- If most users only run the guided reveal, Timeline A remains the acquisition center.

### Regret profile

Low. The index and retrieval layer strengthens every timeline, but leading with it may sacrifice a cultural launch window.

## Timeline C: The Relationship Constellation

### Opening move

Build a private social graph across public conversations and DMs, then let users invite specific people to unlock mutual, consent-based Shared Scrolls.

### First six weeks

- Do not launch a public feature.
- Correct identity matching and distinguish user IDs, handles, renamed accounts, and deleted accounts.
- Integrate tweet events into the relationship timeline instead of analyzing DMs alone.
- Define a consent-safe derived schema that can represent counts, dates, selected posts, and user-approved labels without transmitting raw archives.
- Run qualitative interviews about which relationship memories feel delightful, creepy, painful, or unsafe.

The team experience:

This path is emotionally intense. Product reviews require privacy, abuse, and interpersonal-safety thinking. Edge cases are human rather than merely technical: breakups, bereavement, harassment, power imbalance, blocked accounts, and mismatched memories.

### Months two to four

- Ship a private "People" view with no public ranking.
- Use neutral descriptions such as "frequent public conversations in 2016" rather than "closest person."
- Let users hide people, periods, and categories.
- Prototype a one-way invitation containing no message content.
- Make the invited person import independently and review the proposed overlap.

Likely user reaction:

Some users find it profoundly moving. Others find any automatic social ranking invasive. The feature earns strong private word-of-mouth but can produce severe backlash if marketed carelessly.

### Months five to eight

- Launch private two-person replays.
- Add item-level mutual approval.
- Allow a public Shared Scroll only after both parties approve the final artifact.
- Make consent revocation clear for hosted versions.
- Add an abuse-report and safety-review process before broad promotion.

Likely system reaction:

Invitations convert better than generic referrals because they are personal. However, each failed identity match or surprising disclosure costs disproportionate trust.

### Months nine to twelve

- Add group Scrolls for small, explicitly formed groups.
- Explore friendship anniversaries and shared community eras.
- Keep DMs out of public templates unless every quoted participant approves.

### Long-term shape

Tweet-Scrolls becomes a private social-memory network without a central social graph. The network emerges through mutual local archives and consented derivative artifacts.

### Virality

Potentially very high because every experience can invite another user. The variance is also very high.

### Retention

High for users with meaningful pair and group histories.

### Likelihood

Medium-low as a first product, medium-high as a second act after the private vault earns trust.

### Stress points

- Correct identity resolution.
- Unequal and incomplete archives.
- Consent withdrawal.
- Other people's words inside a user's archive.
- Harassment or unwanted invitations.
- Emotional harm from rankings or resurfacing.
- Secure exchange without building a centralized private-data service.

### Inflection points

- If users repeatedly request pair experiences during Timeline A, demand is real.
- If most users refuse to invite another person even after enjoying a private preview, the loop is too sensitive.
- If users prefer private pair replays over public sharing, optimize for meaningful adoption rather than public virality.
- One privacy incident should pause expansion and trigger a full safety review.

### Regret profile

High if attempted before trust foundations. Low if built later on explicit consent primitives.

## Timeline D: The Open Community Memory Commons

### Opening move

Invite users to donate selected public portions of their archives to community collections and build searchable exhibits around people, topics, and eras.

### First six weeks

- Study existing Community Archive tools and policies.
- Define contribution, deletion, correction, attribution, and licensing rules.
- Distinguish a public post being visible on X from a person consenting to a new persistent corpus.
- Build selection tools that let a user contribute subsets, not an all-or-nothing archive.
- Start with a bounded community that has explicit stewardship.

The team experience:

Engineering is no longer the main constraint. Governance, moderation, identity disputes, data removal, copyright, community norms, and hosting become daily work.

### Months two to four

- Launch one curated exhibit, such as the history of an open-source community or conference.
- Publish a provenance manifest.
- Give contributors a preview of exactly what becomes public.
- Support removal without requiring a GitHub issue that exposes personal context.
- Provide a forkable static export so the collection is not trapped in one service.

Likely user reaction:

Researchers and community historians are enthusiastic. Ordinary users may not understand why they should donate an archive. Public figures may attract most attention, concentrating the corpus and limiting broad participation.

### Months five to eight

- Add topic timelines, semantic search, and public thread reconstruction.
- Let third parties build tools on a documented API or dataset.
- Create community steward roles and visible governance.
- Add rate limits, abuse controls, and correction workflows.

### Months nine to twelve

- Federate or export collections rather than centralizing everything.
- Support institutionally managed archives.
- Pursue partnerships with libraries, researchers, and open-web communities if they align with consent rules.

### Long-term shape

Tweet-Scrolls becomes an open memory infrastructure project. Its public collections can outlive the original platform and support cultural research.

### Virality

High within particular communities, lower as a broad consumer loop.

### Retention

High for researchers and stewards; low for one-time contributors.

### Likelihood

Medium for impact, low for mass consumer adoption without a preceding personal product.

### Stress points

- Consent and removal.
- Copyright and quoted material.
- Public/private account transitions.
- Deleted posts.
- Harassment and decontextualization.
- Hosting and moderation even without monetization.
- Competition and overlap with Community Archive.

### Inflection points

- If a bounded pilot cannot define a trusted removal process, do not scale.
- If third-party builders use the schema more than end users use the exhibit, spin the protocol into a first-class project.
- If public corpus growth depends on celebrity archives, the product has not found a general participation loop.

### Regret profile

High if centralized early. Medium-low if built as opt-in static collections on top of mature private tooling.

## Timeline E: The Sovereign Exit Kit

### Opening move

Become the most reliable open-source tool for turning an X archive into a durable, searchable, portable personal website or knowledge folder.

### First six weeks

- Accept original ZIPs.
- Validate archive integrity and supported files.
- Stream data into a normalized local store.
- Export static HTML, Markdown, JSON, and SQLite.
- Preserve media references and copy available local media safely.
- Document exactly what the archive does and does not contain.

The team experience:

The work is concrete, testable, and infrastructure-heavy. There is less pressure to invent narratives. Compatibility matrices, fixtures, packaging, and support documentation dominate.

### Months two to four

- Add migration packs for static blogs, Obsidian, and common social-export formats.
- Add a local tweet-style viewer.
- Add archive-health reports and checksums.
- Package for macOS, Windows, Linux, Homebrew, Winget, and other relevant channels.
- Publish the parser as a library and CLI.

Likely user reaction:

Users trust and recommend it during deactivation or platform-crisis moments. Many use it once and keep the outputs rather than the app.

### Months five to eight

- Add scheduled export verification.
- Add adapters for old and new archive variants.
- Integrate with broader web-archiving formats where useful.
- Encourage downstream apps to use the normalized schema.

### Months nine to twelve

- Become the dependable infrastructure under other projects, including the Personal Museum.
- Expand only to adjacent archives when maintainers and fixtures exist.

### Long-term shape

Tweet-Scrolls becomes a digital-sovereignty utility and parser standard. It may be widely installed but culturally quiet.

### Virality

Low normally, high during platform events.

### Retention

Low in the app, high in the exported artifacts.

### Likelihood

High for useful adoption, low for sustained cultural virality.

### Stress points

- Format drift and edge cases.
- Cross-platform packaging.
- Static-site security.
- Users expecting data that X did not include.
- Low ongoing engagement after successful export.

### Inflection points

- If event-driven traffic repeatedly produces durable installs and contributors, this can be a strong standalone mission.
- If most users immediately ask "what can I learn from it?", accelerate Timeline A or B.
- If downstream tools adopt the schema, invest in protocol governance.

### Regret profile

Very low. Nearly all work is foundational, although spending too long here can postpone the viral experiment indefinitely.

## Cross-Timeline Analysis

| Path | Consumer upside | Viral ceiling | Retention | Trust risk | Technical fit today | Reversibility | Main dependency |
| --- | --- | --- | --- | --- | --- | --- | --- |
| A. Personal Internet Museum | Broad emotional appeal and strong launch object | High | Medium, then high with search | Medium | Strong | High | Artifact quality and import completion |
| B. Searchable Second Self | Deep recurring utility for heavy users | Medium-low | High | Medium | Strong | High | Search quality and archive coverage |
| C. Relationship Constellation | Unique and deeply personal network value | Very high | High | Very high | Weak until timeline and anonymization gaps close | Medium | Consent, identity matching, and safety |
| D. Community Memory Commons | Cultural preservation and ecosystem value | High in niches | Medium | Very high | Medium | Low if centralized | Governance and stewardship |
| E. Sovereign Exit Kit | Clear ownership and migration value | Event-driven | Low in app | Low-medium | Very strong | Very high | Reliability and packaging |

### Which path is strongest if things go normally?

Timeline A. It has the clearest route to broad attention and can reuse the same local index needed by B and E.

### Which path is safest if things go badly?

Timeline E. Even if nobody shares a Scroll, direct ZIP import, validation, search, and durable export remain valuable.

### Which path has the greatest upside?

Timeline C. Mutual memory can create a real person-to-person network loop. It should be treated as an earned expansion, not a launch gimmick.

### Which path carries the most irreversible risk?

Timeline D if it centralizes public archives, followed by C if it exposes relationship data without mature consent controls.

### What is the recommended sequence?

1. Build the trustworthy import and local index foundation from E.
2. Launch the identity-artifact wedge from A.
3. Deepen retention with search and reuse from B.
4. Test mutual-consent Shared Scrolls from C.
5. Enable bounded, stewarded public collections from D only after governance exists.

This is not five products in a roadmap. It is one set of foundations with evidence-based branch points.

## Recommended Architecture

### Product shape

~~~mermaid
flowchart TB
    ZIP["Original X archive ZIP"] --> CORE["Rust archive core"]
    CORE --> VAULT["Private local vault: SQLite, FTS, media index"]
    VAULT --> PRIVATE["Private desktop museum, search, people, calendar"]
    VAULT --> ENGINE["Deterministic insight engine"]
    ENGINE --> STUDIO["Share Studio and artifact inspector"]
    STUDIO --> ARTIFACT["Sanitized PNG, video, HTML, or .scroll"]
    ARTIFACT --> WEB["Optional web viewer and mobile-friendly landing"]
    WEB --> NEXT["Next user requests archive and installs desktop app"]
    NEXT --> ZIP
~~~

### 1. Rust archive core

Refactor the current processing logic into a library with explicit stages:

- Archive discovery and validation.
- ZIP entry reader with zip-bomb and path-traversal protections.
- Versioned format adapters.
- Streaming parsers.
- Normalization into stable event types.
- Identity resolution with confidence and provenance.
- Deterministic metrics.
- Export and artifact APIs.

Suggested conceptual interfaces:

- ArchiveImporter: recognizes and streams one archive variant.
- EventNormalizer: maps source records into the stable model.
- InsightRecipe: computes a typed, evidence-backed result.
- ArtifactRenderer: renders allowlisted data into a share format.
- Exporter: writes a durable external format.

These names are conceptual, not a final code design.

### 2. Normalized local model

The current code writes final files directly. The evolved product needs an intermediate model that can be queried repeatedly.

Core entities:

- Archive and archive version.
- Account and identity aliases.
- Public post.
- Reply relationship.
- Direct message and conversation.
- Media item.
- Like or bookmark where present.
- Follow or list snapshot where present.
- Link and domain.
- Derived era.
- Collection.
- Insight with evidence references and algorithm version.
- Share artifact with an explicit field manifest.

Every normalized record should retain:

- Source file and source record identity.
- Parsing version.
- Original timestamp and timezone information where available.
- Privacy class.
- Whether the content is public, private, unknown, deleted, or user-supplied.

### 3. Local storage

Recommended:

- SQLite as the durable metadata store.
- FTS5 for deterministic full-text search.
- Filesystem storage for media already present in the archive.
- A content-addressed cache for generated thumbnails and artifacts.
- Database migrations and resumable import checkpoints.

The original ZIP should remain untouched. Users can choose whether the app references it in place or copies it into an app-managed vault.

### 4. Desktop shell

Tauri is the most natural default because:

- The processing core is already Rust.
- It supports a Rust backend with a web-rendered interface.
- It can access local files through explicit scoped permissions.
- It can produce platform-specific installers.
- The same UI code can power a browser-based artifact viewer.

This is a recommendation, not a mandate. A native toolkit may offer tighter platform behavior but would slow cross-platform design iteration.

Desktop responsibilities:

- File picker and drag-and-drop.
- Import progress and resource estimates.
- Local database lifecycle.
- Private exploration.
- Offline mode.
- Share Studio.
- Updates and migration.

### 5. Web companion

The public web application should initially do only five things:

- Explain the value and privacy model.
- Show synthetic or explicitly published Scrolls.
- Give official archive-request instructions.
- Link to signed desktop releases.
- Render optional sanitized .scroll artifacts.

It should not accept raw archive uploads in the initial product.

A later browser-only mode can process smaller, public-post-only archives client-side. It should not become the reliability baseline for huge archives. ReplayWeb.page demonstrates that local browser processing can preserve privacy, while its own documentation notes that a standalone desktop app is preferable for repeated direct filesystem access.

### 6. Insight engine

Use a layered approach:

Layer 1: deterministic facts

- Counts.
- Dates.
- Frequencies.
- Thread lengths.
- Vocabulary shifts.
- Link domains.
- Public interaction counts.

Layer 2: statistical segmentation

- Change-point detection for eras.
- Topic clusters.
- Recurring motifs.
- Activity phases.

Layer 3: generated interpretation

- Human-readable era labels.
- Summaries.
- Suggested captions.

Layer 3 is editable, labeled, optional, and always linked to evidence. It can run through a local model or an explicitly selected provider. The product must remain valuable without it.

### 7. Share compiler

Treat artifact generation like compiling from a restricted language:

- Input fields are allowlisted.
- Privacy class is checked.
- Text snippets are scanned for handles, phone numbers, emails, URLs with tokens, and location-like strings.
- DMs are denied unless a separate explicit capability is enabled.
- Media metadata is stripped where appropriate.
- The user previews the exact final output.
- The manifest records every source item and transform.
- Hosted artifacts have deletion and expiration controls.

This boundary is more important than generic anonymization. It minimizes what leaves the vault rather than trying to make arbitrary private data safe after the fact.

### 8. Plugin model

Four plugin categories can create community leverage:

- Import adapters.
- Insight recipes.
- Visual themes.
- Exporters.

Plugins should declare:

- Which data classes they can read.
- Whether they require network access.
- Which files they write.
- Whether their outputs are private or shareable.
- Their deterministic test fixtures.

The safest initial plugin system is build-time or curated. Loading arbitrary third-party executable code into a DM vault is not an acceptable early shortcut.

### 9. Performance requirements

Targets to validate:

- The app accepts the unopened ZIP.
- Import is streaming and bounded in memory.
- A crash or restart resumes from a checkpoint.
- Cheap summary facts appear before full indexing.
- Search remains responsive while deeper derivations run.
- Users can cancel and delete partial imports.
- Archive-size and disk-space estimates appear before extraction.
- The app never executes HTML or JavaScript contained in the archive.

### 10. Security model

Threats to include:

- Malformed ZIPs.
- Zip bombs.
- Path traversal during extraction.
- Executable archive HTML or JavaScript.
- Untrusted media files.
- Plugin exfiltration.
- External-model prompt and data leakage.
- Accidental inclusion of DMs in public artifacts.
- Hosted artifact enumeration.
- Local database access by other processes.
- Backups and cloud-synced folders containing private vaults.

Minimum trust deliverables before broad launch:

- Correct the anonymization claims immediately.
- Publish a threat model.
- Add a SECURITY.md and private disclosure route.
- Sign release artifacts.
- Publish checksums.
- Add reproducible-build guidance.
- Expose network activity and external providers in the UI.
- Default to no telemetry.
- Add one-click vault deletion and documented data locations.
- Test share artifacts for PII leakage.

## First Public Product

The first public product should feel complete but narrow. It is not a dashboard with every future capability.

### The promised journey

1. Discover a Scroll on mobile.
2. Understand in one sentence that the raw archive stayed private.
3. Request the official X archive.
4. Continue on desktop and install a signed app.
5. Drag the unopened ZIP into the app.
6. See the first meaningful fact while the rest imports.
7. Experience "Your Twitter Eras."
8. Inspect the posts behind each chapter.
9. Search the private archive.
10. Edit and export one safe public Scroll.
11. Delete the local vault or keep it for resurfacing and search.

### The mobile-to-desktop bridge

Discovery will usually happen on a phone even though processing belongs on desktop. The landing experience should therefore have two equal calls to action:

- "Request my X archive."
- "Continue on desktop."

Possible handoff mechanisms:

- A short stable URL to the correct desktop release page.
- The phone share sheet so users can send the link to themselves.
- A QR code shown by the desktop app for returning to the same public artifact.
- An optional calendar reminder or browser notification while waiting for the archive.
- Optional email only if the user explicitly wants it; no account should be required.

Do not encourage the user to download a multi-gigabyte archive to mobile merely to preserve funnel continuity.

### Version-one feature set

Must have:

- macOS, Windows, and Linux distribution strategy, with the first signed platform chosen by tester availability.
- Original ZIP import.
- Archive validation and format report.
- Resumable, bounded-memory indexing.
- Local SQLite and full-text search.
- Public-post timeline and thread view.
- A five-to-seven-card Twitter Eras reveal.
- Evidence links for every insight.
- User editing and omission.
- Public-post-only Share Studio.
- PNG and static HTML export.
- Clear local data location and one-click deletion.
- Offline mode.
- Synthetic demo archive.

Should have:

- Media gallery from files already present in the archive.
- Carousel and short video export.
- Markdown and JSON export.
- "On this day."
- Link-domain and vocabulary views.
- Import diagnostic bundle that contains no private content.

Not in version one:

- Raw archive upload to a server.
- AI analysis of DMs.
- Public DM cards.
- Mutual relationship invitations.
- Community corpus donation.
- Arbitrary third-party plugins.
- Live X API integration.
- Mobile archive import.
- Multiple social-network imports.
- Cloud synchronization.
- A generic social feed inside Tweet-Scrolls.

### First reveal requirements

The reveal should be:

- Specific enough that two users rarely receive identical language.
- Mostly deterministic.
- Progressive.
- Editable.
- Source-backed.
- Visually coherent.
- Safe to abandon without losing the indexed archive.

Potential sequence:

1. "You arrived in [year]. This was your first surviving post."
2. "Your archive spans [number] years and [number] public posts."
3. "Your public voice changed most around these periods."
4. "These ideas followed you across multiple eras."
5. "This forgotten thread still sounds like you."
6. "These were your public conversation neighborhoods."
7. "Name your eras."

The final step matters. User-authored labels transform generated analysis into self-expression.

## Roadmap as an Option Tree

This roadmap is deliberately organized around evidence gates rather than a fixed list of promised features.

### Phase 0: Trustworthy foundation

Goal:

Prove that valid archives can be imported safely, progressively, and repeatedly.

Deliverables:

- Direct ZIP import.
- Streaming and resumable processing.
- Normalized event model.
- SQLite and FTS.
- Archive fixtures.
- Correct privacy claims.
- Security and open-source project files.
- Green test suite.

Exit evidence:

- Representative large archives import without out-of-memory failure.
- Import can resume after interruption.
- Archive variants produce understandable diagnostics.
- No network is required.

### Phase 1: Artifact desirability

Goal:

Prove that the archive can create an artifact users genuinely want to keep or share.

Deliverables:

- Twitter Eras reveal.
- Three artifact families.
- Evidence browser.
- Story editing.
- Share inspector.

Exit evidence:

- A meaningful share of activated testers voluntarily saves an artifact.
- Some users share without a reward or prompt from the team.
- Artifact viewers can accurately explain what the product does.
- No artifact leaks unapproved fields in red-team testing.

Branch:

- Strong saving and sharing: continue Timeline A.
- Strong saving but weak sharing: position as private museum and invest in B.
- Weak saving: revisit the insight and narrative, not the visual polish.

### Phase 2: Habit and reuse

Goal:

Prove that the app remains useful after the reveal.

Deliverables:

- Fast search.
- Collections.
- Media and thread views.
- On-this-day resurfacing.
- Creator exports.

Exit evidence:

- Users return for search or resurfacing.
- Creators export more than once.
- Search results lead to saved items or external work.

Branch:

- Search dominates: accelerate Timeline B.
- Resurfacing dominates: deepen the museum and calendar.
- Exports dominate: deepen the Exit Kit and creator workflows.

### Phase 3: Mutual memory

Goal:

Test whether another person's participation creates enough value to justify the safety complexity.

Deliverables:

- Private People view.
- Neutral relationship descriptors.
- Invitation without raw content.
- Mutual preview and approval.
- Private Shared Scroll.

Exit evidence:

- Users request the feature organically.
- Invite recipients understand what is shared before importing.
- Both participants complete approval.
- Safety interviews show delight substantially outweighs discomfort.

Branch:

- Strong and safe: continue Timeline C.
- Valuable but too sensitive publicly: keep it private.
- Weak conversion or high discomfort: stop without harming the museum.

### Phase 4: Stewarded commons

Goal:

Test one bounded, opt-in community exhibit.

Prerequisites:

- Mature public artifact model.
- Removal and correction workflows.
- Named stewards.
- Legal and policy review appropriate to the chosen community.
- No dependency on dumping full archives into a central database.

Branch:

- Healthy stewardship and downstream use: grow carefully.
- Moderation or consent overwhelms value: export the tooling and stop operating a central service.

## Metrics

### North-star metric

Viral Archive Activations:

> The number of successful archive imports completed within a defined attribution window after the person viewed a shared Scroll.

This measures the whole loop, not merely shares or clicks.

### Companion product-health metric

Meaningful Archive Activations:

> Successful imports where the user views at least three evidence-backed archive items, saves a collection, runs a search, or exports an artifact.

This prevents the project from optimizing only for social impressions.

### Funnel metrics

- Scroll views.
- Scroll view to archive-request instruction.
- Archive-request instruction completion, measured in opt-in research because X does not expose it to the app.
- Desktop continuation rate.
- Installer download and successful launch.
- Valid ZIP selection.
- Import completion by archive size, age, format, and operating system.
- Time to first meaningful reveal.
- Reveal completion.
- Evidence-item opens.
- Artifact saves.
- Artifact shares.
- Downstream activated imports per artifact.
- Viral chain depth.

### Retention metrics

- Search sessions in days 7, 30, and 90.
- On-this-day opens.
- Collections created.
- Repeat exports.
- Vaults retained versus deleted.
- Private pair experiences revisited.

### Trust and safety guardrails

- Confirmed private-data exposure incidents.
- Share drafts blocked by safety checks.
- User-reported inaccurate insights.
- Hosted artifact deletions and reasons, when voluntarily supplied.
- DM sharing attempts and abandonment.
- Import crash and out-of-memory rate.
- Vault deletion success.
- Network calls by feature.
- Security response time.

The target for confirmed unapproved private-data exposure is zero. Growth work pauses after a credible incident until the cause and affected boundary are understood.

### Open-source adoption metrics

- Release downloads by platform.
- Successful package-manager installs.
- Contributors who land a change.
- Supported archive variants with fixtures.
- Third-party insight recipes and exporters.
- Downstream projects using the core schema.
- Median issue response and fix time.
- Percentage of releases with signed artifacts and published checksums.

### Measurement without violating the product

Default-off telemetry creates blind spots, and those blind spots are acceptable. Options include:

- Referral parameters on hosted Scroll links, which measure public viewing but reveal nothing about the archive.
- Opt-in anonymous events whose exact payload can be previewed locally.
- Explicit research cohorts.
- In-app local metrics the user can export voluntarily.
- Public release-download and package-manager data.

Do not weaken the privacy promise to get a cleaner funnel chart.

## Experiments That Collapse Uncertainty

All numeric thresholds below are provisional learning gates, not market forecasts.

### Experiment 1: Artifact desirability before full UI

Question:

Will people voluntarily keep or share an archive-derived identity artifact?

Method:

- Recruit 15 to 25 long-time Twitter users across creators, ordinary users, researchers, and private accounts.
- Process only public-post data on their machines.
- Create three artifact families:
  - Twitter Eras.
  - First post versus now.
  - Twelve years in twelve posts.
- Let each participant edit and remove content.
- Do not ask them to post. Observe what they choose to save or share.
- Interview both sharers and non-sharers.

Positive signal:

- At least 40 percent save one artifact.
- At least 20 percent share or ask when sharing will be available.
- Users describe the artifact with identity language rather than analytics language.

Failure interpretation:

If they admire the visuals but do not save, the content is generic. If they save but do not share, there may still be a private product. If they worry about privacy before discussing the reveal, trust presentation is failing.

### Experiment 2: The delayed archive funnel

Question:

Can interest survive the request-and-wait process?

Method:

- Publish a mobile-first artifact landing page.
- Offer official request instructions and a desktop continuation.
- Test a demo app, a reminder, and no reminder.
- Follow an opt-in cohort from first view to ZIP import.

Positive signal:

- A meaningful fraction of people who request an archive return when it is ready.
- Installing before the archive arrives improves return without causing confusion.

Failure interpretation:

If the wait destroys intent, viral content alone cannot fix the funnel. The product may need recurring campaign reminders, a stronger demo, or an Exit Kit trigger where users already intend to request data.

### Experiment 3: Trust comprehension

Question:

Do nontechnical users believe and understand the local-only model?

Method:

Compare:

- "Your data stays local."
- A visual data-flow diagram.
- An offline-mode demonstration.
- A share manifest preview.

Ask users to explain what leaves the device.

Positive signal:

Most can accurately say that the archive stays local and only the approved artifact leaves.

Failure interpretation:

Open-source and privacy copy are not enough. The architecture needs visible proof.

### Experiment 4: Search retention

Question:

Does the private library create value after the reveal?

Method:

- Give alpha users full-text search, collections, and one creator export.
- Ask them to note real retrieval needs over four weeks.
- Measure locally or through an opt-in study.

Positive signal:

Users return for unprompted searches and use results outside the app.

Failure interpretation:

If search is rarely used, retention may need resurfacing, creator-specific workflows, or an archive-health utility rather than a general library.

### Experiment 5: Shared Scroll consent prototype

Question:

Is a mutual archive experience delightful enough to justify its safety burden?

Method:

- Use paper or local prototypes with pairs who already trust each other.
- Show multiple data-boundary options.
- Include uncomfortable examples on purpose.
- Let either participant veto any category or item.
- Do not transmit raw archives.

Positive signal:

Both people understand the boundary, want the final private replay, and some want to invite another pair.

Failure interpretation:

If users want the insight but not the invitation, keep relationship views private. If one participant feels pressured, redesign or stop.

### Experiment 6: Open-source contribution ladder

Question:

Can an external contributor add one archive adapter, insight recipe, or theme in a weekend?

Method:

- Give five developers a synthetic fixture and contribution guide.
- Observe setup, test, preview, and submission.
- Record every maintainer intervention.

Positive signal:

Most can produce a tested local change without seeing private data.

Failure interpretation:

The extension boundary is too coupled or the documentation is not executable.

## Launch Strategy

### Campaign concept

The campaign should be about the user, not the repository.

Strong directions:

- "The internet remembers you."
- "Your Twitter life, privately replayed."
- "What were your eras?"
- "You posted through history. Now see the history."

Avoid:

- "AI analytics for X."
- "Upload your data."
- "The ultimate Twitter dashboard."
- Technical-first messaging about Rust, CSV, or local LLMs.

### Launch sequence

#### 1. Artifact lab

Before a broad release, publish a gallery made from synthetic archives and artifacts donated with explicit permission. Show variety rather than one visual template.

#### 2. Trusted seed cohort

Invite people with:

- Long histories.
- Distinct eras.
- Public writing or building.
- Audiences likely to care about archives, open source, or internet culture.
- Willingness to explain the local processing model.

Process on their machines. Do not ask for their ZIP.

#### 3. Synchronized but not exclusive launch

A concentrated launch window helps people see several different Scrolls in the same week. It creates comparison and a social prompt without making the product annual-only.

#### 4. Delayed return campaign

Expect users' archives to arrive later. Publish a second wave:

- "Your archive ready? Drop it in."
- Import clinics.
- Troubleshooting guides.
- Community examples.

#### 5. Evergreen triggers

After the launch:

- Account anniversaries.
- New artifact recipes.
- Creator templates.
- Open-source releases.
- Platform migration events.
- Community-history projects.

### Distribution surfaces

- X, because the content and social graph originate there.
- Bluesky and Mastodon, because archive ownership resonates with users who migrated.
- GitHub, Hacker News, and open-source communities for trust and contributors.
- Creator newsletters and blogs for reuse workflows.
- Data-hoarding, digital-preservation, quantified-self, and personal-knowledge communities.
- Group chats for Shared Scrolls and nostalgic artifacts.

### Built-in attribution

Every artifact should include:

- A small Tweet-Scrolls mark, subordinate to the user's story.
- "Generated locally from my X archive."
- An artifact or recipe name.
- A human-readable link or QR code.

Do not turn artifacts into billboards. If the mark harms willingness to share, the loop loses.

## Open-Source Adoption Strategy

### License

For maximum reuse and adoption, the default recommendation is dual MIT or Apache-2.0 for the Rust core and documented schemas. The desktop app can use the same license for simplicity.

Tradeoff:

- A permissive license maximizes downstream use and reduces institutional friction.
- MPL-2.0 is a reasonable alternative if keeping modifications to core files open matters more than unrestricted adoption.
- AGPL creates stronger reciprocity for hosted services but may reduce reuse and complicate the stated objective.

The current absence of a license should be fixed before inviting contributors.

### Project trust kit

Add:

- LICENSE.
- CONTRIBUTING.md.
- SECURITY.md.
- CODE_OF_CONDUCT.md.
- Privacy and threat-model documentation.
- Supported archive-format matrix.
- Synthetic fixtures.
- Architecture decision records.
- Release signing and checksum instructions.
- A clear no-real-archives-in-issues policy.

### Contributor ladders

Provide small, legible ways to participate:

- Add an archive fixture.
- Fix one format adapter.
- Add one deterministic insight recipe.
- Add one export format.
- Translate the request and onboarding flow.
- Create one visual theme.
- Improve one safety detector.
- Reproduce one large-archive performance issue with synthetic data.

### Plugin and recipe gallery

The gallery can itself become a growth surface. Each recipe page should show:

- What question it answers.
- Which archive data it reads.
- Whether it can use DMs.
- Whether its output is safe to share by default.
- Example output using synthetic data.
- Algorithm and evidence rules.
- Author and version.

### Governance

Start with maintainer-led decisions and published principles. Add a lightweight request-for-comment process for:

- Normalized schema changes.
- New private-data capabilities.
- Artifact format changes.
- Network access.
- Public corpus policies.

Governance should become more formal only when contributors and downstream users create a real need.

### Packaging is product work

Maximum adoption requires more than source builds:

- Signed macOS app.
- Signed Windows installer.
- Linux AppImage, Flatpak, or appropriate packages.
- Homebrew and Winget where maintainable.
- Clear update and rollback behavior.
- Checksums and release notes.
- A one-command developer setup.

## Risks and Pre-Mortem

### Failure 1: The reveal is attractive but shallow

Symptom:

Users say "cool" and never request an archive.

Cause:

The cards summarize activity rather than revealing identity.

Mitigation:

Use specific receipts, user-authored era names, and genuinely different story recipes. Test content before polishing animation.

### Failure 2: The archive wait kills momentum

Symptom:

Artifact clicks are high but imports are negligible.

Cause:

The referral loop spans multiple devices and days.

Mitigation:

Treat request, install-before-ready, reminder, and return as a first-class lifecycle. Use evergreen follow-up moments.

### Failure 3: Trust collapses

Symptom:

A screenshot or issue shows that IDs were not anonymized or a DM appeared in a share output.

Cause:

Marketing claims exceeded implementation, or sharing reused private UI state.

Mitigation:

Correct current claims, build a separate share compiler, red-team artifacts, publish a threat model, and pause growth after incidents.

### Failure 4: Large archives fail

Symptom:

The users with the most interesting histories have the worst import experience.

Cause:

Whole-file reads, nonresumable processing, disk surprises, or format variance.

Mitigation:

Streaming import, checkpoints, resource estimation, archive adapters, and synthetic scale tests must precede visual expansion.

### Failure 5: AI damages credibility

Symptom:

Era labels are wrong, summaries invent motives, or users cannot trace a claim.

Cause:

Generated interpretation was treated as truth.

Mitigation:

Deterministic facts first, evidence links, editable labels, confidence, and explicit model boundaries.

### Failure 6: DMs become growth bait

Symptom:

The most clicked cards expose other people's private words or rank intimate relationships.

Cause:

Short-term engagement pressure overrode consent.

Mitigation:

Keep DMs private by default, require a separate capability, and use mutual approval for relationship artifacts.

### Failure 7: It becomes a clone

Symptom:

Users compare it to a local Twitter viewer, analytics dashboard, or Wrapped template and see no reason to switch.

Cause:

The project copied expected features without a distinct product thesis.

Mitigation:

Lead with the private personal museum, user curation, receipts, and the share boundary. Search and analytics support the story.

### Failure 8: Open source does not create contributors

Symptom:

The repository gets stars but all format fixes depend on one maintainer.

Cause:

No fixtures, extension boundaries, contribution ladder, or release discipline.

Mitigation:

Make one adapter or recipe a weekend-sized, testable contribution and credit contributors in the product.

### Failure 9: Public archives overwhelm the project

Symptom:

Removal requests, identity disputes, and moderation consume development capacity.

Cause:

Timeline D launched before governance.

Mitigation:

Keep public contributions bounded, opt-in, stewarded, exportable, and late.

### Failure 10: The team builds foundations forever

Symptom:

The parser becomes excellent but no one has seen an artifact.

Cause:

Low-regret infrastructure work feels safer than testing taste.

Mitigation:

Run the artifact desirability experiment in parallel with foundation work. A manually generated but privacy-safe prototype can test demand before a complete UI.

## Known Unknowns and How to Make Them Known

| Unknown | Why it matters | Cheapest credible test |
| --- | --- | --- |
| Will a person request an archive for this? | Determines the top of the loop | Mobile landing plus opt-in funnel cohort |
| Which artifact creates self-expression rather than novelty? | Determines share rate | Three-family concierge artifact test |
| Can typical users understand local-only processing? | Determines trust and install | Comprehension test with visible data flow |
| How variable are current archive formats? | Determines support cost | Collect schemas and diagnostics, never private content |
| What is time to first value across archive sizes? | Determines activation | Synthetic and volunteer benchmark matrix |
| Is search a recurring habit? | Determines retention | Four-week alpha diary study |
| Are users comfortable with a private People view? | Determines whether C is viable | Safety interviews and paper prototypes |
| Can mutual consent work without a central private-data service? | Determines architecture for C | Derived-data exchange prototype |
| Will contributors build recipes and adapters? | Determines ecosystem leverage | Five-developer contribution test |
| What happens when X changes the archive? | Determines resilience | Versioned fixtures and adapter contract |
| Are hosted artifacts necessary? | Determines operational burden | Compare image-only shares with interactive links |
| Does the name Tweet-Scrolls explain enough? | Determines word of mouth | Landing-page message and recall test |

## Evidence and Verification

### Evidence from the current repository

- The main CLI expects an archive folder and tweets.js rather than the original ZIP: [src/cli.rs](../src/cli.rs).
- Tweet processing reads the entire file into a string, parses it, filters retweets, groups reply threads, and writes text and CSV: [src/processing/tweets.rs](../src/processing/tweets.rs).
- DM processing reads the entire file, creates DM threads, and writes timeline analysis: [src/processing/direct_messages.rs](../src/processing/direct_messages.rs).
- Relationship timeline construction currently adds DM events and explicitly skips tweet events: [src/relationship/analyzer.rs](../src/relationship/analyzer.rs).
- The anonymization module does not currently implement Blake3 hashing despite comments and README claims: [src/relationship/anonymization.rs](../src/relationship/anonymization.rs).
- The package is a Rust 2021 application with no ZIP, SQLite, search, or GUI dependency today: [Cargo.toml](../Cargo.toml).
- A local test run on 2026-07-10 produced 81 passing tests and one failure in the custom-output-directory file-splitter test.

### Evidence from the ecosystem

- X officially documents that archives can contain posts, DMs, media, follows, followers, address-book information, Lists, ad data, and more, and provides the archive-request flow: [How to access and download your X data](https://help.x.com/en/managing-your-account/accessing-your-x-data).
- Shreyas Doshi's public product-decision questions motivate the expected-versus-unexpected solution framing used here: [Product decision questions](https://x.com/shreyas/status/1290703709270228993).
- Spotify describes Wrapped as a personalized reflection and explicitly optimizes individual data stories as social share cards. Its 2023 Wrapped reached a reported 227 million monthly active users: [A decade of Spotify Wrapped](https://newsroom.spotify.com/2024-12-04/10-years-spotify-wrapped/) and [Wrapped for artists](https://newsroom.spotify.com/2024-12-04/from-data-to-strategy-how-artists-can-make-the-most-of-their-2024-wrapped/).
- Spotify's later Shared Party model shows that personal retrospectives can evolve into small-group comparison and sharing, while also requiring disclosure that others can see the data: [Wrapped Party](https://newsroom.spotify.com/2025-12-03/wrapped-party-how-to/).
- Community Archive already operates an open Twitter database with trends, semantic search, public archives, and community-built tools. This confirms both demand and competitive overlap for Timeline D: [Community Archive](https://www.community-archive.org/).
- Birdclaw already positions itself as a local-first Twitter workspace with SQLite, FTS5, archive import, DMs, likes, bookmarks, media, a local web UI, and a CLI. A generic local workspace is therefore not an empty category: [birdclaw](https://github.com/steipete/birdclaw).
- Tweet Archivist already offers uploaded-archive analytics, visualization, preservation, and public sharing. A chart dashboard alone is not differentiated: [Tweet Archivist](https://www.tweetarchivist.com/).
- ReplayWeb.page demonstrates that a browser can load a local archive without uploading it, while its documentation recommends the standalone app for repeated direct filesystem access: [Loading archived items](https://replayweb.page/docs/user-guide/loading/) and [Offline use](https://replayweb.page/docs/user-guide/offline-use/).
- Tauri supports a Rust backend, web frontend, scoped filesystem access, and platform-specific installers, aligning with the current Rust core and desktop requirement: [What is Tauri?](https://v2.tauri.app/start/), [File system](https://v2.tauri.app/plugin/file-system/), and [Distribution](https://v2.tauri.app/distribute/).
- ArchiveBox shows that an open-source archival product can combine local control, durable formats, a CLI, web UI, and desktop distribution: [ArchiveBox](https://github.com/ArchiveBox/ArchiveBox).

### Facts versus inference

Sourced facts:

- What X says is in the archive.
- What the current repository parses and how it reads files.
- Which adjacent products publicly claim search, archive import, analytics, local storage, or public collections.
- Which capabilities Tauri and ReplayWeb.page document.
- Spotify's public description of Wrapped sharing and usage.

Reasoned inferences:

- Identity artifacts are more viral than generic analytics for this product.
- A desktop vault plus web invitation is stronger than a pure desktop or pure web product.
- Search is the likely retention layer.
- Mutual Shared Scrolls have the highest eventual network potential.
- Public community archives should be delayed.

Speculation to test:

- The exact artifact users will share.
- Conversion through the delayed archive funnel.
- The willingness to install a desktop app from an open-source project.
- The safety and desirability of pair experiences.
- The eventual size of the audience.

### Verification questions

Question:

Does X still provide an archive with enough history to power this?

Answer:

Yes in principle. X's current help documentation lists complete post history plus DMs, media, social lists, and other data. Exact completeness and file variants still need fixture testing.

Question:

Is local browser processing impossible?

Answer:

No. ReplayWeb.page proves that substantial local archive processing can happen in a browser. The recommendation for desktop is based on persistent filesystem access, huge files, resumability, packaging, and the existing Rust engine, not a claim of browser impossibility.

Question:

Is a local searchable Twitter archive novel by itself?

Answer:

No. Birdclaw and other tools occupy that territory. Search remains essential, but it is not sufficient positioning.

Question:

Is Wrapped-style sharing alone a durable product?

Answer:

Not necessarily. Spotify's success demonstrates the power of personalized shareable reflection, but Spotify also has continuous usage data and an existing daily habit. Tweet-Scrolls needs search, resurfacing, and reuse to create retention after import.

Question:

Can the current app safely market DM anonymization?

Answer:

No. The current implementation does not substantiate its Blake3 claims. This should be corrected before user growth work.

## Decision Filter

Use these questions for every major feature:

1. Does it make the archive owner feel recognition, agency, or durable utility?
2. Is it useful without uploading the raw archive?
3. Can the claim show its evidence?
4. Does the feature improve acquisition, activation, retention, or trust? Which one?
5. Is the output about the user, or is it advertising the tool?
6. Could another person's private content appear?
7. Can the user preview, omit, edit, and delete?
8. Does it strengthen a reusable foundation across timelines?
9. What evidence would tell us to stop?

### Recommended next decision

Do not choose between all five timelines in the abstract. Run one experiment that collapses the central uncertainty:

> Generate three privacy-safe artifact families from 15 to 25 volunteer public-post archives, on the owners' devices, and observe voluntary saving and sharing.

In parallel, build only the no-regret foundation needed for a real product:

- Original ZIP import.
- Streaming, resumable processing.
- Correct privacy claims.
- Normalized local storage.
- Full-text search.
- A separate share compiler.

If artifacts create pull, launch Timeline A. If private search creates stronger pull, lead with Timeline B while retaining artifacts as acquisition. If neither creates pull, Timeline E remains a useful and honest open-source product. Do not attempt C or D until the project has earned trust.

## Final Synthesis

Tweet-Scrolls has the raw ingredients for something much larger than an archive converter, but maximum virality will not come from adding more analysis. It will come from converting private history into a small number of public objects that feel specific, true, beautiful, and safe.

The strategic shape is:

> Desktop vault, private museum, public Scrolls, mutual memories, open protocol.

The first launch should tell one compelling story: "Here are the eras of your life on Twitter, and here are the posts that prove it." The underlying product should quietly solve the harder job: "Your history is searchable, portable, and yours."

That combination can travel without betraying the archive that makes it possible.
