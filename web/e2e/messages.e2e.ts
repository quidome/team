// The page of the message of the week and its button (story C of duties). The server's clock is
// Monday 12 October 2026 (DEV_NOW), and the test data has no game in that week; other tests depend
// on that. So the tests with a message fix the answers of the server here;
// `crates/server/tests/messages.rs` tests the text from stored data, and that the button and the
// message agree.
import { expect, test, type Page } from '@playwright/test';
import { expectAccessible, expectNoHorizontalScroll, watchConsole } from './helpers';

const TEXT = `Week 12 oktober – 18 oktober

Tafel en scheidsrechter:
za 17 okt 16:00 Blue Drakes X12 1 – De Dunckers X12 1
  Scheidsrechter: Gijs
  Tafel: Eva, nog 1 open`;

/** The answers of the server for a week with a message. */
async function answers(page: Page, text: string) {
	await page.route('**/api/start', async (route) => {
		const response = await route.fetch();
		const answer = await response.json();
		await route.fulfill({ response, json: { ...answer, messages: { week: true } } });
	});
	await page.route('**/api/messages/week', (route) => route.fulfill({ json: { text } }));
}

test('the start page opens the message of the week, and "Back" returns', async ({ page }) => {
	const errors = watchConsole(page);
	await answers(page, TEXT);
	await page.goto('/');
	const messages = page.getByTestId('messages');
	await expect(messages.getByRole('heading', { name: 'Messages' })).toBeVisible();
	await messages.getByRole('link', { name: 'Message of the week' }).click();
	await expect(page).toHaveURL(/\/messages\/week$/);

	await expect(page.getByTestId('message-text')).toHaveText(TEXT);

	await page.getByRole('link', { name: 'Back' }).click();
	await expect(page.getByTestId('messages')).toBeVisible();
	expect(errors).toEqual([]);
});

test('"Copy" puts the text of the message on the clipboard', async ({ page }) => {
	const errors = watchConsole(page);
	await answers(page, TEXT);
	await page.goto('/messages/week');

	await page.getByTestId('copy').click();
	await expect(page.getByTestId('copy-state')).toHaveText('Copied');
	expect(await page.evaluate(() => navigator.clipboard.readText())).toBe(TEXT);
	expect(errors).toEqual([]);
});

test('when the browser refuses the clipboard, the text stays to select', async ({ page }) => {
	await page.addInitScript(() => {
		navigator.clipboard.writeText = () =>
			Promise.reject(new DOMException('Refused', 'NotAllowedError'));
	});
	await answers(page, TEXT);
	await page.goto('/messages/week');
	await page.getByTestId('copy').click();
	await expect(page.getByTestId('copy-state')).toHaveText('Select the text and copy it.');
	await expect(page.getByTestId('message-text')).toHaveText(TEXT);
});

test('the message page fits a phone and is accessible', async ({ page }) => {
	await answers(page, TEXT);
	await page.goto('/messages/week');
	await expect(page.getByTestId('message-text')).toBeVisible();
	await expectNoHorizontalScroll(page, 390);
	await expectAccessible(page);
});

test('without a message the start page has no "Messages", and the page says so', async ({
	page
}) => {
	// The real answers: the test data has nothing in this week.
	await page.goto('/');
	await expect(page.getByTestId('next')).toBeVisible();
	await expect(page.getByTestId('messages')).toHaveCount(0);

	await page.goto('/messages/week');
	await expect(page.getByTestId('no-message')).toHaveText('No message this week.');
	await expect(page.getByTestId('copy')).toHaveCount(0);
});
