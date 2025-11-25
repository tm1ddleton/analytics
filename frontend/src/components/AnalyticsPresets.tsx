import { Paper, Typography, ButtonGroup, Button } from '@mui/material';
import type { Preset } from '../types';

export const PRESETS: Preset[] = [
  { label: 'Returns', analyticType: 'returns' },
  { label: '10-Day Volatility', analyticType: 'volatility', params: { window: 10 } },
  { label: '20-Day Volatility', analyticType: 'volatility', params: { window: 20 } },
  { label: '50-Day Volatility', analyticType: 'volatility', params: { window: 50 } },
];

interface AnalyticsPresetsProps {
  selectedPreset: Preset | null;
  onPresetChange: (preset: Preset) => void;
}

export function AnalyticsPresets({
  selectedPreset,
  onPresetChange,
}: AnalyticsPresetsProps) {
  return (
    <Paper elevation={1} sx={{ p: 2 }}>
      <Typography variant="h6" gutterBottom>
        Analytics
      </Typography>
      <ButtonGroup variant="outlined" size="medium">
        {PRESETS.map((preset) => (
          <Button
            key={preset.label}
            variant={
              selectedPreset?.label === preset.label ? 'contained' : 'outlined'
            }
            onClick={() => onPresetChange(preset)}
          >
            {preset.label}
          </Button>
        ))}
      </ButtonGroup>
    </Paper>
  );
}

