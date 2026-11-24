
from pathlib import Path
import tkinter as tk
import platform

os_name = platform.system()
print(f"Operating System: {os_name}")

ROOTS = [
    Path(r"D:\VS Code Projects\snipster\src"),
    Path(r"D:\VS Code Projects\snipster\src\bin"),
    Path(r"D:\VS Code Projects\snipster\Cargo.toml"),
]

SKIP_DIRS = {
    "node_modules",
    ".next",
    ".git",
    "dist",
    "build",
}

# File extensions we want to include
ALLOWED_EXTENSIONS = {
    ".ts",
    ".tsx",
    ".js",
    ".jsx",
    ".json",
    ".css",
    ".scss",
    ".html",
    ".md",
    ".rs",
    ".toml"
}

files = []

for root_path in ROOTS:
    if not root_path.exists():
        print(f"WARNING: Path does not exist: {root_path}")
        continue

    # If the root itself is a file, include it directly
    if root_path.is_file():
        if root_path.suffix.lower() in ALLOWED_EXTENSIONS:
            files.append(root_path)
        continue

    # If the root is a directory, recursively scan it
    for path in root_path.rglob("*"):
        if not path.is_file():
            continue

        # Skip files inside unwanted directories
        if any(part in SKIP_DIRS for part in path.parts):
            continue

        # Only include known source/text files
        if path.suffix.lower() not in ALLOWED_EXTENSIONS:
            continue

        files.append(path)

# Remove duplicates and sort files consistently
files = sorted(set(files), key=lambda p: str(p).lower())

output = []

for file in files:
    try:
        content = file.read_text(encoding="utf-8")

        output.append(
            f"\n{'=' * 80}\n"
            f"FILE: {file}\n"
            f"{'=' * 80}\n\n"
            f"{content}\n"
        )

        # Without file names (if needed separately)
        # output.append(content)

    except UnicodeDecodeError:
        print(f"Skipped non-UTF8 file: {file}")
    except Exception as e:
        print(f"Error reading {file}: {e}")

final_text = "\n".join(output)

# Copy to Windows clipboard
root = tk.Tk()
root.withdraw()

root.clipboard_clear()
root.clipboard_append(final_text)
root.update()

print(f"Copied {len(files)} files to clipboard.")
print(f"Total characters: {len(final_text):,}")
print("\nFiles:")

for file in files:
    print(f"  - {file}")

# Keep Tk alive briefly so clipboard ownership is safely established
root.after(1000, root.destroy)
root.mainloop()

print("\nDone. Everything is now in your clipboard.")
