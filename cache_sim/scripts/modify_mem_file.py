import sys
from pathlib import Path

def stream_lines(path: Path):
    """
    Generator that yields each line stripped of its trailing newline.
    Uses the built-in buffered iteration provided by Python’s file object,
    so it stays memory-efficient even for multi-gigabyte files.
    """
    with path.open('rt', encoding='utf-8', errors='replace') as f:
        for line in f:
            yield line.rstrip('\n')

def main() -> None:
    if len(sys.argv) != 2:
        print(__doc__, file=sys.stderr)
        sys.exit(1)

    file_path = Path(sys.argv[1])
    if not file_path.is_file():
        print(f"Error: {file_path} is not a file", file=sys.stderr)
        sys.exit(1)


    NEW_FILE = "mem_files/big_minecraft_log2.txt"

    with open(NEW_FILE, "w", encoding="utf-8") as dst:
        for i, line in enumerate(stream_lines(file_path), 1):
            parts = line.split()
            if not parts:
                continue

            op = parts[0]
            if op == "r":
                dst.write(f"{line}\n")
            elif op == "w":
                dst.write(f"{line} 1 69\n")
            else:
                print(f"BAD LINE (ignored): {line}", file=sys.stderr)

            line_count = i
            if i & 0xF_FFFF == 0:    # progress indicator every 100k
                print(f"…processed {i} lines", file=sys.stderr)

    print(f"Finished – processed {line_count} lines, output → {NEW_FILE}")

if __name__ == "__main__":
    main()
