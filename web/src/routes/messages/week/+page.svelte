<!--
	The message of the week (story C of duties): the Dutch text to copy into the chat of the team,
	with "Copy". The text comes from the server at each visit (what is stored now). When the
	browser refuses the clipboard (for example a page without HTTPS), the text stays shown to
	select by hand.
-->
<script lang="ts">
	import { resolve } from '$app/paths';
	import { Button } from '#lib/components/ui/button/index.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	let copy = $state<'idle' | 'copied' | 'failed'>('idle');

	async function copyText(text: string) {
		try {
			await navigator.clipboard.writeText(text);
			copy = 'copied';
		} catch {
			copy = 'failed';
		}
	}
</script>

<svelte:head>
	<title>Message of the week · Team</title>
</svelte:head>

{#if !data.message}
	<section class="py-16 text-center">
		<p class="text-muted-foreground">Log in to see the message.</p>
		<Button href="/auth/login" class="mt-4" data-sveltekit-reload>Log in</Button>
	</section>
{:else}
	{@const text = data.message.text}
	<section class="space-y-4">
		<div class="flex items-center justify-between gap-3">
			<h1 class="text-2xl font-semibold tracking-tight">Message of the week</h1>
			<Button href={resolve('/')} variant="ghost">Back</Button>
		</div>

		{#if text === null || text === undefined}
			<p class="text-sm text-muted-foreground" data-testid="no-message">No message this week.</p>
		{:else}
			<pre
				class="rounded-xl border bg-card p-4 font-sans text-sm break-words whitespace-pre-wrap shadow-xs select-all"
				data-testid="message-text">{text}</pre>

			<div class="flex items-center gap-3">
				<Button onclick={() => copyText(text)} data-testid="copy">Copy</Button>
				<p class="text-sm text-muted-foreground" aria-live="polite" data-testid="copy-state">
					{#if copy === 'copied'}
						Copied
					{:else if copy === 'failed'}
						Select the text and copy it.
					{/if}
				</p>
			</div>
		{/if}
	</section>
{/if}
