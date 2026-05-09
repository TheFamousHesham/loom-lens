// Wire types mirroring the JSON emitted by the Rust `loom-lens-core::CodeGraph`.
// Keep in sync with crates/core/src/graph.rs.

export type Language = 'python' | 'typescript' | 'javascript' | 'rust';

export interface Span {
  byte_start: number;
  byte_end: number;
  line_start: number;
  line_end: number;
  col_start: number;
  col_end: number;
}

export interface FileNode {
  id: number;
  kind: 'file';
  path: string;
  language: Language;
  lines: number;
  span: Span;
}

export interface ModuleNode {
  id: number;
  kind: 'module';
  name: string;
  file: number;
  span: Span;
}

export interface FunctionNode {
  id: number;
  kind: 'function';
  name: string;
  qualified_name: string;
  signature: string;
  span: Span;
}

export interface TypeNode {
  id: number;
  kind: 'type';
  name: string;
  qualified_name: string;
  span: Span;
}

export type GraphNode = FileNode | ModuleNode | FunctionNode | TypeNode;

export type EdgeKind = 'contains' | 'calls' | 'imports' | 'references';

export interface Edge {
  from: number;
  to: number;
  kind: EdgeKind;
  sites: Span[];
}

export interface ParseErrorRecord {
  file: string;
  line: number;
  message: string;
}

export interface Summary {
  files: number;
  functions: number;
  modules: number;
  types: number;
  calls_resolved: number;
  calls_total: number;
  imports_resolved: number;
  imports_total: number;
  languages: Record<string, number>;
  elapsed_ms: number;
  parse_errors: ParseErrorRecord[];
}

export interface CodeGraph {
  graph_id: string;
  repo_root: string;
  nodes: GraphNode[];
  edges: Edge[];
  summary: Summary;
  generated_at: string;
}
