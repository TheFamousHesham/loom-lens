// Convert a CodeGraph into Cytoscape elements + the stylesheet that drives them.

import type { CodeGraph, GraphNode } from './types';

const LANG_COLORS: Record<string, string> = {
  python: '#3776ab',
  typescript: '#3178c6',
  javascript: '#f7df1e',
  rust: '#dea584'
};

// Kind → cytoscape `shape` value. We deliberately use distinct silhouettes so
// the four node types are separable at a glance even when colour is muted.
const KIND_GLYPHS: Record<GraphNode['kind'], string> = {
  file: 'rectangle',
  module: 'round-rectangle',
  function: 'ellipse',
  type: 'hexagon'
};

export interface CyElements {
  nodes: cytoscape.ElementDefinition[];
  edges: cytoscape.ElementDefinition[];
}

/// Returns true when a file's path looks test-y. Path glob (substring of
/// `/test/` or `/tests/`, or basename starting with `test_`); does NOT match
/// substrings inside other names — e.g. `attestation/users.py` is NOT a test.
function looksLikeTest(path: string): boolean {
  if (path.includes('/tests/') || path.includes('/test/')) return true;
  if (path.startsWith('tests/') || path.startsWith('test/')) return true;
  const base = path.split('/').pop() ?? '';
  return base.startsWith('test_');
}

export function toCytoscape(graph: CodeGraph): CyElements {
  const fileLanguage = new Map<number, string>();
  const fileIsTest = new Map<number, boolean>();
  for (const n of graph.nodes) {
    if (n.kind === 'file') {
      fileLanguage.set(n.id, n.language);
      fileIsTest.set(n.id, looksLikeTest(n.path));
    }
  }
  const moduleFile = new Map<number, number>();
  for (const n of graph.nodes) {
    if (n.kind === 'module') moduleFile.set(n.id, n.file);
  }

  // Compute per-node degree so we can size hubs more prominently and so the
  // viewer's label-fade rule can prefer high-degree nodes.
  const degree = new Map<number, number>();
  for (const e of graph.edges) {
    degree.set(e.from, (degree.get(e.from) ?? 0) + 1);
    degree.set(e.to, (degree.get(e.to) ?? 0) + 1);
  }

  const nodeIsTest = (id: number, file: number | undefined): boolean => {
    if (id !== undefined && fileIsTest.get(id)) return true;
    if (file !== undefined && fileIsTest.get(file)) return true;
    return false;
  };

  // child→parent map (Contains edges) so we can derive each function/type's
  // owning file for filter purposes.
  const containsByChild = new Map<number, number>();
  for (const e of graph.edges) {
    if (e.kind === 'contains') {
      containsByChild.set(e.to, e.from);
    }
  }

  const nodes: cytoscape.ElementDefinition[] = graph.nodes.map((n) => {
    const langForNode = (() => {
      if (n.kind === 'file') return n.language;
      if (n.kind === 'module') {
        const f = moduleFile.get(n.id);
        return f !== undefined ? fileLanguage.get(f) : undefined;
      }
      return undefined;
    })();
    const label = n.kind === 'file' ? n.path : n.name;

    let fileForNode: number | undefined;
    if (n.kind === 'file') fileForNode = n.id;
    else if (n.kind === 'module') fileForNode = n.file;
    else {
      const moduleId = containsByChild.get(n.id);
      if (moduleId !== undefined) {
        const m = graph.nodes.find((mn) => mn.id === moduleId);
        if (m && m.kind === 'module') fileForNode = m.file;
      }
    }

    return {
      group: 'nodes',
      data: {
        id: String(n.id),
        kind: n.kind,
        label,
        color: langForNode ? LANG_COLORS[langForNode] ?? '#8a929c' : '#3a4150',
        shape: KIND_GLYPHS[n.kind],
        degree: degree.get(n.id) ?? 0,
        isTest: nodeIsTest(n.id, fileForNode) ? 1 : 0,
        fileId: fileForNode !== undefined ? String(fileForNode) : ''
      }
    };
  });

  // Imports edges are already file→file (per the resolver in build.rs);
  // Calls edges are function→function. Drop Contains since it's redundant
  // with the implicit file/module/function hierarchy.
  const edges: cytoscape.ElementDefinition[] = graph.edges
    .filter((e) => e.kind !== 'contains')
    .map((e, i) => ({
      group: 'edges',
      data: {
        id: `e${i}`,
        source: String(e.from),
        target: String(e.to),
        kind: e.kind
      }
    }));

  return { nodes, edges };
}

// Cytoscape's style-language uses string mappers (`data(color)`, `data(shape)`,
// `data(degree)`) that the @types declarations don't fully validate; cast at
// the boundary so the rest of the file reads naturally.
export const cytoscapeStyle = ([
  {
    selector: 'node',
    style: {
      'background-color': 'data(color)',
      shape: 'data(shape)',
      label: 'data(label)',
      color: '#e8eaed',
      'font-size': '11px',
      'text-outline-color': '#0e1116',
      'text-outline-width': 2,
      'text-valign': 'center',
      'text-halign': 'center',
      // Hide labels until they'd be readable on screen. Cytoscape compares
      // the rendered pixel size against this threshold.
      'min-zoomed-font-size': 6,
      width: 'mapData(degree, 0, 20, 18, 56)',
      height: 'mapData(degree, 0, 20, 18, 56)'
    }
  },
  {
    selector: 'node[kind = "file"]',
    style: {
      width: 'mapData(degree, 0, 20, 60, 110)',
      height: 26,
      'font-size': '10px'
    }
  },
  {
    selector: 'node[kind = "type"]',
    style: {
      width: 'mapData(degree, 0, 12, 32, 60)',
      height: 'mapData(degree, 0, 12, 32, 60)'
    }
  },
  {
    selector: 'edge',
    style: {
      width: 1,
      'line-color': '#444a55',
      'target-arrow-color': '#444a55',
      'target-arrow-shape': 'triangle',
      'curve-style': 'bezier',
      opacity: 0.55
    }
  },
  {
    selector: 'edge[kind = "imports"]',
    style: {
      'line-style': 'dotted',
      'line-color': '#7c8290',
      'target-arrow-color': '#7c8290'
    }
  },
  {
    selector: 'edge[kind = "calls"]',
    style: {
      'line-color': '#61afef',
      'target-arrow-color': '#61afef',
      width: 1.4
    }
  },
  {
    selector: 'node:selected',
    style: {
      'border-width': 3,
      'border-color': '#e0a500'
    }
  },
  {
    // Hide-tests filter applies this class.
    selector: '.test-hidden',
    style: {
      display: 'none'
    }
  },
  {
    // Detail-level filter (modules always hidden; classes/functions hidden
    // unless their toggle is on).
    selector: '.hidden-by-detail',
    style: {
      display: 'none'
    }
  }
] as unknown) as cytoscape.StylesheetStyle[];
