// Zustand store for the viewer.
// At M1: tracks the current graph, the load lifecycle, and selection.
// Effects mode (M2) and Hashes mode (M3) will hang off this store.

import { create } from 'zustand';
import type { CodeGraph } from './types';

export type Mode = 'graph' | 'effects' | 'hashes';

interface ViewerStore {
  graph: CodeGraph | null;
  loading: boolean;
  error: string | null;
  selected: number | null;
  mode: Mode;

  load: (graphId: string) => Promise<void>;
  select: (id: number | null) => void;
  setMode: (m: Mode) => void;
}

export const useViewer = create<ViewerStore>((set) => ({
  graph: null,
  loading: false,
  error: null,
  selected: null,
  mode: 'graph',

  async load(graphId) {
    set({ loading: true, error: null, graph: null });
    try {
      const resp = await fetch(`/api/graph/${encodeURIComponent(graphId)}`);
      if (!resp.ok) {
        const body = await resp.text();
        set({
          loading: false,
          error: `HTTP ${resp.status}: ${body.slice(0, 200)}`
        });
        return;
      }
      const graph = (await resp.json()) as CodeGraph;
      set({ graph, loading: false });
    } catch (e) {
      set({
        loading: false,
        error: e instanceof Error ? e.message : String(e)
      });
    }
  },

  select(id) {
    set({ selected: id });
  },

  setMode(m) {
    set({ mode: m });
  }
}));
