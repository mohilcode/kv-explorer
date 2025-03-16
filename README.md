
# Wrangler KV Explorer

A desktop application built with Tauri to explore and manage the local KV storage used by Cloudflare's Wrangler tool.

## Features

- Select a Wrangler project folder to view its local KV namespaces and entries.
- Display KV entries with keys, values (parsed as JSON if possible), expiration, and metadata.
- Update KV entry values.
- Delete one or more KV entries.

**Note:** This app only interacts with Wrangler's local storage, not live Cloudflare KV storage.

## Prerequisites

- [Node.js](https://nodejs.org/) (v16 or later)
- [pnpm](https://pnpm.io/) (v7 or later)
- [Rust](https://www.rust-lang.org/) (v1.56 or later)
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites)

## Installation

1. Clone the repository:
   ```sh
   git clone https://github.com/yourusername/kv-explorer.git
   cd kv-explorer
   ```

2. Install dependencies:
   ```sh
   pnpm install
   ```

3. Build the app:
   ```sh
   pnpm tauri build
   ```
   The executable will be in `src-tauri/target/release`.

## Development

Run in development mode:
```sh
pnpm tauri dev
```

## Usage

1. Launch the app:
   - **Windows**: Run `kv-explorer.exe` from the release folder.
   - **macOS**: Open `kv-explorer.app`.
   - **Linux**: Execute `kv-explorer`.

2. Click "Select Folder" and choose the root folder of your Wrangler project (containing `.wrangler`).

3. View, update, or delete KV entries as needed.

## Contributing

Contributions are welcome! Please submit issues or pull requests on GitHub.