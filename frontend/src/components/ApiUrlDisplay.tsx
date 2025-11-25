import { useState } from 'react';
import {
  Paper,
  Typography,
  TextField,
  Button,
  Box,
  Snackbar,
} from '@mui/material';
import ContentCopyIcon from '@mui/icons-material/ContentCopy';

interface ApiUrlDisplayProps {
  url: string;
}

export function ApiUrlDisplay({ url }: ApiUrlDisplayProps) {
  const [copySuccess, setCopySuccess] = useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(url);
      setCopySuccess(true);
    } catch (error) {
      console.error('Failed to copy:', error);
    }
  };

  return (
    <Paper elevation={1} sx={{ p: 2 }}>
      <Typography variant="h6" gutterBottom>
        API URL
      </Typography>
      <Box display="flex" gap={1} alignItems="center">
        <TextField
          fullWidth
          size="small"
          value={url}
          InputProps={{
            readOnly: true,
          }}
        />
        <Button
          variant="contained"
          startIcon={<ContentCopyIcon />}
          onClick={handleCopy}
        >
          Copy
        </Button>
      </Box>
      <Typography variant="caption" color="text.secondary" mt={1}>
        Use this URL with curl to query the API directly
      </Typography>

      <Snackbar
        open={copySuccess}
        autoHideDuration={2000}
        onClose={() => setCopySuccess(false)}
        message="URL copied to clipboard!"
      />
    </Paper>
  );
}

