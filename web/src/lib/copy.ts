/**
 * Puts `text` on the clipboard (decision 0024: the coordinator carries it to the chat). "failed"
 * when the browser refuses (for example a page without HTTPS): the page then shows the text to
 * select by hand, with "Select the text and copy it.".
 */
export async function copyText(text: string): Promise<'copied' | 'failed'> {
	try {
		await navigator.clipboard.writeText(text);
		return 'copied';
	} catch {
		return 'failed';
	}
}
