// Known tokenized stocks, indices, commodities, and metals
// This list excludes crypto tokens that happen to end in XUSDT

use std::collections::HashSet;

pub fn get_tradfi_symbols() -> HashSet<&'static str> {
    let mut symbols = HashSet::new();
    
    // Tokenized US Stocks
    symbols.insert("AAPLXUSDT");   // Apple
    symbols.insert("TSLAXUSDT");   // Tesla
    symbols.insert("NVDAXUSDT");   // Nvidia
    symbols.insert("GOOGLXUSDT");  // Google
    symbols.insert("METAXUSDT");   // Meta (Facebook)
    symbols.insert("AMZNXUSDT");   // Amazon
    symbols.insert("MSFTXUSDT");   // Microsoft
    symbols.insert("COINXUSDT");   // Coinbase
    symbols.insert("HOODXUSDT");   // Robinhood
    symbols.insert("MCDXUSDT");    // McDonald's
    
    // Indices
    symbols.insert("SPXUSDT");     // S&P 500
    symbols.insert("SPXPERP");     // S&P 500 Perpetual
    
    // Commodities
    symbols.insert("GASUSDT");     // Natural Gas
    symbols.insert("OILUSDT");     // Oil (if available)
    
    // Metals
    symbols.insert("XAUTUSDT");    // Gold
    symbols.insert("XAGUSDT");     // Silver (if available)
    
    // Add more as needed - these are the main TradFi assets
    // Exclude crypto tokens like: TRXUSDT, AVAXUSDT, ICXUSDT, STXUSDT, etc.
    
    symbols
}

pub fn is_tradfi_symbol(symbol: &str) -> bool {
    let tradfi = get_tradfi_symbols();
    tradfi.contains(symbol)
}

pub fn is_likely_stock(symbol: &str) -> bool {
    // Additional heuristic: real stocks typically have 2-5 letter ticker + XUSDT
    // Crypto tokens often have longer names or specific patterns
    if !symbol.ends_with("XUSDT") {
        return false;
    }
    
    let base = symbol.trim_end_matches("XUSDT");
    
    // Known stock patterns
    let known_stocks = get_tradfi_symbols();
    if known_stocks.contains(symbol) {
        return true;
    }
    
    // Exclude known crypto patterns
    let crypto_patterns = [
        "TRX", "AVAX", "ICX", "STX", "DYD", "GMX", "ZRX", 
        "ZTX", "ZEX", "APE", "SNX", "IMX", "NAV", "WEMI",
        "MBOX", "MBO", "MPL", "CRCL", "EL", "HT", "MB"
    ];
    
    for pattern in &crypto_patterns {
        if base.contains(pattern) || base == *pattern {
            return false;
        }
    }
    
    // If it's 2-5 characters and not in crypto list, might be a stock
    // But to be safe, only include explicitly known symbols
    false
}
