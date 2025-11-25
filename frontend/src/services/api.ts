import axios from 'axios';
import type {
  Asset,
  AnalyticsResponse,
  AnalyticConfig,
  SessionResponse,
} from '../types';

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000';

const apiClient = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

export async function getAssets(): Promise<Asset[]> {
  try {
    const response = await apiClient.get<{ assets: Asset[] }>('/assets');
    return response.data.assets;
  } catch (error) {
    console.error('Failed to fetch assets:', error);
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

