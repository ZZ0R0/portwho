from pathlib import Path
import argparse


def display_files(directory: Path, extension: str) -> None:
    extension = extension.lstrip(".")

    for file in sorted(directory.rglob(f"*.{extension}")):
        if not file.is_file():
            continue

        try:
            content = file.read_text(encoding="utf-8")

            print(f"\n{'=' * 80}")
            print(f"FILE: {file.resolve()}")
            print(f"{'=' * 80}")
            print(content)

        except (OSError, UnicodeError) as error:
            print(f"ERROR: {file}: {error}")


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Display the content of files matching an extension."
    )
    parser.add_argument("directory", type=Path, help="Directory to scan")
    parser.add_argument("extension", help="File extension (e.g. rs, py, txt)")

    args = parser.parse_args()

    if not args.directory.is_dir():
        parser.error(f"Not a directory: {args.directory}")

    display_files(args.directory, args.extension)


if __name__ == "__main__":
    main()