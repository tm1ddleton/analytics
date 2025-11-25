import {
  Paper,
  Typography,
  Box,
  CircularProgress,
} from '@mui/material';
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from 'recharts';
import type { ChartData } from '../types';

interface ChartProps {
  data: ChartData[];
  assets: string[];
  analyticType: string;
  loading: boolean;
  error: string | null;
}

const COLORS = [
  '#1976d2', // blue
  '#ff9800', // orange
  '#4caf50', // green
  '#f44336', // red
  '#9c27b0', // purple
];

export function Chart({ data, assets, analyticType, loading, error }: ChartProps) {
  if (loading) {
    return (
      <Paper elevation={1} sx={{ p: 2, height: 400 }}>
        <Typography variant="h6" gutterBottom>
          {analyticType || 'Analytics'} Chart
        </Typography>
        <Box
          display="flex"
          justifyContent="center"
          alignItems="center"
          height="calc(100% - 40px)"
        >
          <CircularProgress />
        </Box>
      </Paper>
    );
  }

  if (error) {
    return (
      <Paper elevation={1} sx={{ p: 2, height: 400 }}>
        <Typography variant="h6" gutterBottom>
          {analyticType || 'Analytics'} Chart
        </Typography>
        <Typography color="error" variant="body2">
          {error}
        </Typography>
      </Paper>
    );
  }

  if (data.length === 0) {
    return (
      <Paper elevation={1} sx={{ p: 2, height: 400 }}>
        <Typography variant="h6" gutterBottom>
          {analyticType || 'Analytics'} Chart
        </Typography>
        <Typography variant="body2" color="text.secondary">
          No data available. Select assets and an analytic to view the chart.
        </Typography>
      </Paper>
    );
  }

  return (
    <Paper elevation={1} sx={{ p: 2, height: 400 }}>
      <Typography variant="h6" gutterBottom>
        {analyticType || 'Analytics'} Chart
      </Typography>
      <ResponsiveContainer width="100%" height="90%">
        <LineChart data={data} key={data.length}>
          <CartesianGrid strokeDasharray="3 3" />
          <XAxis
            dataKey="timestamp"
            tick={{ fontSize: 12 }}
            tickFormatter={(value) => {
              try {
                const date = new Date(value);
                return `${date.getMonth() + 1}/${date.getDate()}`;
              } catch {
                return value;
              }
            }}
          />
          <YAxis tick={{ fontSize: 12 }} />
          <Tooltip
            labelFormatter={(value) => {
              try {
                return new Date(value as string).toLocaleDateString();
              } catch {
                return value;
              }
            }}
          />
          <Legend />
          {assets.map((asset, index) => (
            <Line
              key={asset}
              type="monotone"
              dataKey={asset}
              stroke={COLORS[index % COLORS.length]}
              dot={false}
              name={asset}
              connectNulls
              isAnimationActive={false}
            />
          ))}
        </LineChart>
      </ResponsiveContainer>
    </Paper>
  );
}

