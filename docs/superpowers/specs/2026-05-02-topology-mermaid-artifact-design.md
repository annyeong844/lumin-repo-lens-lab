# Topology Mermaid Artifact Design

Date: 2026-05-02
Status: approved for implementation

## Purpose

`topology.mermaid.md` is a generated, human-readable visual companion for
`topology.json`. It helps a reviewer see cross-submodule flow, runtime cycles,
and high-degree hub files without reading raw JSON first.

It is not an independent source of truth. Grounded structural claims must still
cite `topology.json` path/value evidence. The Mermaid artifact may be mentioned
as a navigation aid, but not as the citation authority for counts, absence
claims, or complete edge lists.

## Artifact Contract

The artifact is written only when `topology.json` is available. It is derived
entirely from `topology.json` and contains no repository re-scan logic.

The document has stable top-level sections:

1. `How To Read This`
2. `Cross-Submodule Edges`
3. `Runtime Cycles`
4. `Hub Files`
5. `Omitted Detail / Limits`
6. `Citation Contract`

Each section must render an explicit empty state when its source data is absent.
Large lists are capped, and the output reports both the shown count and source
count so omitted detail is visible.

## Data Flow

`audit-repo.mjs` loads `topology.json`, calls `renderTopologyMermaid(topology)`,
and writes `<output>/topology.mermaid.md` atomically with the same artifact
discipline used by other generated outputs.

`manifest.json` records:

- artifact path
- format
- source artifact
- intended use

`audit-summary.latest.md` lists the Mermaid file in the artifact map as a visual
aid only. It must continue to tell the model to cite `topology.json` for exact
topology claims.

## Rendering Rules

Cross-submodule edges are sorted by descending count, then source, then target.
They are capped at a deterministic edge limit.

Runtime cycles render from `topology.sccs[]`. Internal cycle edges are drawn only
when both endpoints are known members and the edge is not type-only. The renderer
must not emit dangling or `undefined` Mermaid node ids.

Hub files render from `topFanIn` and `topFanOut` when present. This section is
plain Markdown, not only a diagram, so reviewers can scan high-degree files even
when Mermaid preview is unavailable.

All Mermaid labels must be escaped for quoted labels. Output must be stable for
the same input.

## Error And Limit Handling

Missing or empty source arrays produce explicit notes, not silent omissions.
Truncated sections report `Showing N of M` and list the configured cap.

If no Mermaid graph can be drawn because there are no cross-submodule edges and
no runtime cycles, the artifact still renders the reading instructions, hub file
section, limits, and citation contract.

## Testing

Tests must cover:

- stable Markdown section contract
- cross-submodule graph rendering and label escaping
- deterministic sorting and cap reporting
- runtime SCC rendering without dangling node ids
- empty-state rendering
- hub file rendering from fan-in and fan-out data
- orchestrator production of `topology.mermaid.md`

The generated skill package must include the same renderer, docs, tests metadata,
and command-routing guidance after `npm run build:skill`.
