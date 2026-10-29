// The message for the next game (story C of duties). The server's clock is Monday 12 October 2026
// (DEV_NOW). These tests run after the tests of duties (one database, in this order): Blue Drakes
// M16 1 is chosen and has coming games, so there is a message.
import { expect, test } from '@playwright/test';
import { expectAccessible, expectNoHorizontalScroll, watchConsole } from './helpers';

test('the start page opens the message for the next game, and "Back" returns', async ({ page }) => {
	const errors = watchConsole(page);
	await page.goto('/');
	const messages = page.getByTestId('messages');
	await expect(messages.getByRole('heading', { name: 'Messages' })).toBeVisible();
	await messages.getByRole('link', { name: 'Message for the next game' }).click();
	await expect(page).toHaveURL(/\/messages\/next-game$/);

	// The text on the page is the text of the server.
	const answer = await (await page.request.get('/api/messages/next-game')).json();
	expect(answer.text).toMatch(/^Volgende wedstrijd: /);
	await expect(page.getByTestId('message-text')).toHaveText(answer.text);

	await page.getByRole('link', { name: 'Back' }).click();
	await expect(page.getByTestId('messages')).toBeVisible();
	expect(errors).toEqual([]);
});

test('"Copy" puts the text of the message on the clipboard', async ({ page }) => {
	const errors = watchConsole(page);
	await page.goto('/messages/next-game');
	const text = await page.getByTestId('message-text').textContent();
	expect(text).toContain('Volgende wedstrijd:');

	await page.getByTestId('copy').click();
	await expect(page.getByTestId('copy-state')).toHaveText('Copied');
	expect(await page.evaluate(() => navigator.clipboard.readText())).toBe(text);
	expect(errors).toEqual([]);
});

test('when the browser refuses the clipboard, the text stays to select', async ({ page }) => {
	await page.addInitScript(() => {
		navigator.clipboard.writeText = () =>
			Promise.reject(new DOMException('Refused', 'NotAllowedError'));
	});
	await page.goto('/messages/next-game');
	await page.getByTestId('copy').click();
	await expect(page.getByTestId('copy-state')).toHaveText('Select the text and copy it.');
	await expect(page.getByTestId('message-text')).toContainText('Volgende wedstrijd:');
});

test('the message page fits a phone and is accessible', async ({ page }) => {
	await page.goto('/messages/next-game');
	await expect(page.getByTestId('message-text')).toBeVisible();
	await expectNoHorizontalScroll(page, 390);
	await expectAccessible(page);
});

test('without a message the start page has no "Messages", and the page says so', async ({
	page
}) => {
	await page.route('**/api/start', async (route) => {
		const response = await route.fetch();
		const answer = await response.json();
		await route.fulfill({ response, json: { ...answer, messages: { next_game: false } } });
	});
	await page.route('**/api/messages/next-game', (route) => route.fulfill({ json: { text: null } }));
	await page.goto('/');
	await expect(page.getByTestId('next')).toBeVisible();
	await expect(page.getByTestId('messages')).toHaveCount(0);

	await page.goto('/messages/next-game');
	await expect(page.getByTestId('no-message')).toHaveText(
		'No message: no coming game and no duties this week.'
	);
	await expect(page.getByTestId('copy')).toHaveCount(0);
});
