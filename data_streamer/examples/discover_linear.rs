use data_streamer::bybit::BybitClient;
use reqwest::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = BybitClient::new();

    println!("=== Fetching ALL Linear Tickers ===\n");
    
    match client.get_tickers("linear").await {
        Ok(tickers) => {
            println!("Total linear tickers: {}\n", tickers.len());
            
            // Filter for potential TradFi (non-crypto) assets
            // We'll be very inclusive to see everything
            let mut potential_tradfi: Vec<String> = tickers
                .iter()
                .filter(|t| {
                    let s = &t.symbol;
                    // Exclude obvious crypto patterns
                    !s.contains("BTC") && !s.contains("ETH") && !s.contains("SOL") &&
                    !s.contains("DOGE") && !s.contains("SHIB") && !s.contains("PEPE") &&
                    !s.contains("ADA") && !s.contains("XRP") && !s.contains("DOT") &&
                    !s.contains("MATIC") && !s.contains("LINK") && !s.contains("UNI") &&
                    !s.contains("ATOM") && !s.contains("NEAR") && !s.contains("APT") &&
                    !s.contains("ARB") && !s.contains("OP") && !s.contains("AVAX") &&
                    !s.contains("FTM") && !s.contains("ALGO") && !s.contains("VET") &&
                    !s.contains("ICP") && !s.contains("FIL") && !s.contains("HBAR") &&
                    !s.contains("LTC") && !s.contains("BCH") && !s.contains("ETC") &&
                    !s.contains("XLM") && !s.contains("TRX") && !s.contains("LUNA") &&
                    !s.contains("LUNC") && !s.contains("USTC") && !s.contains("CAKE") &&
                    !s.contains("SUSHI") && !s.contains("AAVE") && !s.contains("CRV") &&
                    !s.contains("MKR") && !s.contains("SNX") && !s.contains("COMP") &&
                    !s.contains("YFI") && !s.contains("1INCH") && !s.contains("BAL") &&
                    !s.contains("RUNE") && !s.contains("KAVA") && !s.contains("BAND") &&
                    !s.contains("OCEAN") && !s.contains("SAND") && !s.contains("MANA") &&
                    !s.contains("AXS") && !s.contains("GALA") && !s.contains("ENJ") &&
                    !s.contains("CHZ") && !s.contains("FLOW") && !s.contains("THETA") &&
                    !s.contains("EGLD") && !s.contains("XTZ") && !s.contains("EOS") &&
                    !s.contains("NEO") && !s.contains("WAVES") && !s.contains("ZEC") &&
                    !s.contains("DASH") && !s.contains("XMR") && !s.contains("QTUM") &&
                    !s.contains("ONT") && !s.contains("ZIL") && !s.contains("ICX") &&
                    !s.contains("IOTA") && !s.contains("NANO") && !s.contains("RVN") &&
                    !s.contains("DGB") && !s.contains("SC") && !s.contains("ZEN") &&
                    !s.contains("PERP") && // Exclude perpetual contracts for now
                    !s.contains("1000") && // Exclude micro contracts
                    !s.contains("10000") && // Exclude micro contracts
                    !s.contains("1000000") // Exclude micro contracts
                })
                .map(|t| t.symbol.clone())
                .collect();
            
            potential_tradfi.sort();
            
            println!("Potential TradFi Linear Tickers (after basic crypto filter): {}\n", potential_tradfi.len());
            
            // Categorize them
            let mut indices = Vec::new();
            let mut commodities = Vec::new();
            let mut metals = Vec::new();
            let mut forex = Vec::new();
            let mut stocks = Vec::new();
            let mut other = Vec::new();
            
            for symbol in &potential_tradfi {
                if symbol.contains("SPX") || symbol.contains("NAS") || symbol.contains("DJI") || 
                   symbol.contains("NDX") || symbol.contains("DAX") || symbol.contains("FTSE") ||
                   symbol.contains("NIKKEI") || symbol.contains("HSI") || symbol.contains("ASX") {
                    indices.push(symbol);
                } else if symbol.contains("OIL") || symbol.contains("GAS") || symbol.contains("BRENT") ||
                          symbol.contains("WTI") || symbol.contains("NGAS") {
                    commodities.push(symbol);
                } else if symbol.contains("XAU") || symbol.contains("XAG") || symbol.contains("GOLD") ||
                          symbol.contains("SILVER") || symbol.contains("COPPER") || symbol.contains("PLATINUM") {
                    metals.push(symbol);
                } else if symbol.contains("EUR") || symbol.contains("GBP") || symbol.contains("JPY") ||
                          symbol.contains("AUD") || symbol.contains("CAD") || symbol.contains("CHF") ||
                          symbol.contains("NZD") {
                    forex.push(symbol);
                } else if symbol.len() < 15 && !symbol.contains("USDT") {
                    // Might be a stock ticker
                    stocks.push(symbol);
                } else {
                    other.push(symbol);
                }
            }
            
            if !indices.is_empty() {
                println!("📊 INDICES ({}):", indices.len());
                for s in &indices {
                    println!("  - {}", s);
                }
                println!();
            }
            
            if !commodities.is_empty() {
                println!("⛽ COMMODITIES ({}):", commodities.len());
                for s in &commodities {
                    println!("  - {}", s);
                }
                println!();
            }
            
            if !metals.is_empty() {
                println!("🥇 METALS ({}):", metals.len());
                for s in &metals {
                    println!("  - {}", s);
                }
                println!();
            }
            
            if !forex.is_empty() {
                println!("💱 FOREX ({}):", forex.len());
                for s in &forex {
                    println!("  - {}", s);
                }
                println!();
            }
            
            if !stocks.is_empty() {
                println!("📈 POSSIBLE STOCKS ({}):", stocks.len());
                for s in &stocks {
                    println!("  - {}", s);
                }
                println!();
            }
            
            if !other.is_empty() {
                println!("❓ OTHER ({}):", other.len());
                for s in &other {
                    println!("  - {}", s);
                }
                println!();
            }
            
            println!("\n=== SUMMARY ===");
            println!("Total potential TradFi: {}", potential_tradfi.len());
            println!("  Indices: {}", indices.len());
            println!("  Commodities: {}", commodities.len());
            println!("  Metals: {}", metals.len());
            println!("  Forex: {}", forex.len());
            println!("  Possible Stocks: {}", stocks.len());
            println!("  Other: {}", other.len());
        }
        Err(e) => {
            eprintln!("Error fetching linear tickers: {}", e);
        }
    }

    Ok(())
}
