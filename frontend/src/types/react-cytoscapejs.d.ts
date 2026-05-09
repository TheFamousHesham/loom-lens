// Minimal type shim — `react-cytoscapejs` ships JS only.

declare module 'react-cytoscapejs' {
  import type { CSSProperties, ComponentType } from 'react';
  import type cytoscape from 'cytoscape';

  export interface CytoscapeComponentProps {
    elements: cytoscape.ElementDefinition[];
    stylesheet?: cytoscape.StylesheetCSS[] | cytoscape.StylesheetJson;
    layout?: cytoscape.LayoutOptions;
    style?: CSSProperties;
    className?: string;
    cy?: (cy: cytoscape.Core) => void;
    zoom?: number;
    pan?: { x: number; y: number };
    minZoom?: number;
    maxZoom?: number;
    zoomingEnabled?: boolean;
    userZoomingEnabled?: boolean;
    panningEnabled?: boolean;
    userPanningEnabled?: boolean;
    boxSelectionEnabled?: boolean;
    autoungrabify?: boolean;
    autounselectify?: boolean;
    autolock?: boolean;
    headless?: boolean;
  }

  const CytoscapeComponent: ComponentType<CytoscapeComponentProps>;
  export default CytoscapeComponent;
}
