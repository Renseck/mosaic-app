import { test as setup, expect } from '@playwright/test';
import path from 'path';

const AUTH_FILE = path.join(__dirname, '.auth/admin.json');

setup('bootstrap admin and authenticate', async ({ request }) => {
    const username = process.env.PORTAL_ADMIN_USERNAME!;
    const password = process.env.PORTAL_ADMIN_PASSWORD!;

    // 1. Register first user (becomes admin). 401 = already exists, that's fine.
    const registerRes = await request.post('/api/auth/register', {
        data: { username, password },
    });
    expect([201, 401]).toContain(registerRes.status());

    // 2. Log in to get a session cookie
    const loginRes = await request.post('/api/auth/login', {
        data : { username, password },
    });
    expect(loginRes.ok()).toBeTruthy();

    // 3. Verify the session works
    const meRes = await request.get('/api/auth/me');
    expect(meRes.ok()).toBeTruthy();
    const me = await meRes.json();
    expect(me.role).toBe('admin');

    // 4. Persist cookie state for all dependent test projects
    await request.storageState({ path: AUTH_FILE });
})