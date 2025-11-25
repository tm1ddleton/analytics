import {
  Paper,
  Typography,
  Button,
  Box,
  LinearProgress,
} from '@mui/material';
import PlayArrowIcon from '@mui/icons-material/PlayArrow';
import StopIcon from '@mui/icons-material/Stop';

interface ReplayControlsProps {
  onStartReplay: () => Promise<void>;
  onStopReplay: () => Promise<void>;
  status: 'idle' | 'running' | 'stopped' | 'completed';
  progress: number;
  error: string | null;
}

export function ReplayControls({
  onStartReplay,
  onStopReplay,
  status,
  progress,
  error,
}: ReplayControlsProps) {
  const isRunning = status === 'running';

  return (
    <Paper elevation={1} sx={{ p: 2 }}>
      <Typography variant="h6" gutterBottom>
        Replay Controls
      </Typography>

      <Box display="flex" gap={1} mb={2}>
        <Button
          variant="contained"
          startIcon={<PlayArrowIcon />}
          onClick={onStartReplay}
          disabled={isRunning}
        >
          Start Replay
        </Button>
        <Button
          variant="outlined"
          startIcon={<StopIcon />}
          onClick={onStopReplay}
          disabled={!isRunning}
        >
          Stop
        </Button>
      </Box>

      <Box mb={1}>
        <Typography variant="body2" color="text.secondary">
          Status: {status.charAt(0).toUpperCase() + status.slice(1)}
        </Typography>
      </Box>

      <Box mb={1}>
        <Typography variant="body2" color="text.secondary" gutterBottom>
          Progress: {Math.round(progress * 100)}%
        </Typography>
        <LinearProgress variant="determinate" value={progress * 100} />
      </Box>

      {error && (
        <Typography color="error" variant="caption">
          {error}
        </Typography>
      )}
    </Paper>
  );
}

