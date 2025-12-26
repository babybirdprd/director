import { test, expect } from '@playwright/test';
import fs from 'fs';
import path from 'path';

// Usage:
// TARGET_URL=http://localhost:3000 OUTPUT=proof/actual.png npx playwright test scripts/snapshot.spec.ts

const targetUrl = process.env.TARGET_URL || 'http://localhost:3000';
const outputPath = process.env.OUTPUT || 'proof/actual.png';

test('capture screenshot', async ({ page }) => {
  console.log(`Navigating to ${targetUrl}`);
  await page.goto(targetUrl);

  // Wait for network idle or specific element if needed
  // For now, wait for network idle to ensure assets loaded
  await page.waitForLoadState('networkidle');

  // Ensure output directory exists
  const dir = path.dirname(outputPath);
  if (!fs.existsSync(dir)){
      fs.mkdirSync(dir, { recursive: true });
  }

  console.log(`Saving screenshot to ${outputPath}`);
  await page.screenshot({ path: outputPath, fullPage: true });
});
