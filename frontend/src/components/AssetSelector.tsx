import {
  Paper,
  Typography,
  FormGroup,
  FormControlLabel,
  Checkbox,
  Button,
  Box,
  CircularProgress,
} from '@mui/material';

interface AssetSelectorProps {
  availableAssets: string[];
  selectedAssets: string[];
  onSelectionChange: (assets: string[]) => void;
  loading: boolean;
  error: string | null;
}

export function AssetSelector({
  availableAssets,
  selectedAssets,
  onSelectionChange,
  loading,
  error,
}: AssetSelectorProps) {
  const handleToggle = (asset: string) => {
    const newSelection = selectedAssets.includes(asset)
      ? selectedAssets.filter((a) => a !== asset)
      : [...selectedAssets, asset];
    onSelectionChange(newSelection);
  };

  const handleSelectAll = () => {
    onSelectionChange(availableAssets);
  };

  const handleClearAll = () => {
    onSelectionChange([]);
  };

  if (loading) {
    return (
      <Paper elevation={1} sx={{ p: 2 }}>
        <Typography variant="h6" gutterBottom>
          Select Assets
        </Typography>
        <Box display="flex" justifyContent="center" p={2}>
          <CircularProgress />
        </Box>
      </Paper>
    );
  }

  return (
    <Paper elevation={1} sx={{ p: 2 }}>
      <Typography variant="h6" gutterBottom>
        Select Assets
      </Typography>
      
      {error && (
        <Typography color="error" variant="caption" display="block" mb={1}>
          {error}
        </Typography>
      )}

      <FormGroup row>
        {availableAssets.map((asset) => (
          <FormControlLabel
            key={asset}
            control={
              <Checkbox
                checked={selectedAssets.includes(asset)}
                onChange={() => handleToggle(asset)}
              />
            }
            label={asset}
          />
        ))}
      </FormGroup>

      <Box mt={2} display="flex" gap={1}>
        <Button size="small" variant="outlined" onClick={handleSelectAll}>
          Select All
        </Button>
        <Button size="small" variant="outlined" onClick={handleClearAll}>
          Clear All
        </Button>
      </Box>
    </Paper>
  );
}

