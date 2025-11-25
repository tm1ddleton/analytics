import { useState, useEffect } from 'react';
import { Container, Typography, Box } from '@mui/material';
import { AssetSelector } from './components/AssetSelector';
import { AnalyticsPresets, PRESETS } from './components/AnalyticsPresets';
import { ApiUrlDisplay } from './components/ApiUrlDisplay';
import { Chart } from './components/Chart';
import { ReplayControls } from './components/ReplayControls';
import { getAssets, getAnalytics, createReplaySession, stopReplaySession, buildApiUrl } from './services/api';
import { connectToStream, closeStream } from './services/sse';
import type { Preset, ChartData, AnalyticUpdate, ProgressUpdate } from './types';

function App() {
  // Assets state
  const [availableAssets, setAvailableAssets] = useState<string[]>([]);
  const [selectedAssets, setSelectedAssets] = useState<string[]>([]);
  const [assetsLoading, setAssetsLoading] = useState(true);
  const [assetsError, setAssetsError] = useState<string | null>(null);

  // Analytics state
  const [selectedPreset, setSelectedPreset] = useState<Preset | null>(PRESETS[0]);

  // Chart state
  const [chartData, setChartData] = useState<ChartData[]>([]);
  const [chartLoading, setChartLoading] = useState(false);
  const [chartError, setChartError] = useState<string | null>(null);

  // Replay state
  const [replaySessionId, setReplaySessionId] = useState<string | null>(null);
  const [replayStatus, setReplayStatus] = useState<'idle' | 'running' | 'stopped' | 'completed'>('idle');
  const [replayProgress, setReplayProgress] = useState(0);
  const [replayError, setReplayError] = useState<string | null>(null);

  // SSE state
  const [eventSource, setEventSource] = useState<EventSource | null>(null);

  // Load assets on mount
  useEffect(() => {
    async function loadAssets() {
      try {
        const assets = await getAssets();
        setAvailableAssets(assets.map((a) => a.key));
        setAssetsLoading(false);
      } catch (error) {
        setAssetsError(error instanceof Error ? error.message : 'Failed to load assets');
        setAssetsLoading(false);
      }
    }
    loadAssets();
  }, []);

  // Fetch analytics data when preset or assets change
  useEffect(() => {
    if (selectedAssets.length > 0 && selectedPreset && replayStatus === 'idle') {
      fetchAnalytics();
    }
  }, [selectedAssets, selectedPreset]);

  const fetchAnalytics = async () => {
    if (selectedAssets.length === 0) {
      setChartError('Please select at least one asset');
      return;
    }

    if (!selectedPreset) {
      return;
    }

    setChartLoading(true);
    setChartError(null);

    try {
      // Fetch data for all selected assets
      const responses = await Promise.all(
        selectedAssets.map((asset) =>
          getAnalytics(asset, selectedPreset.analyticType, {
            start: '2024-01-01',
            end: '2024-12-31',
            window: selectedPreset.params?.window,
          })
        )
      );

      // Transform data into chart format
      const dataMap = new Map<string, ChartData>();

      responses.forEach((response) => {
        response.data.forEach((point) => {
          if (!dataMap.has(point.timestamp)) {
            dataMap.set(point.timestamp, { timestamp: point.timestamp });
          }
          const entry = dataMap.get(point.timestamp)!;
          entry[response.asset] = point.value;
        });
      });

      const chartData = Array.from(dataMap.values()).sort(
        (a, b) => new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime()
      );

      setChartData(chartData);
      setChartLoading(false);
    } catch (error) {
      setChartError(error instanceof Error ? error.message : 'Failed to fetch analytics');
      setChartLoading(false);
    }
  };

  const handleStartReplay = async () => {
    if (selectedAssets.length === 0) {
      setReplayError('Please select at least one asset');
      return;
    }

    if (!selectedPreset) {
      setReplayError('Please select an analytic');
      return;
    }

    setReplayError(null);
    setReplayProgress(0);
    setChartData([]); // Clear existing data

    try {
      // Create replay session
      const session = await createReplaySession(
        selectedAssets,
        [
          {
            type: selectedPreset.analyticType,
            parameters: selectedPreset.params
              ? { window: selectedPreset.params.window?.toString() || '' }
              : {},
          },
        ],
        '2024-01-01',
        '2024-12-31'
      );

      setReplaySessionId(session.session_id);
      setReplayStatus('running');

      // Connect to SSE stream
      const es = connectToStream(
        session.session_id,
        handleUpdate,
        handleProgress,
        handleComplete,
        handleError
      );
      setEventSource(es);
    } catch (error) {
      setReplayError(error instanceof Error ? error.message : 'Failed to start replay');
      setReplayStatus('idle');
    }
  };

  const handleStopReplay = async () => {
    if (replaySessionId && eventSource) {
      try {
        await stopReplaySession(replaySessionId);
        closeStream(eventSource);
        setEventSource(null);
        setReplayStatus('stopped');
      } catch (error) {
        setReplayError(error instanceof Error ? error.message : 'Failed to stop replay');
      }
    }
  };

  const handleUpdate = (data: AnalyticUpdate) => {
    setChartData((prevData) => {
      const existingEntry = prevData.find((d) => d.timestamp === data.timestamp);
      if (existingEntry) {
        return prevData.map((d) =>
          d.timestamp === data.timestamp ? { ...d, [data.asset]: data.value } : d
        );
      } else {
        const newData = [...prevData, { timestamp: data.timestamp, [data.asset]: data.value }];
        return newData.sort(
          (a, b) => new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime()
        );
      }
    });
  };

  const handleProgress = (progress: ProgressUpdate) => {
    setReplayProgress(progress.progress);
  };

  const handleComplete = () => {
    setReplayStatus('completed');
    if (eventSource) {
      closeStream(eventSource);
      setEventSource(null);
    }
  };

  const handleError = (error: Error) => {
    setReplayError(error.message);
    setReplayStatus('idle');
    if (eventSource) {
      closeStream(eventSource);
      setEventSource(null);
    }
  };

  // Build API URL for display
  const apiUrl =
    selectedAssets.length > 0 && selectedPreset
      ? buildApiUrl(selectedAssets[0], selectedPreset.analyticType, {
          start: '2024-01-01',
          end: '2024-12-31',
          window: selectedPreset.params?.window,
        })
      : 'Select an asset and analytic to see the API URL';

  return (
    <Container maxWidth="lg" sx={{ py: 4 }}>
      <Typography variant="h4" component="h1" gutterBottom>
        Analytics Dashboard
      </Typography>

      <Box display="flex" flexDirection="column" gap={2}>
        <AssetSelector
          availableAssets={availableAssets}
          selectedAssets={selectedAssets}
          onSelectionChange={setSelectedAssets}
          loading={assetsLoading}
          error={assetsError}
        />

        <AnalyticsPresets
          selectedPreset={selectedPreset}
          onPresetChange={setSelectedPreset}
        />

        <ApiUrlDisplay url={apiUrl} />

        <Chart
          data={chartData}
          assets={selectedAssets}
          analyticType={selectedPreset?.label || ''}
          loading={chartLoading}
          error={chartError}
        />

        <ReplayControls
          onStartReplay={handleStartReplay}
          onStopReplay={handleStopReplay}
          status={replayStatus}
          progress={replayProgress}
          error={replayError}
        />
      </Box>
    </Container>
  );
}

export default App;
