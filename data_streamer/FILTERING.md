# TradFi Asset Filtering

## Overview
This document explains how the data_streamer filters out cryptocurrency tokens and only includes genuine TradFi (Traditional Finance) assets.

## Filtering Strategy

### Spot Category (Tokenized Stocks)
**Filter**: Explicit whitelist of known tokenized stocks

**Included (11 assets)**:
- AAPLXUSDT - Apple
- TSLAXUSDT - Tesla  
- NVDAXUSDT - Nvidia
- GOOGLXUSDT - Google
- METAXUSDT - Meta (Facebook)
- AMZNXUSDT - Amazon
- MSFTXUSDT - Microsoft (if available)
- COINXUSDT - Coinbase
- HOODXUSDT - Robinhood
- MCDXUSDT - McDonald's
- SPXUSDT - S&P 500 (also in spot)
- XAUTUSDT - Gold (also in spot)

**Excluded (21 crypto tokens that end in XUSDT)**:
- TRXUSDT - Tron (crypto)
- AVAXUSDT - Avalanche (crypto)
- ICXUSDT - ICON (crypto)
- STXUSDT - Stacks (crypto)
- DYDXUSDT - dYdX (crypto)
- GMXUSDT - GMX (crypto)
- ZRXUSDT - 0x (crypto)
- ZTXUSDT - ZTX (crypto)
- ZEXUSDT - ZEX (crypto)
- APEXUSDT - ApeX (crypto)
- SNXUSDT - Synthetix (crypto)
- IMXUSDT - Immutable X (crypto)
- NAVXUSDT - Navcoin (crypto)
- WEMIXUSDT - Wemix (crypto)
- MBOXUSDT - MOBOX (crypto)
- MBXUSDT - MBX (crypto)
- MPLXUSDT - MPL (crypto)
- CRCLXUSDT - CRCL (crypto)
- ELXUSDT - EL (crypto)
- HTXUSDT - HT (crypto)
- MXUSDT - MX (crypto)

### Linear Category (Indices/Commodities/Metals)
**Filter**: Pattern-based with explicit exclusions

**Included (3 assets)**:
- GASUSDT - Natural Gas
- SPXUSDT - S&P 500 Index
- XAUTUSDT - Gold

**Excluded**:
- BANANAS31USDT - Meme token
- SPXPERP - Perpetual contract (can be included if needed)
- XAGUSDT - Silver (not currently available)
- OILUSDT - Oil (not currently available)

## Why This Matters

### Problem
Bybit uses the "XUSDT" suffix for both:
1. Tokenized stocks (e.g., AAPLXUSDT = Apple stock)
2. Cryptocurrency tokens (e.g., TRXUSDT = Tron crypto)

Without filtering, you would download 32 symbols, but only 11 are actual TradFi assets.

### Solution
We use an explicit whitelist approach:
- **Spot**: Only symbols in `tradfi_filter::get_tradfi_symbols()`
- **Linear**: Pattern matching with exclusions for meme tokens

## Updating the Filter

### To Add New Tokenized Stocks
Edit `src/tradfi_filter.rs` and add to the HashSet:

```rust
symbols.insert("IBMXUSDT");    // IBM
symbols.insert("NFLXXUSDT");   // Netflix
```

### To Include Perpetuals
In `src/main.rs`, remove the PERP exclusion:

```rust
// Change this line:
!s.contains("PERP") // Exclude perpetuals

// To:
true // Include perpetuals
```

### To Add New Commodities/Indices
The linear filter uses pattern matching. New assets with these patterns will be auto-included:
- XAU*, XAG* (metals)
- *GAS*, *OIL* (energy)
- SPX*, NAS100*, DJI* (indices)

## Verification

Run the application and check Step 1 output:

```bash
cargo run
```

Expected output:
```
=== Step 1: Identify TradFi assets ===

Fetching spot tickers...
Found 11 tokenized stock tickers (TradFi only)
  - AAPLXUSDT
  - TSLAXUSDT
  - NVDAXUSDT
  - GOOGLXUSDT
  - METAXUSDT
  - AMZNXUSDT
  - COINXUSDT
  - HOODXUSDT
  - MCDXUSDT
  - SPXUSDT
  - XAUTUSDT

Fetching linear tickers...
Found 3 TradFi linear tickers (indices/commodities/metals)
  - GASUSDT
  - SPXUSDT
  - XAUTUSDT
```

## Data Quality Impact

**Before filtering**: 32 spot + 5 linear = 37 symbols (21 crypto, 16 TradFi)
**After filtering**: 11 spot + 3 linear = 14 symbols (0 crypto, 14 TradFi)

This ensures:
- ✅ Only genuine TradFi assets are downloaded
- ✅ No cryptocurrency contamination
- ✅ Clean datasets for traditional finance analysis
- ✅ Reduced storage and bandwidth usage
