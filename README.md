# Wrangler KV Explorer

A desktop application built with Tauri to explore and manage both local and remote Cloudflare KV storage used by Wrangler.

## Features

- Select a Wrangler project folder to view its local KV namespaces and entries.
- Connect to your Cloudflare account using Account ID and API Token to access remote KV namespaces.
- View all KV namespaces in your Cloudflare account.
- Display KV entries with keys, values (parsed as JSON if possible), expiration, and metadata.
- Update KV entry values for both local and remote storage.
- Delete KV entries.
- Comprehensive management of both local and remote KV storage in one interface.

## Prerequisites

- [Node.js](https://nodejs.org/) (v16 or later)
- [pnpm](https://pnpm.io/) (v7 or later)
- [Rust](https://www.rust-lang.org/) (v1.56 or later)
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites)

## Installation

### Pre-built Releases

Ready-to-use release builds are available for download from the [Releases](https://github.com/mohilcode/kv-explorer/releases) page.

### Building from Source

1. Clone the repository:
   ```sh
   git clone https://github.com/mohilcode/kv-explorer.git
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

2. For local KV storage:
   - Click "Select Folder" and choose the root folder of your Wrangler project (containing `.wrangler`).

3. For remote Cloudflare KV storage:
   - Enter your Cloudflare Account ID and API Token.
   - Click "Connect" to access your remote KV namespaces.

4. View, update, or delete KV entries as needed for both local and remote storage.

## Contributing

Contributions are welcome! Please submit issues or pull requests on GitHub.