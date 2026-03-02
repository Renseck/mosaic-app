import { test, expect } from '@playwright/test';

test.describe('Authentication flow', () => {
    test('logged-in user lands on dashboard list', async ({ page }) => {
        await page.goto('/');
        await expect(page).toHaveURL(/\/dashboards/);
    });

    test('logout redirects to login page', async({ page }) => {
        await page.goto('/dashboards');
        await expect(page).toHaveURL(/\/dashboards/);

        await page.getByRole('button', {name: /logout|sign out/i }).click();
        await expect(page).toHaveURL(/\/login/);
    });

    test('authenticated user is redirect to login', async ({ browser }) => {
        // Fresh context
        const context = await browser.newContext();
        const page = await context.newPage();

        await page.goto('http://localhost:' + (process.env.PORTAL_PORT || '8080') + '/dashboards');
        await expect(page).toHaveURL(/\/login/);

        await context.close();
    });

    test('login with valid credentials', async ({ browser }) => {
    const context = await browser.newContext();
    const page = await context.newPage();

    await page.goto('http://localhost:' + (process.env.PORTAL_PORT || '8080') + '/login');

    await page.fill('#username', process.env.PORTAL_ADMIN_USERNAME!);
    await page.fill('#password', process.env.PORTAL_ADMIN_PASSWORD!);
    await page.getByRole('button', { name: /sign in/i }).click();

    await expect(page).toHaveURL(/\/dashboards/);
    await context.close();
  });

  test('login with wrong password shows error', async ({ browser }) => {
    const context = await browser.newContext();
    const page = await context.newPage();

    await page.goto('http://localhost:' + (process.env.PORTAL_PORT || '8080') + '/login');

    await page.fill('#username', process.env.PORTAL_ADMIN_USERNAME!);
    await page.fill('#password', 'wrongpassword');
    await page.getByRole('button', { name: /sign in/i }).click();

    await expect(page.locator('.bg-red-50')).toBeVisible();
    await expect(page).toHaveURL(/\/login/);
    await context.close();
  });
});