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

const pastLabel = (page: import('@playwright/test').Page) =>
	page.getByTestId('labels').getByRole('button', { name: 'Past', exact: true });

test('the list of the season, and a tap on a row opens the training', async ({ page }) => {
	const errors = watchConsole(page);
	await page.goto('/season');
	await pastLabel(page).click();
	const coming = page.getByTestId('part-coming').getByTestId('training-row');
	const past = page.getByTestId('part-past').getByTestId('training-row');
	await expect(coming.first()).toHaveAttribute('data-date', '2026-10-13');
	await expect(past.first()).toHaveAttribute('data-date', '2026-10-06');
	await expect(past.first().getByTestId('training-status')).toHaveText('Not opened');
	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);

	await past.first().getByRole('button').first().click();
	await expect(page).toHaveURL(/\/training\/2026-10-06$/);
	await expect(page.getByTestId('training-heading')).toHaveText('Tue 6 Oct · 19:45–21:00');
	await expect(page.getByTestId('training-top')).toHaveText('All there');
	await expect(page.getByTestId('training-player').first()).toBeVisible();
	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);

	// "Back" goes to "Season" of the training's season, with "Past" still on.
	await page.getByRole('link', { name: 'Back' }).click();
	await expect(page).toHaveURL(/\/season\?season=2026-2027$/);
	await expect(pastLabel(page)).toHaveAttribute('aria-pressed', 'true');
	await expect(page.locator('[data-date="2026-10-06"]').getByTestId('training-status')).toHaveText(
		'All there'
	);
	expect(errors).toEqual([]);
});

test('"Cancel training" on a day that is not opened, and "Show cancelled"', async ({ page }) => {
	const errors = watchConsole(page);
	await page.goto('/season');
	const tuesday = page.locator('[data-testid="training-row"][data-date="2026-10-20"]');
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
	await page.goto('/season');
	await page.getByRole('button', { name: 'Add training' }).click();
	const form = page.getByTestId('add-training');
	await expect(form.getByLabel('Date')).toHaveAttribute('min', '2026-07-01');
	await form.getByLabel('Date').fill('2026-10-22');
	await form.getByLabel('Start').fill('19:00');
	await form.getByLabel('End').fill('20:30');
	await form.getByRole('button', { name: 'Save' }).click();
	await expect(page).toHaveURL(/\/training\/2026-10-22$/);
	await expect(page.getByTestId('training-heading')).toHaveText('Thu 22 Oct · 19:00–20:30');

	await page.goto('/season');
	await expect(page.locator('[data-testid="training-row"][data-date="2026-10-22"]')).toContainText(
		'extra'
	);
	await page.getByRole('button', { name: 'Add training' }).click();
	await form.getByLabel('Date').fill('2026-11-03');
	await form.getByLabel('Date').dispatchEvent('change');
	await expect(form.getByLabel('Start')).toHaveValue('19:45');
	await expect(form.getByLabel('End')).toHaveValue('21:00');
	await form.getByRole('button', { name: 'Save' }).click();
	await expect(page.getByTestId('training-heading')).toHaveText('Tue 3 Nov · 19:45–21:00');

	// The same date again is refused.
	await page.goto('/season');
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
	await page.goto('/season');
	await expect(
		page
			.locator('[data-testid="training-row"][data-date="2026-10-27"]')
			.getByTestId('training-status')
	).toHaveText('Not opened');

	await page.goto('/training/2026-10-27');
	await page.getByRole('button', { name: 'Open training' }).click();
	await expect(page.getByTestId('training-top')).toHaveText('All there');
	expect(errors).toEqual([]);
});

test('a tap on the card opens the next training', async ({ page }) => {
	const errors = watchConsole(page);
	await page.goto('/');
	await page.getByTestId('training-card').click();
	await expect(page).toHaveURL(/\/training\/2026-10-13\?from=start$/);
	await expect(page.getByTestId('training-top')).toHaveText('All there');
	// "Back" returns to the start page it came from.
	await page.getByRole('link', { name: 'Back' }).click();
	await expect(page).toHaveURL(/\/$/);
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
	// The row shows the tap at once; it is stored when "saving…" is gone.
	await expect(yara.getByTestId('saving')).toHaveCount(0);
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
	await expect(yara.getByTestId('saving')).toHaveCount(0);
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
	await expect(yara.getByTestId('saving')).toHaveText('saving…');
	// Before the first answer: another reason.
	await yara.getByRole('button', { name: /Yara/ }).click();
	await yara.getByRole('button', { name: 'Ill' }).click();
	await expect(yara.getByTestId('player-mark')).toHaveText('Ill');
	await expect.poll(() => order).toEqual(['first sent', 'first answered', 'second sent']);
	await expect(yara.getByTestId('player-mark')).toHaveText('Ill');
	await expect(yara.getByTestId('saving')).toHaveCount(0);
	await page.unrouteAll({ behavior: 'ignoreErrors' });
	await page.reload();
	await expect(yara.getByTestId('player-mark')).toHaveText('Ill');
	expect(errors).toEqual([]);
});

test('a tap without an answer for 10 seconds counts as not saved', async ({ page }) => {
	test.setTimeout(60_000);
	const errors = watchConsole(page);
	// The answer never comes (the network in the hall is gone).
	await page.route('**/api/trainings/2026-10-06/absences/*', () => new Promise(() => {}));
	await page.goto('/training/2026-10-06');
	const yara = page.getByTestId('training-player').filter({ hasText: 'Yara' });
	await yara.getByRole('button', { name: /Yara/ }).click();
	await yara.getByRole('button', { name: 'School' }).click();
	await expect(yara.getByTestId('player-mark')).toHaveText('School');
	await expect(yara.getByTestId('saving')).toBeVisible();
	await expect(page.getByTestId('not-saved')).toHaveText('Not saved. Try again.', {
		timeout: 15_000
	});
	await expect(yara.getByTestId('player-mark')).toHaveText('Ill');
	await expect(yara.getByTestId('saving')).toHaveCount(0);
	await page.unrouteAll({ behavior: 'ignoreErrors' });
	await page.reload();
	await expect(yara.getByTestId('player-mark')).toHaveText('Ill');
	expect(errors.filter((line) => !line.includes('ERR_ABORTED'))).toEqual([]);
});

test('a reload that was sent before a tap does not undo the tap', async ({ page }) => {
	const errors = watchConsole(page);
	await page.goto('/training/2026-10-06');
	const yara = page.getByTestId('training-player').filter({ hasText: 'Yara' });
	await expect(yara.getByTestId('player-mark')).toHaveText('Ill');
	// The reload when the phone comes back is slow; its answer still says "Ill".
	await page.route('**/api/trainings/2026-10-06', async (route) => {
		const answer = await route.fetch();
		await new Promise((done) => setTimeout(done, 1500));
		await route.fulfill({ response: answer });
	});
	const reloaded = page.waitForResponse('**/api/trainings/2026-10-06');
	await page.evaluate(() => document.dispatchEvent(new Event('visibilitychange')));
	await yara.getByRole('button', { name: /Yara/ }).click();
	await yara.getByRole('button', { name: 'There' }).click();
	await expect(yara.getByTestId('saving')).toHaveCount(0);
	await reloaded;
	await page.waitForTimeout(200);
	await expect(yara.getByTestId('player-mark')).toHaveText('there');
	await page.unrouteAll({ behavior: 'ignoreErrors' });
	// Back to "Ill" for the tests after this one.
	await yara.getByRole('button', { name: /Yara/ }).click();
	await yara.getByRole('button', { name: 'Ill' }).click();
	await expect(yara.getByTestId('saving')).toHaveCount(0);
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
	await expect(page).toHaveURL(/\/season\?season=2026-2027$/);
	expect(messages).toEqual(['Delete the training of Tue 3 Nov? Its attendance is removed.']);
	await expect(page.locator('[data-date="2026-11-03"]').getByTestId('training-status')).toHaveText(
		'Not opened'
	);
	await expect(page.locator('[data-date="2026-10-21"]')).toContainText('extra');
	expect(errors).toEqual([]);
});

test('"Checked" and "Undo check" on a training', async ({ page }) => {
	const errors = watchConsole(page);
	await page.goto('/training/2026-10-06');
	await page.getByRole('button', { name: 'Checked', exact: true }).click();
	await expect(page.getByTestId('checked')).toHaveText('Checked');
	await expect(page.getByRole('button', { name: 'Undo check' })).toBeVisible();
	await expect(page.getByRole('button', { name: 'Checked', exact: true })).toHaveCount(0);
	await page.reload();
	await expect(page.getByTestId('checked')).toHaveText('Checked');
	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);

	// "Season" shows it on the row.
	await page.goto('/season');
	await pastLabel(page).click();
	await expect(
		page
			.locator('[data-testid="training-row"][data-date="2026-10-06"]')
			.getByTestId('training-status')
	).toHaveText(/ · Checked$/);

	await page.goto('/training/2026-10-06');
	await page.getByRole('button', { name: 'Undo check' }).click();
	await expect(page.getByRole('button', { name: 'Checked', exact: true })).toBeVisible();
	await expect(page.getByTestId('checked')).toHaveCount(0);

	// A training after today is not complete yet: no "Checked".
	await page.goto('/training/2026-10-13');
	await expect(page.getByTestId('training-top')).not.toHaveText('Not opened');
	await expect(page.getByRole('button', { name: 'Checked', exact: true })).toHaveCount(0);
	expect(errors).toEqual([]);
});

test('a check that cannot be saved shows "Not saved. Try again." and the stored state', async ({
	page
}) => {
	// The server saves the check, but its answer is lost: the page shows what is stored.
	await page.route('**/api/trainings/2026-10-06/checked', async (route) => {
		await route.fetch();
		await route.fulfill({ status: 500, json: { error: 'internal' } });
	});
	await page.goto('/training/2026-10-06');
	await page.getByRole('button', { name: 'Checked', exact: true }).click();
	await expect(page.getByTestId('not-saved')).toHaveText('Not saved. Try again.');
	await expect(page.getByTestId('checked')).toHaveText('Checked');
	await page.unrouteAll({ behavior: 'ignoreErrors' });
	await page.getByRole('button', { name: 'Undo check' }).click();
	await expect(page.getByRole('button', { name: 'Checked', exact: true })).toBeVisible();
});

test('a check of a training deleted on another screen says so', async ({ page }) => {
	await page.route('**/api/trainings/2026-10-06/checked', (route) =>
		route.fulfill({ status: 400, json: { error: 'not_opened' } })
	);
	await page.goto('/training/2026-10-06');
	await page.getByRole('button', { name: 'Checked', exact: true }).click();
	await expect(page.getByRole('alert')).toHaveText(
		'This training no longer exists. Load the page again.'
	);
	await expect(page.getByTestId('not-saved')).toHaveCount(0);
});

test('a cancelled training has no "Checked"', async ({ page }) => {
	await page.goto('/training/2026-10-06');
	await page.getByRole('button', { name: 'Cancel training' }).click();
	await expect(page.getByTestId('training-top')).toHaveText('Cancelled');
	await expect(page.getByRole('button', { name: 'Checked', exact: true })).toHaveCount(0);
	await page.getByRole('button', { name: 'Undo cancel' }).click();
	await expect(page.getByRole('button', { name: 'Checked', exact: true })).toBeVisible();
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

test('a player added from 1 September counts as there at the trainings since', async ({ page }) => {
	const errors = watchConsole(page);
	// The tests before opened Tue 6, Tue 13 and Tue 27 Oct, moved an extra training to Wed 21 Oct,
	// and cancelled Tue 20 Oct (a cancelled training does not count).
	await page.goto('/team');
	await page.getByRole('button', { name: 'Add player' }).click();
	const form = page.getByTestId('add-player');
	await form.getByLabel('Player').selectOption({ label: 'New player' });
	await form.getByLabel('First name').fill('Zane');
	await form.getByRole('button', { name: 'Save' }).click();
	await expect(page.getByTestId('now-there')).toHaveText(
		'This player now counts as there at 4 earlier trainings: Tue 6 Oct, Tue 13 Oct, Wed 21 Oct, Tue 27 Oct.'
	);
	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);
	expect(errors).toEqual([]);
});

test('a player with absences is not offered "Delete player"', async ({ page }) => {
	const errors = watchConsole(page);
	// Yara is absent ("Ill") on Tue 6 Oct; Zane has no absences.
	await page.goto('/players');
	await page.getByTestId('player').filter({ hasText: 'Yara' }).click();
	await expect(page.getByTestId('player-name')).toHaveText('Yara');
	await expect(page.getByRole('button', { name: 'Delete player' })).toHaveCount(0);
	await page.goto('/players');
	await page.getByTestId('player').filter({ hasText: 'Zane' }).click();
	await expect(page.getByRole('button', { name: 'Delete player' })).toBeVisible();
	expect(errors).toEqual([]);
});

test("a new season proposes last season's weekly trainings", async ({ page }) => {
	// The server's clock has no last season, so the answer is replaced; the API tests cover the
	// real proposal. "Add" saves it for real.
	const errors = watchConsole(page);
	await page.route('**/api/weekly-trainings', async (route) => {
		if (route.request().method() !== 'GET') return route.continue();
		const list = await (await route.fetch()).json();
		await route.fulfill({
			json: {
				...list,
				proposals: [
					{ weekday: 4, start: '19:00', end: '20:30', from: '2026-09-03', until: '2027-05-27' }
				]
			}
		});
	});
	await page.goto('/settings');
	const proposal = page.getByTestId('proposal');
	await expect(proposal).toHaveText(/Thursday\s+19:00–20:30\s*· Thu 3 Sept – Thu 27 May/);
	await expectAccessible(page);
	await expectNoHorizontalScroll(page, 320);
	await proposal.getByRole('button', { name: 'Add Thursday 19:00–20:30' }).click();
	await expect(page.getByTestId('proposals')).toHaveCount(0);
	await expect(
		page.getByTestId('weekly-training').filter({ hasText: 'Thursday 19:00–20:30' })
	).toBeVisible();
	await page.unrouteAll({ behavior: 'ignoreErrors' });
	expect(errors).toEqual([]);
});

// The overview (attendance at training, story C), on the data of the tests above.
test.describe('the overview', () => {
	test('an earlier season can be chosen on "Attendance"', async ({ page }) => {
		const errors = watchConsole(page);
		// 2025-2026 has no data, but a season in the address is in the choice.
		await page.goto('/attendance?season=2025-2026');
		await expect(page.getByRole('heading', { level: 1 })).toHaveText('Attendance · 2025-2026');
		const choice = page.getByTestId('season-choice');
		await expect(choice).toHaveValue('2025-2026');
		await choice.selectOption('2026-2027');
		await expect(page).toHaveURL(/\/attendance\?season=2026-2027$/);
		await expect(page.getByRole('heading', { level: 1 })).toHaveText('Attendance · 2026-2027');
		// Back to 2025-2026 with the browser (the choice offers it only while it is chosen).
		await page.goBack();
		await expect(page).toHaveURL(/\/attendance\?season=2025-2026$/);
		await expect(page.getByRole('heading', { level: 1 })).toHaveText('Attendance · 2025-2026');
		// The list of trainings is on "Season".
		await expect(page.getByRole('button', { name: 'Add training' })).toHaveCount(0);
		await expect(page.getByTestId('training-row')).toHaveCount(0);
		await expectAccessible(page);
		await expectNoHorizontalScroll(page, 320);
		expect(errors).toEqual([]);
	});

	test('an earlier season can be chosen on "Season", and "Back" returns to it', async ({
		page
	}) => {
		const errors = watchConsole(page);
		await page.goto('/season?season=2025-2026');
		await expect(page.getByRole('heading', { level: 1 })).toHaveText('Season 2025-2026');
		await expect(page.getByRole('button', { name: 'Add training' })).toHaveCount(0);
		await page.getByTestId('season-choice').selectOption('2026-2027');
		await expect(page).toHaveURL(/\/season\?season=2026-2027$/);

		// An open "Add training" form and its error do not move to another season.
		await page.getByRole('button', { name: 'Add training' }).click();
		const form = page.getByTestId('add-training');
		await form.getByLabel('Date').fill('2026-10-06');
		await form.getByLabel('Start').fill('19:45');
		await form.getByLabel('End').fill('21:00');
		await form.getByRole('button', { name: 'Save' }).click();
		await expect(form.getByRole('alert')).toHaveText('There is already a training on this date.');
		await page.goBack();
		await expect(page).toHaveURL(/\/season\?season=2025-2026$/);
		await expect(page.getByTestId('add-training')).toHaveCount(0);
		await expect(page.getByRole('alert')).toHaveCount(0);

		// The last training day of May 2026 belongs to 2025-2026: "Back" goes to that season.
		await page.goto('/training/2026-05-26');
		await page.getByRole('link', { name: 'Back' }).click();
		await expect(page).toHaveURL(/\/season\?season=2025-2026$/);
		// The refused save shows in the console as a failed request.
		expect(errors.filter((line) => !line.includes('400'))).toEqual([]);
	});

	test('a season that cannot be read', async ({ page }) => {
		const errors = watchConsole(page);
		await page.goto('/attendance?season=abc');
		await expect(page.getByTestId('invalid-season')).toHaveText('This season cannot be read.');
		await page.getByRole('link', { name: 'Current season' }).click();
		await expect(page.getByRole('heading', { level: 1 })).toHaveText('Attendance · 2026-2027');
		// The answer 400 shows in the console as a failed request.
		expect(errors.filter((line) => !line.includes('400'))).toEqual([]);
	});
	// The data of the tests above: on Mon 12 Oct only Tue 6 Oct counts (Yara "Ill"); the Tuesdays of
	// September and the Thursdays from Thu 3 Sep (the last weekly training added) were never opened:
	// 11 days. Zane has a role from Tue 1 Sep and no absences.
	test('the totals change while past trainings are entered', async ({ page }) => {
		const errors = watchConsole(page);
		const zane = page.getByTestId('player-totals').filter({ hasText: 'Zane' });
		await page.goto('/attendance');
		await expect(page.getByTestId('not-opened')).toHaveText('11 training days not opened');
		await expect(zane).toContainText('of 1 training');

		// The link opens the oldest day that is not opened: Tue 1 Sep. Zane was at school.
		await page.getByRole('link', { name: '11 training days not opened' }).click();
		await expect(page).toHaveURL(/\/training\/2026-09-01\?from=attendance$/);
		await page.getByRole('button', { name: 'Open training' }).click();
		const atTraining = page.getByTestId('training-player').filter({ hasText: 'Zane' });
		await atTraining.getByRole('button', { name: /Zane/ }).click();
		await atTraining.getByRole('button', { name: 'School' }).click();
		await expect(atTraining.getByTestId('saving')).toHaveCount(0);
		await page.getByRole('link', { name: 'Back' }).click();
		await expect(page.getByTestId('not-opened')).toHaveText('10 training days not opened');
		await expect(zane).toContainText('of 2 trainings');
		await expect(zane.getByTestId('totals-counts')).toHaveText(
			'1 there · 1 absent, valid · 0 absent, not valid'
		);

		// Three more Tuesdays from "Season", everyone there.
		for (const [date, left] of [
			['2026-09-08', '9 training days not opened'],
			['2026-09-15', '8 training days not opened'],
			['2026-09-22', '7 training days not opened']
		]) {
			await page.goto('/season');
			await page.getByTestId('labels').getByRole('button', { name: 'Past', exact: true }).click();
			await page
				.locator(`[data-testid="training-row"][data-date="${date}"]`)
				.getByRole('button')
				.first()
				.click();
			await expect(page).toHaveURL(new RegExp(`/training/${date}$`));
			await page.goto('/attendance');
			await expect(page.getByTestId('not-opened')).toHaveText(left);
		}
		await expect(zane).toContainText('of 5 trainings');
		await expect(zane.getByTestId('totals-counts')).toHaveText(
			'4 there · 1 absent, valid · 0 absent, not valid'
		);
		expect(errors).toEqual([]);
	});

	test('a tap on a player shows the trainings it missed', async ({ page }) => {
		const errors = watchConsole(page);
		await page.goto('/attendance');
		const zane = page.getByTestId('player-totals').filter({ hasText: 'Zane' });
		const button = zane.getByRole('button');
		await expect(button).toHaveAttribute('aria-expanded', 'false');
		await button.click();
		await expect(button).toHaveAttribute('aria-expanded', 'true');
		await expect(zane.getByTestId('missed')).toHaveText('Tue 1 Sept · School');
		await expectAccessible(page);
		await expectNoHorizontalScroll(page, 320);
		await button.click();
		await expect(zane.getByTestId('missed')).toHaveCount(0);
		expect(errors).toEqual([]);
	});

	test('a cancelled training does not count, and counts again after "Undo cancel"', async ({
		page
	}) => {
		const errors = watchConsole(page);
		const zane = page.getByTestId('player-totals').filter({ hasText: 'Zane' });
		await page.goto('/training/2026-09-01');
		await page.getByRole('button', { name: 'Cancel training' }).click();
		await expect(page.getByTestId('training-top')).toHaveText('Cancelled');
		await page.goto('/attendance');
		await expect(zane).toContainText('of 4 trainings');
		await expect(zane.getByTestId('totals-counts')).toHaveText(
			'4 there · 0 absent, valid · 0 absent, not valid'
		);

		await page.goto('/training/2026-09-01');
		await page.getByRole('button', { name: 'Undo cancel' }).click();
		await expect(page.getByTestId('training-top')).toHaveText('1 absent');
		await page.goto('/attendance');
		await expect(zane).toContainText('of 5 trainings');
		expect(errors).toEqual([]);
	});

	test('trainings at which no player had a role', async ({ page }) => {
		const errors = watchConsole(page);
		// Such a season needs trainings before any role; the answer is replaced.
		await page.route('**/api/attendance*', async (route) => {
			const overview = await (await route.fetch()).json();
			await route.fulfill({ json: { ...overview, team: [], left: [] } });
		});
		await page.goto('/attendance');
		await expect(page.getByTestId('no-player-totals')).toHaveText(
			'No player had a role at these trainings. Add players in Team.'
		);
		await page.unrouteAll({ behavior: 'ignoreErrors' });
		expect(errors).toEqual([]);
	});

	test('a season with no trainings yet', async ({ page }) => {
		const errors = watchConsole(page);
		await page.goto('/attendance?season=2025-2026');
		await expect(page.getByTestId('no-trainings-yet')).toHaveText('No trainings yet this season.');
		await expect(page.getByTestId('not-opened')).toHaveCount(0);
		expect(errors).toEqual([]);
	});
});
