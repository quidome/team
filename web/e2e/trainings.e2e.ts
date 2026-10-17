// Trainings and attendance (attendance at training, story B). The server's clock is Monday 12
// October 2026 (DEV_NOW), so the season is 2026-2027. These tests run after the player tests (one
// database, in this order) and add their own Tuesday weekly training.
import { expect, test } from '@playwright/test';
import { expectAccessible, expectNoHorizontalScroll, watchConsole } from './helpers';

test.describe.configure({ mode: 'serial' });

test('a Tuesday weekly training gives the card on the start page', async ({ page }) => {
	const errors = watchConsole(page);
	await page.goto('/settings');
	await page.getByRole('button', { name: 'Add weekly training' }).click();
	const weekly = page.getByTestId('weekly-form');
	await weekly.getByLabel('Day').selectOption('2');
	await expect(weekly.getByLabel('First training')).toHaveValue('2026-09-01');
	await weekly.getByLabel('Start').fill('19:45');
	await weekly.getByLabel('End').fill('21:00');
	await weekly.getByRole('button', { name: 'Save weekly training' }).click();
	await expect(page.getByTestId('weekly-training')).toHaveCount(1);

	await page.goto('/');
	await expect(
		page.getByTestId('team-links').getByRole('link', { name: 'Attendance' })
	).toBeVisible();
	const card = page.getByTestId('training-card');
	await expect(card).toHaveText(/Training · Tue 13 Oct\s*· 19:45\s*Not opened yet/);
	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);
	expect(errors).toEqual([]);
});

test('the list of the season, and a tap on a row opens the training', async ({ page }) => {
	const errors = watchConsole(page);
	await page.goto('/attendance');
	await expect(page.getByRole('heading', { name: 'Attendance · 2026-2027' })).toBeVisible();
	const coming = page.getByTestId('coming').getByTestId('training-row');
	const past = page.getByTestId('past').getByTestId('training-row');
	await expect(coming.first()).toHaveAttribute('data-date', '2026-10-13');
	await expect(past.first()).toHaveAttribute('data-date', '2026-10-06');
	await expect(past.first().getByTestId('training-status')).toHaveText('Not opened');
	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);

	await past
		.first()
		.getByRole('button', { name: /^Tue 6 Oct/ })
		.click();
	await expect(page).toHaveURL(/\/training\/2026-10-06$/);
	await expect(page.getByTestId('training-heading')).toHaveText('Tue 6 Oct · 19:45–21:00');
	await expect(page.getByTestId('training-top')).toHaveText('All there');
	await expect(page.getByTestId('training-player').first()).toBeVisible();
	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);

	await page.getByRole('link', { name: 'Back' }).click();
	await expect(page.locator('[data-date="2026-10-06"]').getByTestId('training-status')).toHaveText(
		'All there'
	);
	expect(errors).toEqual([]);
});

test('"Cancel training" on a day that is not opened, and "Show cancelled"', async ({ page }) => {
	const errors = watchConsole(page);
	await page.goto('/attendance');
	const tuesday = page.locator('[data-date="2026-10-20"]');
	await tuesday.getByRole('button', { name: 'Cancel training of Tue 20 Oct' }).click();
	await expect(tuesday.getByTestId('training-status')).toHaveText('Cancelled');
	await page.getByLabel('Show cancelled').uncheck();
	await expect(tuesday).toHaveCount(0);
	await page.reload();
	await expect(page.getByLabel('Show cancelled')).not.toBeChecked();
	await expect(tuesday).toHaveCount(0);
	await page.getByLabel('Show cancelled').check();
	await expect(tuesday.getByTestId('training-status')).toHaveText('Cancelled');
	expect(errors).toEqual([]);
});

test('"Add training" with typed times, and with the proposed times', async ({ page }) => {
	const errors = watchConsole(page);
	await page.goto('/attendance');
	await page.getByRole('button', { name: 'Add training' }).click();
	const form = page.getByTestId('add-training');
	await expect(form.getByLabel('Date')).toHaveAttribute('min', '2026-07-01');
	await form.getByLabel('Date').fill('2026-10-22');
	await form.getByLabel('Start').fill('19:00');
	await form.getByLabel('End').fill('20:30');
	await form.getByRole('button', { name: 'Save' }).click();
	await expect(page).toHaveURL(/\/training\/2026-10-22$/);
	await expect(page.getByTestId('training-heading')).toHaveText('Thu 22 Oct · 19:00–20:30');

	await page.goto('/attendance');
	await expect(page.locator('[data-date="2026-10-22"]')).toContainText('extra');
	await page.getByRole('button', { name: 'Add training' }).click();
	await form.getByLabel('Date').fill('2026-11-03');
	await form.getByLabel('Date').dispatchEvent('change');
	await expect(form.getByLabel('Start')).toHaveValue('19:45');
	await expect(form.getByLabel('End')).toHaveValue('21:00');
	await form.getByRole('button', { name: 'Save' }).click();
	await expect(page.getByTestId('training-heading')).toHaveText('Tue 3 Nov · 19:45–21:00');

	// The same date again is refused.
	await page.goto('/attendance');
	await page.getByRole('button', { name: 'Add training' }).click();
	await form.getByLabel('Date').fill('2026-10-22');
	await form.getByLabel('Start').fill('19:00');
	await form.getByLabel('End').fill('20:30');
	await form.getByRole('button', { name: 'Save' }).click();
	await expect(form.getByRole('alert')).toHaveText('There is already a training on this date.');
	expect(errors).toEqual([
		'Failed to load resource: the server responded with a status of 400 (Bad Request)'
	]);
});

test('a training reached by its address is opened only with "Open training"', async ({ page }) => {
	const errors = watchConsole(page);
	await page.goto('/training/2026-10-27');
	await expect(page.getByTestId('training-top')).toHaveText('Not opened');
	await page.goto('/attendance');
	await expect(page.locator('[data-date="2026-10-27"]').getByTestId('training-status')).toHaveText(
		'Not opened'
	);

	await page.goto('/training/2026-10-27');
	await page.getByRole('button', { name: 'Open training' }).click();
	await expect(page.getByTestId('training-top')).toHaveText('All there');
	expect(errors).toEqual([]);
});

test('a tap on the card opens the next training', async ({ page }) => {
	const errors = watchConsole(page);
	await page.goto('/');
	await page.getByTestId('training-card').click();
	await expect(page).toHaveURL(/\/training\/2026-10-13$/);
	await expect(page.getByTestId('training-top')).toHaveText('All there');
	await page.goto('/');
	await expect(page.getByTestId('training-card')).toHaveText(/All there/);
	expect(errors).toEqual([]);
});
