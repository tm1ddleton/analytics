import axios from 'axios';
import type {
  Asset,
  AnalyticsResponse,
  AnalyticConfig,
  SessionResponse,
  DagVisualization,
} from '../types';

// Use relative URLs if VITE_API_URL is empty or not set (for Docker/production with nginx proxy)
// Otherwise use the provided URL or default to localhost for development
// Note: We use a function to get the origin at runtime to ensure the port is included
// In production builds, VITE_API_URL="" gets replaced with "" by Vite
const envApiUrl = import.meta.env.VITE_API_URL;

// Function to get the API base URL at runtime (not at build time)
function getApiBaseUrl(): string {
  // If VITE_API_URL is explicitly set and non-empty, use it
  if (envApiUrl && envApiUrl !== '' && envApiUrl.trim() !== '') {
    return envApiUrl;
  }
  
  // Otherwise, use the current page origin (includes port, e.g., http://localhost:5173)
  if (typeof window !== 'undefined' && window.location) {
    // Explicitly construct the origin with port to ensure it's always included
    const protocol = window.location.protocol; // http: or https:
    const hostname = window.location.hostname; // localhost or 127.0.0.1
    const port = window.location.port; // 5173 (empty string for default ports 80/443)
    
    // Use 127.0.0.1 instead of localhost to avoid browser hostname resolution issues
    // that might strip the port
    const resolvedHostname = hostname === 'localhost' ? '127.0.0.1' : hostname;
    
    // Construct origin explicitly to ensure port is always included
    if (port) {
      return `${protocol}//${resolvedHostname}:${port}`;
    } else {
      // Default ports - use origin as-is (shouldn't happen in our case since we're on 5173)
      return window.location.origin;
    }
  }
  
  // Fallback for SSR or non-browser environments
  return 'http://localhost:3000';
}

const API_BASE_URL = getApiBaseUrl();

// Debug: Log the base URL being used
if (typeof window !== 'undefined') {
  console.log('API Base URL:', API_BASE_URL, 'Origin:', window.location.origin);
}

const apiClient = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Debug: Log the actual request URL being constructed
apiClient.interceptors.request.use((config) => {
  console.log('Axios Request Config:', {
    baseURL: config.baseURL,
    url: config.url,
    fullURL: config.baseURL ? `${config.baseURL}${config.url}` : config.url,
  });
  return config;
});

export async function getAssets(): Promise<Asset[]> {
  try {
    // Construct full URL to avoid axios baseURL issues
    // Remove leading slash from path since we're using full URL
    const url = API_BASE_URL.endsWith('/') 
      ? `${API_BASE_URL}assets` 
      : `${API_BASE_URL}/assets`;
    
    console.log('Making request to full URL:', url);
    const response = await axios.get<{ assets: Asset[] }>(url, {
      headers: {
        'Content-Type': 'application/json',
      },
    });
    return response.data.assets;
  } catch (error) {
    console.error('Failed to fetch assets:', error);
    if (axios.isAxiosError(error)) {
      console.error('Request URL:', error.config?.url);
      console.error('Request baseURL:', error.config?.baseURL);
    }
    throw new Error('Failed to fetch assets. Please check if the server is running.');
  }
}

export async function getAnalytics(
  asset: string,
  analyticType: string,
  params: { start: string; end: string; window?: number }
): Promise<AnalyticsResponse> {
  try {
    const queryParams = new URLSearchParams({
      start: params.start,
      end: params.end,
    });
    
    if (params.window !== undefined) {
      queryParams.append('window', params.window.toString());
    }

    const response = await apiClient.get<AnalyticsResponse>(
      `/analytics/${asset}/${analyticType}?${queryParams.toString()}`
    );
    return response.data;
  } catch (error) {
    console.error(`Failed to fetch analytics for ${asset}:`, error);
    throw new Error(`Failed to fetch analytics for ${asset}`);
  }
}

export async function createReplaySession(
  assets: string[],
  analytics: AnalyticConfig[],
  startDate: string,
  endDate: string
): Promise<SessionResponse> {
  try {
    const response = await apiClient.post<SessionResponse>('/replay', {
      assets,
      analytics,
      start_date: startDate,
      end_date: endDate,
    });
    return response.data;
  } catch (error) {
    console.error('Failed to create replay session:', error);
    throw new Error('Failed to create replay session');
  }
}

export async function stopReplaySession(sessionId: string): Promise<void> {
  try {
    await apiClient.delete(`/replay/${sessionId}`);
  } catch (error) {
    console.error('Failed to stop replay session:', error);
    throw new Error('Failed to stop replay session');
  }
}

export async function getDagVisualization(
  asset: string,
  analyticType: string,
  params: { start: string; end: string; window?: number; override?: string }
): Promise<DagVisualization> {
  try {
    const queryParams = new URLSearchParams({
      asset,
      analytic: analyticType,
      start: params.start,
      end: params.end,
    });
    
    if (params.window !== undefined) {
      queryParams.append('window', params.window.toString());
    }
    
    if (params.override) {
      queryParams.append('override', params.override);
    }

    const response = await apiClient.get<DagVisualization>(
      `/dag/visualize?${queryParams.toString()}`
    );
    return response.data;
  } catch (error) {
    console.error('Failed to fetch DAG visualization:', error);
    throw new Error('Failed to fetch DAG visualization');
  }
}

// Helper function to build API URL for display
export function buildApiUrl(
  asset: string,
  analyticType: string,
  params: { start: string; end: string; window?: number }
): string {
  const queryParams = new URLSearchParams({
    start: params.start,
    end: params.end,
  });
  
  if (params.window !== undefined) {
    queryParams.append('window', params.window.toString());
  }

  return `${API_BASE_URL}/analytics/${asset}/${analyticType}?${queryParams.toString()}`;
}

