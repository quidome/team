import AxeBuilder from '@axe-core/playwright';
import { expect, type Page } from '@playwright/test';

/** Fails the test on console errors, for example a blocked script or style (security policy). */
export function watchConsole(page: Page): string[] {
	const errors: string[] = [];
	page.on('console', (message) => {
		if (message.type() !== 'error') return;
		// Without login, the app asks /api/me and gets 401: expected, not an error.
		const notLoggedIn =
			message.location().url.endsWith('/api/me') && message.text().includes('401');
		if (!notLoggedIn) errors.push(message.text());
	});
	page.on('pageerror', (error) => errors.push(error.message));
	return errors;
}

/** No serious or critical accessibility problems. Waits for running animations first: axe
 * measures colour contrast halfway a fade (`transition-all` on the label buttons) otherwise. */
export async function expectAccessible(page: Page) {
	await page.evaluate(() =>
		Promise.all(document.getAnimations().map((animation) => animation.finished))
	);
	const results = await new AxeBuilder({ page }).analyze();
	const serious = results.violations.filter(
		(violation) => violation.impact === 'serious' || violation.impact === 'critical'
	);
	expect(serious.map((violation) => `${violation.id}: ${violation.help}`)).toEqual([]);
}

/** No horizontal scrolling at this width. */
export async function expectNoHorizontalScroll(page: Page, width: number) {
	await page.setViewportSize({ width, height: 800 });
	const overflow = await page.evaluate(
		() => document.documentElement.scrollWidth - document.documentElement.clientWidth
	);
	expect(overflow).toBeLessThanOrEqual(0);
}
