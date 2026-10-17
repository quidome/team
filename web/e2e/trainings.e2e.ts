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

test('mark a player absent with a reason, and there again', async ({ page }) => {
	const errors = watchConsole(page);
	// Yara only trains, from 1 September (the proposed date).
	await page.goto('/team');
	await page.getByRole('button', { name: 'Add player' }).click();
	const form = page.getByTestId('add-player');
	await form.getByLabel('Player').selectOption({ label: 'New player' });
	await form.getByLabel('First name').fill('Yara');
	await form.getByLabel('Only trains').check();
	await form.getByRole('button', { name: 'Save' }).click();
	await expect(page.getByTestId('member').filter({ hasText: 'Yara' })).toBeVisible();

	await page.goto('/training/2026-10-06');
	const yara = page.getByTestId('training-player').filter({ hasText: 'Yara' });
	await expect(yara.getByTestId('player-mark')).toHaveText('there');
	await yara.getByRole('button', { name: /Yara/ }).click();
	await expect(yara.getByTestId('reasons').getByRole('button').first()).toHaveText('School');
	await yara.getByRole('button', { name: 'Ill' }).click();
	await expect(yara.getByTestId('player-mark')).toHaveText('Ill');
	await expect(page.getByTestId('training-top')).toHaveText('1 absent');
	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);

	// Stored: also after a reload.
	await page.reload();
	await expect(yara.getByTestId('player-mark')).toHaveText('Ill');

	await yara.getByRole('button', { name: /Yara/ }).click();
	await expect(yara.getByTestId('reasons').getByRole('button').first()).toHaveText('There');
	await yara.getByRole('button', { name: 'There' }).click();
	await expect(yara.getByTestId('player-mark')).toHaveText('there');
	await expect(page.getByTestId('training-top')).toHaveText('All there');
	await page.reload();
	await expect(yara.getByTestId('player-mark')).toHaveText('there');
	expect(errors).toEqual([]);
});

test('a tap that cannot be saved shows the old state and "Not saved. Try again."', async ({
	page
}) => {
	const errors = watchConsole(page);
	await page.route('**/api/trainings/2026-10-06/absences/*', (route) =>
		route.fulfill({ status: 500, json: { error: 'internal' } })
	);
	await page.goto('/training/2026-10-06');
	const yara = page.getByTestId('training-player').filter({ hasText: 'Yara' });
	await yara.getByRole('button', { name: /Yara/ }).click();
	await yara.getByRole('button', { name: 'School' }).click();
	await expect(page.getByTestId('not-saved')).toHaveText('Not saved. Try again.');
	await expect(yara.getByTestId('player-mark')).toHaveText('there');
	await expect(page.getByTestId('training-top')).toHaveText('All there');
	await page.unrouteAll({ behavior: 'ignoreErrors' });
	await page.reload();
	await expect(yara.getByTestId('player-mark')).toHaveText('there');
	expect(errors).toEqual([
		'Failed to load resource: the server responded with a status of 500 (Internal Server Error)'
	]);
});

test('a second tap waits for the first, so a late answer never overwrites it', async ({ page }) => {
	const errors = watchConsole(page);
	const order: string[] = [];
	let first = true;
	await page.route('**/api/trainings/2026-10-06/absences/*', async (route) => {
		if (first) {
			first = false;
			order.push('first sent');
			// A slow network: the first answer comes late.
			await new Promise((done) => setTimeout(done, 1000));
			await route.continue();
			order.push('first answered');
		} else {
			order.push('second sent');
			await route.continue();
		}
	});
	await page.goto('/training/2026-10-06');
	const yara = page.getByTestId('training-player').filter({ hasText: 'Yara' });
	await yara.getByRole('button', { name: /Yara/ }).click();
	await yara.getByRole('button', { name: 'School' }).click();
	await expect(yara.getByTestId('player-mark')).toHaveText('School');
	// Before the first answer: another reason.
	await yara.getByRole('button', { name: /Yara/ }).click();
	await yara.getByRole('button', { name: 'Ill' }).click();
	await expect(yara.getByTestId('player-mark')).toHaveText('Ill');
	await expect.poll(() => order).toEqual(['first sent', 'first answered', 'second sent']);
	await expect(yara.getByTestId('player-mark')).toHaveText('Ill');
	await page.unrouteAll({ behavior: 'ignoreErrors' });
	await page.reload();
	await expect(yara.getByTestId('player-mark')).toHaveText('Ill');
	expect(errors).toEqual([]);
});

test('"Cancel training", "Undo cancel", "Change" and "Delete training"', async ({ page }) => {
	const errors = watchConsole(page);
	await page.goto('/training/2026-10-06');
	await page.getByRole('button', { name: 'Cancel training' }).click();
	await expect(page.getByTestId('training-top')).toHaveText('Cancelled');
	// A cancelled training keeps its absence and can still be marked.
	const yara = page.getByTestId('training-player').filter({ hasText: 'Yara' });
	await expect(yara.getByTestId('player-mark')).toHaveText('Ill');
	await page.getByRole('button', { name: 'Undo cancel' }).click();
	await expect(page.getByTestId('training-top')).toHaveText('1 absent');

	// "Change" of the extra training of Thu 22 Oct to Wed 21 Oct.
	await page.goto('/training/2026-10-22');
	await page.getByRole('button', { name: 'Change' }).click();
	const change = page.getByTestId('change-training');
	await expect(change.getByLabel('Date')).toHaveValue('2026-10-22');
	await change.getByLabel('Date').fill('2026-10-21');
	await change.getByRole('button', { name: 'Save' }).click();
	await expect(page).toHaveURL(/\/training\/2026-10-21$/);
	await expect(page.getByTestId('training-heading')).toHaveText('Wed 21 Oct · 19:00–20:30');
	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);

	// "Delete training" of Tue 3 Nov, after the confirmation.
	await page.goto('/training/2026-11-03');
	const messages: string[] = [];
	page.once('dialog', (dialog) => {
		messages.push(dialog.message());
		void dialog.accept();
	});
	await page.getByRole('button', { name: 'Delete training' }).click();
	await expect(page).toHaveURL(/\/attendance$/);
	expect(messages).toEqual(['Delete the training of Tue 3 Nov? Its attendance is removed.']);
	await expect(page.locator('[data-date="2026-11-03"]').getByTestId('training-status')).toHaveText(
		'Not opened'
	);
	await expect(page.locator('[data-date="2026-10-21"]')).toContainText('extra');
	expect(errors).toEqual([]);
});

test('no player with a role, and no reasons', async ({ page }) => {
	const errors = watchConsole(page);
	let players = false;
	await page.route('**/api/trainings/2026-10-06', async (route) => {
		const day = await (await route.fetch()).json();
		await route.fulfill({
			json: players
				? {
						...day,
						reasons: day.reasons.map((reason: object) => ({ ...reason, hidden: true }))
					}
				: { ...day, players: [] }
		});
	});
	await page.goto('/training/2026-10-06');
	await expect(page.getByTestId('no-players')).toHaveText('No player has a role on this date.');
	players = true;
	await page.reload();
	await expect(page.getByTestId('no-reasons')).toHaveText('No reasons. Add one in Reasons.');
	const yara = page.getByTestId('training-player').filter({ hasText: 'Yara' });
	await yara.getByRole('button', { name: /Yara/ }).click();
	await expect(yara.getByTestId('reasons').getByRole('button')).toHaveText(['There']);
	await page.unrouteAll({ behavior: 'ignoreErrors' });
	expect(errors).toEqual([]);
});
