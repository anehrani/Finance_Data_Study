import csv
from datetime import datetime

def convert_csv_to_txt(csv_file_path, txt_file_path):
    """
    Convert CSV file to TXT file in the format: YYYYMMDD Open High Low Close

    Args:
        csv_file_path (str): Path to the input CSV file
        txt_file_path (str): Path to the output TXT file
    """
    with open(csv_file_path, 'r', newline='', encoding='utf-8') as csvfile, \
         open(txt_file_path, 'w', encoding='utf-8') as txtfile:

        reader = csv.reader(csvfile)
        next(reader)  # Skip header

        for row in reader:
            if len(row) < 6:
                continue  # Skip malformed rows

            local_time_str = row[0]
            open_price = row[1]
            high_price = row[2]
            low_price = row[3]
            close_price = row[4]
            # Volume is ignored as per the target format

            # Parse the date
            # Format: "DD.MM.YYYY HH:MM:SS.sss GMT+0900"
            try:
                # Remove the milliseconds and GMT part for simplicity
                # Actually, let's parse properly
                # The format has .sss which is microseconds, but strptime can handle %f
                # But GMT+0900 is +09:00
                date_obj = datetime.strptime(local_time_str, "%d.%m.%Y %H:%M:%S.%f GMT%z")
                date_str = date_obj.strftime("%Y%m%d")
            except ValueError:
                # If parsing fails, try without milliseconds
                try:
                    date_obj = datetime.strptime(local_time_str.split('.')[0] + " GMT+0900", "%d.%m.%Y %H:%M:%S GMT%z")
                    date_str = date_obj.strftime("%Y%m%d")
                except ValueError:
                    print(f"Failed to parse date: {local_time_str}")
                    continue

            # Write to txt
            txtfile.write(f"{date_str} {open_price} {high_price} {low_price} {close_price}\n")

if __name__ == "__main__":
    # Example usage
    csv_path = "data/XAGUSD_Candlestick_1_D_BID_05.05.2003-08.11.2025.csv"
    txt_path = "data/XAGUSD_converted.txt"
    convert_csv_to_txt(csv_path, txt_path)
    print(f"Conversion completed. Output saved to {txt_path}")