// Type definitions for the Analytics Dashboard

export interface Asset {
  key: string;
  type: string;
  name: string;
  data_available_from?: string;
  data_available_to?: string;
}

export interface AnalyticsParams {
  start: string;
  end: string;
  window?: number;
}

export interface AnalyticDataPoint {
  timestamp: string;
  value: number | null;
}

export interface AnalyticsResponse {
  asset: string;
  analytic: string;
  parameters: Record<string, string>;
  start_date: string;
  end_date: string;
  data: AnalyticDataPoint[];
}

export interface ChartData {
  timestamp: string;
  [asset: string]: number | string | null;
}

export interface Preset {
  label: string;
  analyticType: 'returns' | 'volatility';
  params?: { window?: number };
}

export interface AnalyticConfig {
  type: string;
  parameters?: Record<string, string>;
}

export interface SessionResponse {
  session_id: string;
  status: string;
  assets: string[];
  analytics: string[];
  start_date: string;
  end_date: string;
  stream_url: string;
}

export interface AnalyticUpdate {
  asset: string;
  analytic: string;
  timestamp: string;
  value: number;
}

export interface ProgressUpdate {
  current_date: string;
  progress: number;
}

// DAG Visualization Types
export interface NodePosition {
  x: number;
  y: number;
}

export interface VisualizationNode {
  id: number;
  node_type: string;
  analytic_type: string;
  assets: string[];
  params: Record<string, string>;
  position?: NodePosition;
  data_url?: string;
  code_url?: string;
  description?: string;
}

export interface VisualizationEdge {
  source: number;
  target: number;
  label?: string;
}

export interface DagMetadata {
  node_count: number;
  edge_count: number;
  api_base_url: string;
  code_base_url: string;
}

export interface DagVisualization {
  nodes: VisualizationNode[];
  edges: VisualizationEdge[];
  metadata: DagMetadata;
}

