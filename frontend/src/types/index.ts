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

