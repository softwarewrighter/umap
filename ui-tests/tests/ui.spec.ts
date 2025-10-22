import { test, expect } from '@playwright/test';

const BASE_URL = process.env.BASE_URL || 'http://127.0.0.1:8080';

test.describe('UMAP Visualizer UI', () => {
  test('loads homepage and shows controls', async ({ page }) => {
    await page.goto(BASE_URL);
    await expect(page).toHaveTitle(/UMAP Visualizer/);
    await expect(page.getByPlaceholder('Search query...')).toBeVisible();
    await expect(page.getByRole('button', { name: /Search/ })).toBeVisible();
  });

  test('ingests a sample and renders a plot', async ({ page }) => {
    await page.goto(BASE_URL);

    // Upload sample text
    const fileInput = page.locator('input[type="file"]');
    await expect(fileInput).toBeVisible();
    await fileInput.setInputFiles(require('path').resolve(__dirname, '../../samples/sample.txt'));
    await page.getByRole('button', { name: /Ingest Files/ }).click();

    // Wait for server to respond and UI to update status text
    await expect(page.locator('text=Ingested')).toBeVisible({ timeout: 10_000 });

    // Run a search and wait for a plot to appear
    await page.getByPlaceholder('Search query...').fill('whale ship sea');
    await page.getByRole('button', { name: /Search/ }).click();

    // Plotly renders into a div with class .js-plotly-plot
    await expect(page.locator('.js-plotly-plot')).toBeVisible({ timeout: 20_000 });
  });
});

