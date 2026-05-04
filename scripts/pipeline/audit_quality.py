import csv
import math
import sys
import os

def analyze_file(filepath):
    print(f"=== HCSN Data Quality Audit: {os.path.basename(filepath)} ===")
    
    total = 0
    nan_inf = 0
    valid_rows = []
    
    try:
        with open(filepath, 'r') as f:
            reader = csv.DictReader(f)
            for row in reader:
                total += 1
                bad = False
                for val in row.values():
                    v = val.strip().lower()
                    if v in ('nan', 'inf', '-inf'):
                        bad = True
                        break
                if bad:
                    nan_inf += 1
                else:
                    valid_rows.append(row)
    except Exception as e:
        print(f"Error reading file: {e}")
        return

    print(f"  Total Lines:    {total}")
    print(f"  Corrupted:      {nan_inf} ({nan_inf/max(total,1)*100:.2f}%)")
    print(f"  Valid:          {len(valid_rows)} ({len(valid_rows)/max(total,1)*100:.2f}%)")
    
    if len(valid_rows) > 0:
        print(f"  Status:         {'SUCCESS' if nan_inf == 0 else 'WARNING'}")
    else:
        print(f"  Status:         FAILED (No valid data)")

if __name__ == "__main__":
    if len(sys.argv) > 1:
        analyze_file(sys.argv[1])
    else:
        print("Usage: python3 audit_quality.py <csv_filepath>")
