#!/usr/bin/env python3
"""
Download historical stock data from investing.com
Reads a list of stocks from a text file and downloads their historical data
from 1995 until now in daily timeframe.
"""

import os
import sys
from datetime import datetime
from pathlib import Path
import investpy


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


def download_stock_data(symbol, country, start_date, end_date, output_dir):
    """
    Download historical data for a single stock.
    
    Args:
        symbol: Stock symbol
        country: Country code (e.g., 'united states', 'japan', 'germany')
        start_date: Start date in format 'DD/MM/YYYY'
        end_date: End date in format 'DD/MM/YYYY'
        output_dir: Directory to save the downloaded data
        
    Returns:
        True if successful, False otherwise
    """
    try:
        print(f"Downloading {symbol} ({country})...")
        
        # Download historical data
        df = investpy.get_stock_historical_data(
            stock=symbol,
            country=country,
            from_date=start_date,
            to_date=end_date
        )
        
        # Create output directory if it doesn't exist
        Path(output_dir).mkdir(parents=True, exist_ok=True)
        
        # Save to CSV file
        output_file = os.path.join(output_dir, f"{symbol}_{country}.csv")
        df.to_csv(output_file)
        
        print(f"  ✓ Successfully downloaded {len(df)} rows to {output_file}")
        return True
        
    except Exception as e:
        print(f"  ✗ Error downloading {symbol} ({country}): {e}")
        return False


def main():
    """Main function to orchestrate the download process."""
    
    # Configuration
    input_file = "stocks_list.txt"  # File containing list of stocks
    output_dir = "data/historical_data"   # Directory to save downloaded data
    start_date = "01/01/1995"        # Start date
    end_date = datetime.now().strftime("%d/%m/%Y")  # Today's date
    default_country = "united states"  # Default country
    
    # Check if input file exists
    if not os.path.exists(input_file):
        print(f"Creating example '{input_file}' file...")
        with open(input_file, 'w') as f:
            f.write("# Stock list format:\n")
            f.write("# symbol,country (country is optional, defaults to 'united states')\n")
            f.write("# Examples:\n")
            f.write("AAPL\n")
            f.write("MSFT\n")
            f.write("GOOGL\n")
            f.write("TSLA\n")
            f.write("# With country specified:\n")
            f.write("# TOYOTA,japan\n")
            f.write("# SAP,germany\n")
        print(f"Please edit '{input_file}' and add your stock symbols, then run this script again.")
        sys.exit(0)
    
    # Read stock list
    print(f"Reading stock list from '{input_file}'...")
    stock_entries = read_stock_list(input_file)
    
    if not stock_entries:
        print("No stocks found in the input file.")
        sys.exit(1)
    
    print(f"Found {len(stock_entries)} stock(s) to download.")
    print(f"Date range: {start_date} to {end_date}")
    print(f"Output directory: {output_dir}")
    print("-" * 60)
    
    # Download data for each stock
    successful = 0
    failed = 0
    
    for entry in stock_entries:
        # Parse entry (format: "SYMBOL" or "SYMBOL,country")
        parts = [p.strip() for p in entry.split(',')]
        symbol = parts[0]
        country = parts[1].lower() if len(parts) > 1 else default_country
        
        if download_stock_data(symbol, country, start_date, end_date, output_dir):
            successful += 1
        else:
            failed += 1
    
    # Summary
    print("-" * 60)
    print(f"Download complete!")
    print(f"  Successful: {successful}")
    print(f"  Failed: {failed}")
    print(f"  Total: {len(stock_entries)}")
    print(f"\nData saved to: {output_dir}/")


if __name__ == "__main__":
    main()
