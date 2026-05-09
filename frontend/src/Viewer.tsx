import { useEffect, useMemo, useRef, useState } from 'react';
import CytoscapeComponent from 'react-cytoscapejs';
import cytoscape from 'cytoscape';
import dagre from 'cytoscape-dagre';
import { useViewer } from './store';
import { cytoscapeStyle, toCytoscape } from './cy';
import type { GraphNode } from './types';

// Register the dagre layout extension exactly once at module load.
cytoscape.use(dagre);

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
  const [hideTests, setHideTests] = useState<boolean>(true);
  const [showClasses, setShowClasses] = useState<boolean>(false);
  const [showFunctions, setShowFunctions] = useState<boolean>(false);

  useEffect(() => {
    void load(graphId);
  }, [graphId, load]);

  const elements = useMemo(() => {
    if (!graph) return [];
    const { nodes, edges } = toCytoscape(graph);
    return [...nodes, ...edges];
  }, [graph]);

  // Whenever any toggle (or the data) changes, reapply visibility classes
  // and re-run the dagre layout against what's left.
  useEffect(() => {
    const cy = cyRef.current;
    if (!cy) return;
    cy.batch(() => {
      cy.elements().removeClass('test-hidden hidden-by-detail');

      // Detail level: modules are always hidden (1:1 with files; redundant);
      // classes hidden unless `showClasses`; functions hidden unless `showFunctions`.
      const hidden = cy.nodes().filter((n) => {
        const k = n.data('kind') as string;
        if (k === 'module') return true;
        if (k === 'type' && !showClasses) return true;
        if (k === 'function' && !showFunctions) return true;
        return false;
      });
      hidden.addClass('hidden-by-detail');
      cy.edges()
        .filter((e) => hidden.contains(e.source()) || hidden.contains(e.target()))
        .addClass('hidden-by-detail');

      if (hideTests) {
        const testNodes = cy.nodes().filter((n) => Number(n.data('isTest')) === 1);
        testNodes.addClass('test-hidden');
        cy.edges()
          .filter((e) => testNodes.contains(e.source()) || testNodes.contains(e.target()))
          .addClass('test-hidden');
      }
    });
    // Choose layout by density. Sparse graphs (default file-only view)
    // benefit from a force-directed cose layout that arranges nodes by
    // mutual attraction; dense graphs (with classes/functions visible) get
    // dagre's layered approach. Either way fit only the visible subset.
    const visibleCount = cy.elements(':visible').length;
    const layoutName = visibleCount > 200 ? 'dagre' : 'cose';
    const layoutOpts: cytoscape.LayoutOptions =
      layoutName === 'dagre'
        ? ({
            name: 'dagre',
            animate: false,
            rankDir: 'TB',
            spacingFactor: 0.9,
            nodeSep: 20,
            rankSep: 60,
            fit: false
          } as cytoscape.LayoutOptions)
        : ({
            name: 'cose',
            animate: false,
            randomize: true,
            componentSpacing: 80,
            nodeRepulsion: 6000,
            idealEdgeLength: 90,
            edgeElasticity: 100,
            gravity: 60,
            fit: false
          } as cytoscape.LayoutOptions);
    const layout = cy.layout(layoutOpts);
    layout.on('layoutstop', () => {
      const visible = cy.elements(':visible');
      cy.fit(visible, 40);
    });
    layout.run();
  }, [hideTests, showClasses, showFunctions, elements]);

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
        <label
          style={{
            fontSize: '0.85rem',
            color: '#8a929c',
            display: 'flex',
            alignItems: 'center',
            gap: '0.35rem',
            marginLeft: '0.6rem',
            cursor: 'pointer',
            userSelect: 'none'
          }}
          title="Hide files in /tests/, /test/, or with names starting with test_"
        >
          <input
            type="checkbox"
            checked={hideTests}
            onChange={(e) => setHideTests(e.target.checked)}
          />
          hide tests
        </label>
        <label
          style={{
            fontSize: '0.85rem',
            color: '#8a929c',
            display: 'flex',
            alignItems: 'center',
            gap: '0.35rem',
            marginLeft: '0.4rem',
            cursor: 'pointer',
            userSelect: 'none'
          }}
          title="Reveal individual function nodes (off by default — file/module/class is the entry view)"
        >
          <input
            type="checkbox"
            checked={showClasses}
            onChange={(e) => setShowClasses(e.target.checked)}
          />
          show classes
        </label>
        <label
          style={{
            fontSize: '0.85rem',
            color: '#8a929c',
            display: 'flex',
            alignItems: 'center',
            gap: '0.35rem',
            marginLeft: '0.4rem',
            cursor: 'pointer',
            userSelect: 'none'
          }}
          title="Reveal individual function nodes (off by default — file/class is the entry view)"
        >
          <input
            type="checkbox"
            checked={showFunctions}
            onChange={(e) => setShowFunctions(e.target.checked)}
          />
          show functions
        </label>
        <div className="meta">
          {graph.summary.files} files · {graph.summary.functions} functions ·{' '}
          {graph.summary.modules} modules · {graph.summary.types} types ·{' '}
          {graph.summary.imports_resolved}/{graph.summary.imports_total} imports ·{' '}
          {graph.summary.calls_resolved}/{graph.summary.calls_total} calls · @{graph.graph_id}
        </div>
      </header>

      <div className="loom-canvas">
        <CytoscapeComponent
          elements={elements}
          stylesheet={cytoscapeStyle}
          layout={
            {
              name: 'dagre',
              animate: false,
              rankDir: 'TB',
              spacingFactor: 1,
              fit: true
            } as cytoscape.LayoutOptions
          }
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
          <span>click a node to inspect · drag to disentangle · scroll to zoom</span>
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
