import { useEffect, useMemo, useRef } from 'react';
import CytoscapeComponent from 'react-cytoscapejs';
import type cytoscape from 'cytoscape';
import { useViewer } from './store';
import { cytoscapeStyle, toCytoscape } from './cy';
import type { GraphNode } from './types';

export interface ViewerProps {
  graphId: string;
}

export function Viewer({ graphId }: ViewerProps) {
  const graph = useViewer((s) => s.graph);
  const loading = useViewer((s) => s.loading);
  const error = useViewer((s) => s.error);
  const selected = useViewer((s) => s.selected);
  const select = useViewer((s) => s.select);
  const load = useViewer((s) => s.load);
  const cyRef = useRef<cytoscape.Core | null>(null);

  useEffect(() => {
    void load(graphId);
  }, [graphId, load]);

  const elements = useMemo(() => {
    if (!graph) return [];
    const { nodes, edges } = toCytoscape(graph);
    return [...nodes, ...edges];
  }, [graph]);

  if (error) {
    return (
      <div className="loom-empty">
        <div>
          <strong>Could not load graph.</strong>
        </div>
        <div>{error}</div>
        <div>
          <code>graph_id</code>: {graphId}
        </div>
      </div>
    );
  }

  if (loading || !graph) {
    return <div className="loom-empty">Loading graph {graphId}...</div>;
  }

  if (graph.nodes.length === 0) {
    return (
      <div className="loom-empty">
        <div>
          <strong>Graph contains no nodes.</strong>
        </div>
        <div>
          The repo at <code>{graph.repo_root}</code> had no parseable files.
        </div>
      </div>
    );
  }

  return (
    <div className="loom-shell">
      <header className="loom-topbar">
        <h1>Loom Lens</h1>
        <div className="modes">
          <span className="mode active">Graph</span>
          <span className="mode" title="lands at M2">
            Effects
          </span>
          <span className="mode" title="lands at M3">
            Hashes
          </span>
        </div>
        <div className="meta">
          {graph.summary.files} files · {graph.summary.functions} functions ·{' '}
          {graph.summary.modules} modules ·{' '}
          {Object.entries(graph.summary.languages)
            .map(([k, v]) => `${k} ${v}`)
            .join(', ')}{' '}
          · @{graph.graph_id}
        </div>
      </header>

      <div className="loom-canvas">
        <CytoscapeComponent
          elements={elements}
          stylesheet={cytoscapeStyle}
          layout={{ name: 'cose', animate: false, fit: true }}
          style={{ width: '100%', height: '100%' }}
          cy={(cy) => {
            cyRef.current = cy;
            cy.removeListener('select unselect');
            cy.on('select', 'node', (evt) => {
              const id = Number(evt.target.id());
              if (!Number.isNaN(id)) select(id);
            });
            cy.on('unselect', 'node', () => select(null));
          }}
        />
      </div>

      <footer className="loom-status">
        <span>parsed in {graph.summary.elapsed_ms} ms</span>
        <span>generated {graph.generated_at}</span>
        {selected !== null && graph.nodes[selected] ? (
          <span>
            selected:&nbsp;
            <code>{describe(graph.nodes[selected])}</code>
          </span>
        ) : (
          <span>click a node to inspect</span>
        )}
      </footer>
    </div>
  );
}

function describe(n: GraphNode | undefined): string {
  if (!n) return '';
  if (n.kind === 'file') return `file ${n.path}`;
  if (n.kind === 'module') return `module ${n.name}`;
  if (n.kind === 'function') return `fn ${n.qualified_name}`;
  return `type ${n.qualified_name}`;
}
