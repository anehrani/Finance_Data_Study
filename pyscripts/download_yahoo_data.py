#!/usr/bin/env python3
"""
Download historical stock data from Yahoo Finance
Reads a list of stocks from a text file and downloads their historical data
from 1995 until now in daily timeframe.

This is a more reliable alternative to investing.com scraping.
"""

import os
import sys
from datetime import datetime
from pathlib import Path
import yfinance as yf
import pandas as pd


def read_stock_list(filename):
    """
    Read stock symbols from a text file.
    
    Args:
        filename: Path to the text file containing stock symbols (one per line)
        
    Returns:
        List of stock symbols
    """
    try:
        with open(filename, 'r') as f:
            # Read lines, strip whitespace, and filter out empty lines and comments
            stocks = [line.strip() for line in f if line.strip() and not line.strip().startswith('#')]
        return stocks
    except FileNotFoundError:
        print(f"Error: File '{filename}' not found.")
        sys.exit(1)
    except Exception as e:
        print(f"Error reading file '{filename}': {e}")
        sys.exit(1)


def download_stock_data(symbol, start_date, end_date, output_dir):
    """
    Download historical data for a single stock using yfinance.
    
    Args:
        symbol: Stock symbol (Yahoo Finance format)
        start_date: Start date in format 'YYYY-MM-DD'
        end_date: End date in format 'YYYY-MM-DD'
        output_dir: Directory to save the downloaded data
        
    Returns:
        True if successful, False otherwise
    """
    try:
        print(f"Downloading {symbol}...")
        
        # Create ticker object
        ticker = yf.Ticker(symbol)
        
        # Download historical data
        df = ticker.history(start=start_date, end=end_date, interval='1d')
        
        if df.empty:
            print(f"  ✗ No data available for {symbol}")
            return False
        
        # Create output directory if it doesn't exist
        Path(output_dir).mkdir(parents=True, exist_ok=True)
        
        # Save to CSV file
        output_file = os.path.join(output_dir, f"{symbol}.csv")
        df.to_csv(output_file)
        
        print(f"  ✓ Successfully downloaded {len(df)} rows to {output_file}")
        print(f"    Date range: {df.index[0].strftime('%Y-%m-%d')} to {df.index[-1].strftime('%Y-%m-%d')}")
        return True
        
    except Exception as e:
        print(f"  ✗ Error downloading {symbol}: {e}")
        return False


def main():
    """Main function to orchestrate the download process."""
    
    # Configuration
    input_file = "stocks_list.txt"  # File containing list of stocks
    output_dir = "data/historical_data"   # Directory to save downloaded data
    start_date = "1995-01-01"        # Start date
    end_date = datetime.now().strftime("%Y-%m-%d")  # Today's date
    
    # Check if input file exists
    if not os.path.exists(input_file):
        print(f"Creating example '{input_file}' file...")
        with open(input_file, 'w') as f:
            f.write("# Stock list format (Yahoo Finance symbols):\n")
            f.write("# One symbol per line. Lines starting with # are comments.\n")
            f.write("# \n")
            f.write("# US Stocks:\n")
            f.write("AAPL\n")
            f.write("MSFT\n")
            f.write("GOOGL\n")
            f.write("AMZN\n")
            f.write("TSLA\n")
            f.write("NVDA\n")
            f.write("META\n")
            f.write("JPM\n")
            f.write("V\n")
            f.write("WMT\n")
            f.write("# \n")
            f.write("# International stocks (use Yahoo Finance symbols):\n")
            f.write("# 7203.T      # Toyota (Tokyo)\n")
            f.write("# SAP         # SAP (NYSE)\n")
            f.write("# HSBA.L      # HSBC (London)\n")
            f.write("# NESN.SW     # Nestle (Switzerland)\n")
            f.write("# 005930.KS   # Samsung (Korea)\n")
        print(f"Please edit '{input_file}' and add your stock symbols, then run this script again.")
        sys.exit(0)
    
    # Read stock list
    print(f"Reading stock list from '{input_file}'...")
    stock_symbols = read_stock_list(input_file)
    
    if not stock_symbols:
        print("No stocks found in the input file.")
        sys.exit(1)
    
    print(f"Found {len(stock_symbols)} stock(s) to download.")
    print(f"Date range: {start_date} to {end_date}")
    print(f"Output directory: {output_dir}")
    print("-" * 60)
    
    # Download data for each stock
    successful = 0
    failed = 0
    
    for symbol in stock_symbols:
        if download_stock_data(symbol, start_date, end_date, output_dir):
            successful += 1
        else:
            failed += 1
        print()  # Empty line for readability
    
    # Summary
    print("-" * 60)
    print(f"Download complete!")
    print(f"  Successful: {successful}")
    print(f"  Failed: {failed}")
    print(f"  Total: {len(stock_symbols)}")
    print(f"\nData saved to: {output_dir}/")


if __name__ == "__main__":
    main()
