# 🚀 Quick Start Guide

Get the Analytics POC running in 2 minutes!

## Prerequisites

- **Rust** toolchain installed (`rustc --version`)
- **Node.js 18+** installed (`node --version`)
  - Uses Vite 5.x (compatible with Node 18-20)
- Git repository cloned

## Step 1: Run the Demo

From the project root:

```bash
./run-demo.sh
```

This single command will:
1. Build the Rust backend
2. Start the API server (port 3000)
3. Start the React frontend (port 5173)
4. Wait for both to be ready

## Step 2: Open Your Browser

Navigate to: **http://localhost:5173**

You should see the Analytics Dashboard!

## Step 3: Try the Features

### Pull-Mode Query (Historical Data)

1. **Select an asset**: Check "AAPL"
2. **Choose analytics**: Click "20-Day Volatility"
3. **View chart**: Historical data loads automatically

The chart shows the complete time-series from 2024-01-01 to 2024-12-31.

### Push-Mode Replay (Real-Time)

1. **Select assets**: Check "AAPL" and "MSFT"
2. **Choose analytics**: Click "10-Day Volatility"
3. **Start replay**: Click "Start Replay" button
4. **Watch**: Chart updates in real-time as analytics compute!
5. **Monitor**: Progress bar shows how far along the replay is
6. **Stop**: Click "Stop" to end the session

### Copy API URL

1. **Select asset & analytic** as above
2. **Click "Copy"** in the API URL section
3. **Test with curl**:

```bash
curl "http://localhost:3000/analytics/AAPL/volatility?start=2024-01-01&end=2024-12-31&window=20"
```

## Step 4: Stop the Demo

Press **Ctrl+C** in the terminal, or run:

```bash
./stop-demo.sh
```

---

## Troubleshooting

### Port Already in Use

If you see "address already in use" errors:

```bash
# Kill any processes on ports 3000 and 5173
lsof -ti:3000 | xargs kill -9
lsof -ti:5173 | xargs kill -9
```

Then run `./run-demo.sh` again.

### Backend Won't Start

Check the backend log:

```bash
cat backend.log
```

Common issues:
- **Database file missing**: Run data download examples first
- **Port 3000 blocked**: Another service is using that port

### Frontend Won't Start

Check the frontend log:

```bash
cat frontend.log
```

Common issues:
- **Dependencies not installed**: Run `cd frontend && npm install`
- **Port 5173 blocked**: Another Vite server is running

### No Data in Charts

The demo expects data to be available in the SQLite database. If you see "No data available" errors:

1. **Download sample data**:

```bash
# Run the Yahoo Finance downloader example
cargo run --example download_yahoo_data
```

2. **Or use test data**:

```bash
# Run integration tests to populate test database
cargo test
```

---

## What's Next?

- Read the [README.md](README.md) for architecture details
- Check [docs/API.md](docs/API.md) for API documentation
- Explore [examples/](examples/) for more code examples
- Review [agent-os/product/roadmap.md](agent-os/product/roadmap.md) for future plans

---

## Demo Video Script

Want to show this to someone? Here's a 2-minute demo script:

1. **"This is a real-time analytics engine built in Rust"**
   - Show the dashboard

2. **"Let's query some historical volatility data"**
   - Select AAPL
   - Click "20-Day Volatility"
   - Show the chart loading

3. **"Now let's see it compute in real-time"**
   - Add MSFT to the selection
   - Click "Start Replay"
   - Watch the chart update live

4. **"It's all backed by a REST API"**
   - Show the API URL
   - Copy and run curl command in terminal

5. **"The engine uses a DAG to track dependencies"**
   - Explain: Data → Returns → Volatility
   - Everything updates incrementally

6. **"Built with Rust for performance and React for the UI"**
   - Show it's fast and smooth

**Total time: 2 minutes** ⏱️

Enjoy! 🎉

