import { defineConfig } from '@playwright/test';
import path from 'path';

export default defineConfig({
  testDir: './tests',
  timeout: 60_000,
  expect: { timeout: 10_000 },
  use: {
    headless: true,
    baseURL: process.env.BASE_URL || 'http://127.0.0.1:8080',
  },
  webServer: {
    command: `bash -lc "set -e; cd ..; if ! command -v trunk >/dev/null; then echo 'Please install trunk: cargo install trunk' >&2; exit 1; fi; cd crates/umap-web && trunk build --release --dist dist; cd ../..; cargo run -q -p umap-cli -- serve --db data.test.db --static-dir crates/umap-web/dist --addr 127.0.0.1:8080"`,
    cwd: path.resolve(__dirname),
    port: 8080,
    reuseExistingServer: true,
    timeout: 120_000,
  },
  projects: [
    { name: 'chromium', use: { browserName: 'chromium' } },
  ],
});
