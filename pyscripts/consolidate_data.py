import pandas as pd
import os
import glob

def consolidate_data():
    # Define input and output paths
    input_dir = 'pyscripts/data/historical_data'
    output_file = 'pyscripts/data/consolidated_close_prices.csv'
    
    # Check if directory exists
    if not os.path.exists(input_dir):
        print(f"Error: Directory {input_dir} not found.")
        return

    # Find all CSV files
    csv_files = glob.glob(os.path.join(input_dir, '*.csv'))
    
    if not csv_files:
        print("No CSV files found in the directory.")
        return

    print(f"Found {len(csv_files)} files. Processing...")

    all_data = []

    for file_path in csv_files:
        # Extract asset name from filename (e.g., 'AAPL.csv' -> 'AAPL')
        filename = os.path.basename(file_path)
        asset_name = os.path.splitext(filename)[0]
        
        try:
            # Read CSV
            # We assume the date is in the first column or named 'Date'
            df = pd.read_csv(file_path)
            
            # Ensure Date column is datetime and set as index
            if 'Date' in df.columns:
                df['Date'] = pd.to_datetime(df['Date'], utc=True).dt.date
                df.set_index('Date', inplace=True)
            else:
                print(f"Warning: 'Date' column not found in {filename}. Skipping.")
                continue

            # Extract Close price and rename series
            if 'Close' in df.columns:
                close_series = df['Close'].rename(asset_name)
                all_data.append(close_series)
            else:
                print(f"Warning: 'Close' column not found in {filename}. Skipping.")
        
        except Exception as e:
            print(f"Error processing {filename}: {e}")

    if all_data:
        # Concatenate all series into a single DataFrame
        # outer join ensures we keep all dates, filling missing values with NaN
        consolidated_df = pd.concat(all_data, axis=1, join='outer')
        
        # Sort by date
        consolidated_df.sort_index(inplace=True)
        
        # Save to CSV
        consolidated_df.to_csv(output_file)
        print(f"Successfully created consolidated data at: {output_file}")
        print(f"Shape: {consolidated_df.shape}")
        print("First few rows:")
        print(consolidated_df.head())
    else:
        print("No valid data found to consolidate.")

if __name__ == "__main__":
    consolidate_data()
