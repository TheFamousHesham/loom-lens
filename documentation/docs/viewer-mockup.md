# Viewer Mockup

Text-based wireframes for the three modes of the Loom Lens browser viewer. The actual UI is rendered with React + Tailwind + Cytoscape.js. These wireframes lock the layout and the interaction model; visual polish (typography, exact colors, spacing) is delegated to M2 implementation.

> **Status:** Drafted at Checkpoint 1. Implementation lands at M1 (Graph), M2 (Effects), M3 (Hashes). When implementation diverges from this document, fix the implementation, not the spec.

---

## Common chrome (all three modes)

```
┌──────────────────────────────────────────────────────────────────────────────────────────┐
│  Loom Lens │ ◉ Graph │ ○ Effects │ ○ Hashes │   [search nodes…    ] │  loom-lens@a1b2c3d │
├────────────────────────┬───────────────────────────────────────────────┬─────────────────┤
│                        │                                               │                 │
│  Filters               │              Graph canvas                     │  Details        │
│  (mode-specific)       │              (Cytoscape.js)                   │  (selected      │
│                        │                                               │   node)         │
│                        │                                               │                 │
│                        │                                               │                 │
│                        │                                               │                 │
│                        │                                               │                 │
│                        │                                               │                 │
├────────────────────────┴───────────────────────────────────────────────┴─────────────────┤
│  142 files · 1,830 functions · 67 modules · python (87) typescript (55)        ⊕ ⊖  ⌂   │
└──────────────────────────────────────────────────────────────────────────────────────────┘
```

- **Top bar.** Loom Lens wordmark on the left; three mode tabs (only one active); a node search field; the repo identity (`<name>@<short_commit>` or `<path>@WORKING_TREE`).
- **Left rail.** Mode-specific filter/control panel. Collapsible to a strip of icons.
- **Center canvas.** The graph itself. Pan/zoom via mouse-drag/scroll; selection by click; multi-select by shift-click; lasso by shift-drag on empty space.
- **Right rail.** Details for the selected node — appears empty when no selection, populated when one or more nodes selected.
- **Bottom bar.** Repo summary plus zoom controls (`⊕`/`⊖`) and a `⌂` "fit graph" reset.
- **Keyboard.** `g` / `e` / `h` cycle modes; `/` focuses search; `f` fits graph; `escape` clears selection; arrow keys pan.

---

## Mode 1 — Graph

The structural view. Nodes are files/modules/functions/types; edges are contains/calls/imports/references. Colors encode *language*, not effects. The default mode after `analyze_repo`.

```
┌──────────────────────────────────────────────────────────────────────────────────────────┐
│  Loom Lens │ ◉ Graph │ ○ Effects │ ○ Hashes │   [search…           ] │  loom-lens@a1b2c3d │
├────────────────────────┬───────────────────────────────────────────────┬─────────────────┤
│ NODES                  │                                               │ src/api/users.py│
│ ☑ Files          (142) │                            ●━━━━━━━━━━━●      │ ::fetch_user    │
│ ☑ Modules        (67)  │       ┌──── src/api/users.py ────┐            │                 │
│ ☑ Functions   (1,830)  │       │  ◆ User      ╲           │            │ Function        │
│ ☑ Types         (203)  │       │  ● fetch_user╲╲          │            │ Lines 42–58    │
│                        │       │  ● make_user  ╲╲    ┌──── src/main.py │ python          │
│ EDGES                  │       │  ● delete_user╲╲╲   │ ● run           │                 │
│ ☑ Contains    (3,420)  │       └────────────────╲╲╲──┘                 │ Signature:      │
│ ☑ Calls       (5,210)  │             │           ╲╲╲                   │ def fetch_user( │
│ ☑ Imports       (610)  │             │            ╲╲╲ ┌── tests/ ──┐   │   user_id: int  │
│ ☑ References    (440)  │             ▼              ╲╲│ ● test_use │   │ ) -> User       │
│                        │       ┌── src/db/orm.py ──┐  │            │   │                 │
│ LANGUAGES              │       │ ◆ Connection      │  └────────────┘   │ Calls into:     │
│ ☑ python  ▆▆▆▆░░░ 87   │       │ ● connect         │                   │   requests.get  │
│ ☑ typescript ▆▆░ 55    │       │ ● query           │                   │   raise_if_     │
│ ☑ rust         ░    0  │       └───────────────────┘                   │     invalid     │
│                        │                                               │                 │
│ PATH FILTER            │                                               │ Called by:      │
│ [src/api/**         ]  │                                               │   User.refresh  │
│                        │                                               │   /handlers/me  │
├────────────────────────┴───────────────────────────────────────────────┴─────────────────┤
│  142 files · 1,830 functions · 67 modules · python (87) typescript (55)        ⊕ ⊖  ⌂   │
└──────────────────────────────────────────────────────────────────────────────────────────┘
```

- **Node glyphs.**
  - `■` File (rectangle)
  - `▣` Module (rounded rectangle)
  - `●` Function (filled circle)
  - `◆` Type / class (diamond)
- **Node color.** By language. Python = `#3776ab`, TypeScript = `#3178c6`, Rust = `#dea584`. Saturated when selected/hovered, faded when filtered out.
- **Edge style.**
  - Solid arrow: `Calls`
  - Dotted arrow: `Imports`
  - Dashed arrow: `References` (function → type)
  - Containment is rendered as visual nesting (a function inside its file box); no explicit edge.
- **Layout.** Cytoscape's `cose-bilkent` for first render; user can drag nodes; double-clicking a file collapses/expands its containment.
- **Right rail when a function is selected.** File path, line range, language, full signature, "Calls into" (callees) summary, "Called by" (callers) summary, hash tag (truncated, copy-icon to copy full hash), inline source preview (lazy-loaded on expand).

---

## Mode 2 — Effects

Same graph topology; coloring switches from "by language" to "by effect." Multiple effects on a single node are rendered as a pie split. Confidence is encoded as fill style.

```
┌──────────────────────────────────────────────────────────────────────────────────────────┐
│  Loom Lens │ ○ Graph │ ◉ Effects │ ○ Hashes │   [search…           ] │  loom-lens@a1b2c3d │
├────────────────────────┬───────────────────────────────────────────────┬─────────────────┤
│ EFFECTS                │                                               │ src/api/users.py│
│ ☑ Net  ●        (145)  │                              ╔═══╗            │ ::fetch_user    │
│ ☑ IO   ●         (88)  │        ┌── src/api/users.py ─║Net║─┐          │                 │
│ ☑ Mut  ●        (220)  │        │  ◆ User       ●    ╚═══╝  │          │ Effects:        │
│ ☑ Throw ●       (410)  │        │  ●═════ fetch_user        │          │  ● Net    def.  │
│ ☑ Async ●        (95)  │        │  ●▓▓▓▓▓ make_user         │          │     fetch() at  │
│ ☑ Random ●       (12)  │        │  ●░░░░░ delete_user       │          │     line 45     │
│ ☑ Time  ●        (31)  │        └───────────────────────────┘          │                 │
│ ☑ Foreign ●      (18)  │              │                                │  ● Throw  prob. │
│ ☐ External (off)       │              ▼                                │     raises      │
│                        │       ┌── src/db/orm.py ──────┐                │     HTTPError   │
│ CONFIDENCE             │       │ ●═════ connect (Net)  │                │                 │
│ ▣ Definite  (solid)    │       │ ●▓▓▓▓▓ query (Net,IO) │                │ Hash:           │
│ ▒ Probable (striped)   │       │ ●░░░░░ ping (Net,?)   │                │   blake3:7f3a…  │
│ ░ Possible (outline)   │       └───────────────────────┘                │                 │
│                        │                                               │ Called by:      │
│ ▼ Hide pure functions  │       Pure functions hidden (240 nodes)       │   User.refresh  │
│   ☑ on                 │                                               │                 │
│                        │                                               │                 │
├────────────────────────┴───────────────────────────────────────────────┴─────────────────┤
│  Showing 1,590 of 1,830 functions · effect Net (145) IO (88) Mut (220) …       ⊕ ⊖  ⌂   │
└──────────────────────────────────────────────────────────────────────────────────────────┘
```

- **Effect color palette** (mnemonic, contrast-checked WCAG AA):
  - Net = `#1f77b4` (blue)
  - IO = `#ff7f0e` (orange)
  - Mut = `#d62728` (red)
  - Throw = `#9467bd` (purple)
  - Async = `#17becf` (teal)
  - Random = `#bcbd22` (lime)
  - Time = `#7f7f7f` (gray)
  - Foreign = `#8c564b` (brown)
  - Pure = `#e0e0e0` (light gray, low salience)
- **Confidence fills**, per ADR 0002 §"UI mapping":
  - `definite` → solid color (`█`)
  - `probable` → diagonal stripes (`▒`)
  - `possible` → outline only (`░`)
- **Multiple effects.** A node with both Net and Throw is rendered as a half-and-half disc; three effects → tri-split; >3 → "primary effect" by descending priority (Net, IO, Mut, Throw, Async, Random, Time, Foreign), with a small "+N" badge indicating overflow.
- **Filters.** Each effect has an enable/disable checkbox and a count. Disabling an effect dims (not hides) nodes that have it as their *only* effect. "Hide pure functions" is a one-click utility for quickly seeing where the impure code lives.
- **Right rail effect listing.** For the selected node, each effect appears with its confidence and the literal evidence (`fetch() at line 45`) so the user can verify the inference at a glance.

---

## Mode 3 — Hashes

Same graph topology; coloring switches to identity classes. The default sub-view is "duplicates" — equal-hash functions tinted the same color. A second sub-view, "diff," activates after a `compare_hashes` call: changed/added/removed/renamed function classes.

```
┌──────────────────────────────────────────────────────────────────────────────────────────┐
│  Loom Lens │ ○ Graph │ ○ Effects │ ◉ Hashes │   [search…           ] │  loom-lens@a1b2c3d │
├────────────────────────┬───────────────────────────────────────────────┬─────────────────┤
│ SUBVIEW                │                                               │ Equivalence     │
│ ◉ Duplicates (current) │      Duplicate classes shown as ringed        │ class:          │
│ ○ Diff                 │      groups; class colors encode identity.    │ blake3:7f3a…    │
│                        │                                               │                 │
│ DUPLICATES             │      ┌── class A ──┐    ┌── class B ──┐       │ 4 members:      │
│ Class A    4 members ● │      │ ● parse_pkt │    │ ● to_dict   │       │ • src/api/serial│
│ Class B    3 members ● │      │ ● parsePkt  │    │ ● serialize │       │   ize.py::      │
│ Class C    2 members ● │      │ ● _parse    │    │ ● dump_json │       │   parse_packet  │
│ Class D    2 members ● │      │ ● handle_   │    └─────────────┘       │ • src/legacy.   │
│ … 19 more              │      │   payload   │                          │   parsePkt      │
│                        │      └─────────────┘                          │ • src/util/p.py │
│ MIN MEMBERS            │                                               │   ::_parse      │
│ [≥ 2  ▾]               │      Singletons (1 member, no duplicate)      │ • src/h.py::    │
│                        │      are rendered at low opacity.             │   handle_payload│
│ ▼ Show near-duplicates │                                               │                 │
│   ☐ off                │                                               │                 │
│                        │                                               │                 │
│ DIFF (idle)            │                                               │ Verdict:        │
│  before [HEAD~1   ]    │                                               │  4× exact dup   │
│  after  [HEAD     ]    │                                               │  candidate for  │
│  [Compare]             │                                               │  consolidation  │
│                        │                                               │                 │
├────────────────────────┴───────────────────────────────────────────────┴─────────────────┤
│  23 duplicate classes · 56 affected functions · 4-member class is largest      ⊕ ⊖  ⌂   │
└──────────────────────────────────────────────────────────────────────────────────────────┘
```

```
┌────────────── Hashes mode · Diff sub-view (after Compare clicked) ──────────────┐
│   before HEAD~1 (a1b2c3) ←→ after HEAD (f5e6d7)                                │
│                                                                                │
│   ╔══════ changed (3) ══════╗  ╔═══ added (1) ════╗  ╔═══ removed (2) ════╗   │
│   ║ ●●● semantically diff   ║  ║ ● new function   ║  ║ ●● vanished       ║   │
│   ╚═════════════════════════╝  ╚══════════════════╝  ╚═══════════════════╝   │
│                                                                                │
│   ╔════════ renamed (1) ════════╗                                              │
│   ║ ● same hash, new location   ║                                              │
│   ╚═════════════════════════════╝                                              │
│                                                                                │
│   text-only-change (whitespace, comments, formatting) does NOT appear here.    │
└────────────────────────────────────────────────────────────────────────────────┘
```

- **Duplicate-class colors.** A small palette (8 colors) cycled across classes; classes ranked by member count. Singletons render at opacity 0.25 to keep visual focus on the multi-member classes.
- **Near-duplicate toggle.** When on, dashed connectors link near-duplicate classes (low edit distance over canonical AST form). Defers to v0.2 if the algorithm is too slow at scale; v0.1 ships with exact-dup only.
- **Diff sub-view** activates after the user enters two refs and clicks Compare. Result: four colored buckets at the canvas edge, each containing the affected functions. Selecting a function in a bucket shows before/after source side-by-side in the right rail.
- **What this mode answers**, from ARCHITECTURE.md §10:
  - "Are there duplicates worth deduping?" — duplicates sub-view, ranked by class size.
  - "What semantically changed?" — diff sub-view; the textual noise of `git diff` is filtered out by construction.
  - "Did I rename without changing behavior?" — the `renamed (1)` bucket isolates exactly that case.

---

## Cross-mode interactions

- **Selection persists across modes.** Switching from Graph to Effects with a function selected keeps that function selected and pans the canvas to it.
- **The viewer URL encodes mode and filter state.** `…/r/<graph_id>?mode=effects&filter=effect:Net&node=fn_142` — sharable, bookmarkable, and the `query_graph` tool returns these URLs directly.
- **The right rail's "open in editor" link** uses an `editor://` URL handler if the user has one configured (e.g., Cursor, VSCode). Falls back to a "copy file:line" button.

---

## Notes for implementation

- The viewer is a single React app served from `crates/viewer/`. State is in a Zustand store; URL → state and state → URL are both handled by `react-router-dom` v6 with custom serialization for filter params.
- Cytoscape is wrapped in `react-cytoscapejs`. The graph data is fetched once per `graph_id` from `/api/graph/:graph_id`; mode changes only re-style nodes and edges, never re-fetch.
- The 10k-node performance budget is a M2 acceptance criterion; if Cytoscape's WebGL renderer can't hit it, the layout falls back to "show top N nodes by degree, with a `graph-too-large` banner inviting the user to filter."
- All three modes share the same canvas component. The differences are entirely in stylesheet and the active filter panel.
- Accessibility: every color encoding has a redundant pattern/glyph (so colorblind users aren't reliant on hue); keyboard navigation reaches every interactive control.
