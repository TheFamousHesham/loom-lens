import { Viewer } from './Viewer';

// Tiny "router": parse `graph_id` from `/r/<id>...` and pass to <Viewer>.
// React Router becomes worth its weight at M3 (multi-page diff view).
function parseGraphIdFromPath(): string | null {
  const m = window.location.pathname.match(/^\/r\/([0-9a-fA-F]{6,16})/);
  return m && m[1] ? m[1] : null;
}

export function App() {
  const graphId = parseGraphIdFromPath();

  if (!graphId) {
    return (
      <div className="loom-empty">
        <div>
          <strong>Loom Lens</strong>
        </div>
        <div>
          Open a viewer URL of the form <code>/r/&lt;graph_id&gt;</code>.
        </div>
        <div>
          Run <code>loom-lens analyze &lt;path&gt;</code> in a terminal first; it
          prints the URL.
        </div>
      </div>
    );
  }

  return <Viewer graphId={graphId} />;
}
