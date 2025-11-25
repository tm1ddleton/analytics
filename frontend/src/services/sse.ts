import type { AnalyticUpdate, ProgressUpdate } from '../types';

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000';

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

