import pandas as pd
import os
import glob

def convert_csv_to_txt():
    # Define input and output paths
    input_dir = 'pyscripts/data/historical_data'
    output_dir = 'pyscripts/historical_data'
    
    # Create output directory if it doesn't exist
    os.makedirs(output_dir, exist_ok=True)
    
    # Check if input directory exists
    if not os.path.exists(input_dir):
        print(f"Error: Directory {input_dir} not found.")
        return

    # Find all CSV files
    csv_files = glob.glob(os.path.join(input_dir, '*.csv'))
    
    if not csv_files:
        print("No CSV files found in the directory.")
        return

    print(f"Found {len(csv_files)} files. Converting...")

    for file_path in csv_files:
        # Extract asset name from filename (e.g., 'AAPL.csv' -> 'AAPL')
        filename = os.path.basename(file_path)
        asset_name = os.path.splitext(filename)[0]
        
        try:
            # Read CSV
            df = pd.read_csv(file_path)
            
            # Check if required columns exist
            required_cols = ['Date', 'Open', 'High', 'Low', 'Close']
            if not all(col in df.columns for col in required_cols):
                print(f"Warning: Missing required columns in {filename}. Skipping.")
                continue
            
            # Parse date and format as YYYYMMDD
            # Use utc=True to handle mixed timezones
            df['Date'] = pd.to_datetime(df['Date'], utc=True, errors='coerce')
            
            # Drop rows where date parsing failed
            df = df.dropna(subset=['Date'])
            
            if len(df) == 0:
                print(f"Warning: No valid dates in {filename}. Skipping.")
                continue
            
            df['DateFormatted'] = df['Date'].dt.strftime('%Y%m%d')
            
            # Select and order columns: Date Open High Low Close
            output_df = df[['DateFormatted', 'Open', 'High', 'Low', 'Close']].copy()
            
            # Drop rows with NaN values
            output_df.dropna(inplace=True)
            
            # Define output file path
            output_file = os.path.join(output_dir, f"{asset_name}.txt")
            
            # Write to text file (space-separated, no header, no index)
            output_df.to_csv(
                output_file,
                sep=' ',
                header=False,
                index=False,
                float_format='%.6f'
            )
            
            print(f"✓ Converted {filename} -> {asset_name}.txt ({len(output_df)} rows)")
        
        except Exception as e:
            print(f"✗ Error processing {filename}: {e}")

    print(f"\nConversion complete! Files saved to: {output_dir}")

if __name__ == "__main__":
    convert_csv_to_txt()
