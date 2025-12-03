import type { AnalyticUpdate, ProgressUpdate } from '../types';

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
    return window.location.origin;
  }
  
  // Fallback for SSR or non-browser environments
  return 'http://localhost:3000';
}

const API_BASE_URL = getApiBaseUrl();

export function connectToStream(
  sessionId: string,
  onUpdate: (data: AnalyticUpdate) => void,
  onProgress: (progress: ProgressUpdate) => void,
  onComplete: () => void,
  onError: (error: Error) => void
): EventSource {
  const url = `${API_BASE_URL}/stream/${sessionId}`;
  const eventSource = new EventSource(url);

  eventSource.addEventListener('update', (event) => {
    try {
      const data = JSON.parse(event.data) as AnalyticUpdate;
      onUpdate(data);
    } catch (error) {
      console.error('Failed to parse update event:', error);
    }
  });

  eventSource.addEventListener('progress', (event) => {
    try {
      const data = JSON.parse(event.data) as ProgressUpdate;
      onProgress(data);
    } catch (error) {
      console.error('Failed to parse progress event:', error);
    }
  });

  eventSource.addEventListener('complete', () => {
    onComplete();
    eventSource.close();
  });

  eventSource.addEventListener('connected', (event) => {
    console.log('Connected to stream:', event.data);
  });

  eventSource.onerror = (error) => {
    console.error('SSE connection error:', error);
    onError(new Error('Connection lost'));
    eventSource.close();
  };

  return eventSource;
}

export function closeStream(eventSource: EventSource): void {
  eventSource.close();
}

