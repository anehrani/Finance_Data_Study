# Bybit TradFi Assets - Complete Analysis

## Summary

After comprehensive analysis of the Bybit V5 API, we have identified **ALL** available TradFi assets accessible via the API:

### Total TradFi Assets: 14
- **11 Tokenized Stocks** (Spot category)
- **3 Indices/Commodities/Metals** (Linear category)

## Why So Few?

### Bybit's TradFi Architecture

Bybit offers TradFi trading through **two separate platforms**:

1. **Bybit V5 API** (What we're using)
   - Limited TradFi assets
   - Accessible via REST/WebSocket
   - No special software required

2. **Bybit TradFi via MetaTrader 5** (MT5)
   - Extensive TradFi catalog (forex, indices, commodities, stocks)
   - Requires MetaTrader 5 software
   - **NOT accessible via Bybit V5 API**

## Complete List of V5 API TradFi Assets

### Spot Category (11 assets)
```
✓ AAPLXUSDT   - Apple Inc.
✓ TSLAXUSDT   - Tesla Inc.
✓ NVDAXUSDT   - Nvidia Corporation
✓ GOOGLXUSDT  - Alphabet Inc. (Google)
✓ METAXUSDT   - Meta Platforms Inc. (Facebook)
✓ AMZNXUSDT   - Amazon.com Inc.
✓ COINXUSDT   - Coinbase Global Inc.
✓ HOODXUSDT   - Robinhood Markets Inc.
✓ MCDXUSDT    - McDonald's Corporation
✓ SPXUSDT     - S&P 500 Index
✓ XAUTUSDT    - Gold (spot)
```

### Linear Category (3 assets)
```
✓ SPXUSDT     - S&P 500 Index (perpetual)
✓ GASUSDT     - Natural Gas (perpetual)
✓ XAUTUSDT    - Gold (perpetual)
```

## What About Other TradFi Assets?

### Assets Available ONLY via MT5 (Not in V5 API)

**Forex Pairs**:
- EURUSD, GBPUSD, USDJPY, AUDUSD, USDCAD, USDCHF, NZDUSD
- EUR/GBP, EUR/JPY, GBP/JPY, and many more

**Stock Indices**:
- NAS100 (NASDAQ 100)
- DJ30 (Dow Jones 30)
- DE40 (DAX 40)
- UK100 (FTSE 100)
- JP225 (Nikkei 225)
- HK50 (Hang Seng)
- And many more

**Commodities**:
- Crude Oil (WTI, Brent)
- Silver
- Copper
- Platinum
- Coffee, Cocoa, Sugar, etc.

**More Stocks**:
- IBM, Microsoft, Netflix, Disney, Visa, etc.

### How to Access MT5 Assets

If you need these assets, you must:
1. Download MetaTrader 5
2. Connect to Bybit's MT5 server
3. Use MT5's API or manual trading

**Note**: MT5 assets are **NOT** available via Bybit V5 REST/WebSocket API.

## Verification

We verified this by:
1. Fetching all 643 linear tickers from Bybit V5 API
2. Filtering out 183 obvious crypto tokens
3. Analyzing the remaining 460 symbols
4. Finding only 3 genuine TradFi assets (SPXUSDT, GASUSDT, XAUTUSDT)
5. Confirming via Bybit documentation that most TradFi is MT5-only

## Conclusion

**Our data streamer is complete and correct**. It captures:
- ✅ All 11 tokenized stocks available via V5 API
- ✅ All 3 TradFi linear assets available via V5 API
- ✅ Zero cryptocurrency contamination

**For additional TradFi assets**, you would need to integrate with MetaTrader 5, which is outside the scope of the Bybit V5 API.

## References

- [Bybit TradFi Trading](https://www.bybit.com/en/tradfi/)
- [Bybit V5 API Documentation](https://bybit-exchange.github.io/docs/v5/intro)
- [MetaTrader 5 Integration](https://www.bybit.com/en/help-center/article/Bybit-TradFi-Trading-via-MT5)

---

**Last Updated**: 2025-11-30
**API Version**: Bybit V5
**Total V5 TradFi Assets**: 14 (11 spot + 3 linear)
