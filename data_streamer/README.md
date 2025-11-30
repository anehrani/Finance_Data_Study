# Bybit TradFi Data Streamer

A comprehensive Rust application for downloading and streaming TradFi (Traditional Finance) data from Bybit, including historical data backfill, real-time tick streaming, and OHLCV bar aggregation.

## Features

### 1. **Historical Data Download** (REST API)
- Downloads up to 1000 days of daily OHLCV data per symbol
- Creates `MARKETS.TXT` index file for backtesting tools
- Format compatible with legacy trading systems

### 2. **Real-time Tick Streaming** (WebSocket API)
- Captures every single trade/price change
- No missed ticks (unlike REST polling)
- Separate streams for Spot and Linear categories

### 3. **OHLCV Bar Aggregation**
- Real-time 1-minute bar construction from tick data
- Automatic bar completion and file writing
- Suitable for algorithmic trading and analysis

## Supported Assets

### Spot Category (XUSDT - Tokenized Stocks)
**11 genuine tokenized stocks** (crypto tokens excluded):
- **Tech**: AAPLXUSDT (Apple), TSLAXUSDT (Tesla), NVDAXUSDT (Nvidia), GOOGLXUSDT (Google), METAXUSDT (Meta)
- **E-commerce**: AMZNXUSDT (Amazon), COINXUSDT (Coinbase)
- **Finance**: HOODXUSDT (Robinhood)
- **Food**: MCDXUSDT (McDonald's)
- **Indices**: SPXUSDT (S&P 500)
- **Metals**: XAUTUSDT (Gold)

> **Note**: The filter excludes 21 cryptocurrency tokens that happen to end in "XUSDT" (e.g., TRXUSDT, AVAXUSDT, ICXUSDT). See [FILTERING.md](FILTERING.md) for details.

### Linear Category (Perpetuals - Indices/Commodities/Metals)
**3 TradFi assets**:
- **Indices**: SPXUSDT (S&P 500)
- **Commodities**: GASUSDT (Natural Gas)
- **Metals**: XAUTUSDT (Gold)

> **Total**: 14 genuine TradFi assets (11 unique stocks + 3 linear instruments)

## Directory Structure

```
data_streamer/
├── historical_data/
│   ├── spot/
│   │   ├── MARKETS.TXT          # Index file with absolute paths
│   │   ├── AAPLXUSDT.TXT        # Daily OHLC data
│   │   ├── TSLAXUSDT.TXT
│   │   └── ...
│   └── linear/
│       ├── MARKETS.TXT
│       ├── XAUTUSDT.TXT
│       └── ...
├── tick_data/
│   ├── spot/
│   │   ├── AAPLXUSDT.txt        # Raw tick data
│   │   └── ...
│   └── linear/
│       ├── XAUTUSDT.txt
│       └── ...
└── bar_data/
    ├── spot/
    │   ├── AAPLXUSDT.txt        # 1-minute OHLCV bars
    │   └── ...
    └── linear/
        ├── XAUTUSDT.txt
        └── ...
```

## Data Formats

### Historical Data (Daily)
**Location**: `historical_data/{category}/{SYMBOL}.TXT`

**Format**: `YYYYMMDD Open High Low Close`

**Example**:
```
20250701 184.65 217.62 184.65 212.69
20250702 212.69 215.71 208.00 215.71
20250703 215.71 267.01 211.00 214.31
```

### Tick Data (Real-time)
**Location**: `tick_data/{category}/{SYMBOL}.txt`

**Format**: `timestamp,price,volume,side`

**Example**:
```
1701234567890,43250.50,0.125,Buy
1701234568123,43251.00,0.250,Sell
1701234569456,43250.75,0.100,Buy
```

**Fields**:
- `timestamp`: Unix timestamp in milliseconds
- `price`: Trade price
- `volume`: Trade volume
- `side`: Trade side (Buy/Sell)

### Bar Data (1-minute OHLCV)
**Location**: `bar_data/{category}/{SYMBOL}.txt`

**Format**: `YYYYMMDD HH:MM:SS Open High Low Close Volume`

**Example**:
```
20250130 12:00:00 43250.50 43255.00 43248.00 43252.00 1.250
20250130 12:01:00 43252.00 43260.00 43250.00 43258.00 2.100
20250130 12:02:00 43258.00 43258.00 43253.00 43255.00 0.850
```

## Usage

### Quick Start
```bash
cd data_streamer
cargo run
```

### What Happens

**Step 1: Asset Discovery**
```
=== Bybit TradFi Data Streamer ===

=== Step 1: Identify TradFi assets ===

Fetching spot tickers...
Found 32 XUSDT tickers (tokenized stocks)
  - AAPLXUSDT
  - TSLAXUSDT
  - NVDAXUSDT
  ...

Fetching linear tickers...
Found 4 TradFi linear tickers
  - GASUSDT
  - SPXPERP
  - SPXUSDT
  - XAUTUSDT
```

**Step 2: Historical Data Download**
```
=== Step 2: Download historical data ===

=== Downloading historical data for spot ===
Downloading historical data for AAPLXUSDT...
  ✓ Downloaded 154 bars for AAPLXUSDT
Downloading historical data for TSLAXUSDT...
  ✓ Downloaded 189 bars for TSLAXUSDT
...
Historical data saved to: historical_data/spot
Markets file: historical_data/spot/MARKETS.TXT

=== Downloading historical data for linear ===
Downloading historical data for XAUTUSDT...
  ✓ Downloaded 1000 bars for XAUTUSDT
...
```

**Step 3: Real-time Streaming**
```
=== Step 3: Start real-time tick streaming ===
Press Ctrl+C to stop

Connecting to spot WebSocket...
Connected to spot!
Subscribed to 32 spot symbols
Created files for AAPLXUSDT
Created files for TSLAXUSDT
...
[spot] Subscription confirmed

Connecting to linear WebSocket...
Connected to linear!
Subscribed to 4 linear symbols
Created files for XAUTUSDT
Created files for SPXUSDT
...
[linear] Subscription confirmed

[spot] Received 100 ticks
[linear] Received 100 ticks
[spot] Received 200 ticks
...
```

### Stop Streaming
Press `Ctrl+C` to gracefully stop data collection.

## Technical Details

### WebSocket Endpoints
- **Spot**: `wss://stream.bybit.com/v5/public/spot`
- **Linear**: `wss://stream.bybit.com/v5/public/linear`

### REST API Endpoints
- **Tickers**: `https://api.bybit.com/v5/market/tickers?category={spot|linear}`
- **Klines**: `https://api.bybit.com/v5/market/kline?category={spot|linear}&symbol={SYMBOL}&interval=D&limit=1000`

### Subscription Message
```json
{
  "op": "subscribe",
  "args": [
    "publicTrade.XAUTUSDT",
    "publicTrade.AAPLXUSDT"
  ]
}
```

### Trade Message Format
```json
{
  "topic": "publicTrade.XAUTUSDT",
  "type": "snapshot",
  "data": [{
    "T": 1701234567890,
    "s": "XAUTUSDT",
    "p": "2050.50",
    "v": "0.1",
    "S": "Buy"
  }]
}
```

## OHLCV Aggregation Logic

The application constructs 1-minute bars in real-time:

1. **Bar Initialization**: First trade in a minute sets Open, High, Low, Close
2. **Bar Update**: Subsequent trades update High, Low, Close, and accumulate Volume
3. **Bar Completion**: When a new minute starts, the previous bar is written to file
4. **Timestamp**: Bars are timestamped at the start of the minute

Example:
- Trade at 12:00:15 → Opens bar for 12:00:00
- Trade at 12:00:45 → Updates bar for 12:00:00
- Trade at 12:01:05 → Completes bar for 12:00:00, opens bar for 12:01:00

## Dependencies

```toml
[dependencies]
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
chrono = "0.4"
tokio-tungstenite = "0.24"
futures-util = "0.3"
```

## Use Cases

### Backtesting
Use `historical_data/{category}/MARKETS.TXT` with your backtesting framework:
```bash
./backtest historical_data/spot/MARKETS.TXT
```

### Real-time Trading
Monitor `tick_data/` or `bar_data/` directories for live market data.

### Data Analysis
Analyze tick-level microstructure or aggregate bars for technical analysis.

### Research
Study price formation, liquidity, and market dynamics of tokenized assets.

## Limitations & Notes

- **MT5 Exclusive CFDs**: Some CFDs are only available via MetaTrader 5, not through Bybit V5 API
- **Historical Limit**: REST API provides max 1000 bars per request
- **WebSocket Stability**: Includes ping/pong handling for connection keepalive
- **Disk Space**: Tick data can grow large; monitor disk usage
- **Market Hours**: Some assets may have limited trading hours

## Future Enhancements

- [ ] Configurable bar intervals (5m, 15m, 1h, 4h, 1d)
- [ ] Database storage (PostgreSQL/TimescaleDB)
- [ ] Data compression for long-term storage
- [ ] Reconnection logic with exponential backoff
- [ ] Multi-exchange support (Binance, OKX, etc.)
- [ ] Real-time alerts and notifications
- [ ] Web dashboard for monitoring

## Troubleshooting

### No data received
- Check internet connection
- Verify symbols are actively trading
- Check Bybit API status

### Connection drops
- Normal for long-running connections
- Restart the application
- Future version will auto-reconnect

### Missing historical data
- Some symbols may have limited history
- Check symbol availability on Bybit

## License

This project is for educational and research purposes. Ensure compliance with Bybit's Terms of Service when using their API.
