// Convert a CodeGraph into Cytoscape elements.

import type { CodeGraph, GraphNode } from './types';

const LANG_COLORS: Record<string, string> = {
  python: '#3776ab',
  typescript: '#3178c6',
  javascript: '#f7df1e',
  rust: '#dea584'
};

const KIND_GLYPHS: Record<GraphNode['kind'], string> = {
  file: 'rectangle',
  module: 'round-rectangle',
  function: 'ellipse',
  type: 'diamond'
};

export interface CyElements {
  nodes: cytoscape.ElementDefinition[];
  edges: cytoscape.ElementDefinition[];
}

export function toCytoscape(graph: CodeGraph): CyElements {
  const fileLanguage = new Map<number, string>();
  for (const n of graph.nodes) {
    if (n.kind === 'file') fileLanguage.set(n.id, n.language);
  }
  const moduleFile = new Map<number, number>();
  for (const n of graph.nodes) {
    if (n.kind === 'module') moduleFile.set(n.id, n.file);
  }

  const nodes: cytoscape.ElementDefinition[] = graph.nodes.map((n) => {
    const langForNode = (() => {
      if (n.kind === 'file') return n.language;
      if (n.kind === 'module') {
        const f = moduleFile.get(n.id);
        return f !== undefined ? fileLanguage.get(f) : undefined;
      }
      // Functions / types inherit the enclosing file's colour via their parent.
      return undefined;
    })();
    const label =
      n.kind === 'file'
        ? n.path
        : n.kind === 'module'
        ? n.name
        : n.kind === 'function'
        ? n.name
        : n.name;
    return {
      group: 'nodes',
      data: {
        id: String(n.id),
        kind: n.kind,
        label,
        color: langForNode ? LANG_COLORS[langForNode] ?? '#888' : '#3a4150',
        shape: KIND_GLYPHS[n.kind]
      }
    };
  });

  const edges: cytoscape.ElementDefinition[] = graph.edges.map((e, i) => ({
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

// Cytoscape's style-language uses string mappers like `data(color)` and
// `data(shape)` that the @types declarations can't fully validate; we cast
// at the boundary so the rest of the file reads naturally.
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
      width: 28,
      height: 28
    }
  },
  {
    selector: 'node[kind = "file"]',
    style: {
      width: 60,
      height: 28,
      'font-size': '10px'
    }
  },
  {
    selector: 'node[kind = "type"]',
    style: {
      width: 32,
      height: 32
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
      opacity: 0.6
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
  }
] as unknown) as cytoscape.StylesheetStyle[];
