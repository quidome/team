<!--
	One row of the panel "Administration" (the administration panel): a mark of an event, with its
	state as a fact ("Not checked", "Checked"), ✓ when set and ○ when not, an optional line below,
	and one button: the action of a tap when not set, "Undo" when set. Only the next step of the
	coordinator (`next`) is orange. `data-done` and `data-next` let the browser tests read the state.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import { Button } from '#lib/components/ui/button/index.js';

	let {
		state,
		done,
		line,
		action,
		next = false,
		busy,
		onclick,
		testid
	}: {
		/** The fact: "Not checked", "Checked", "Schedule entered", … */
		state: string;
		/** ✓ when true, ○ when false. */
		done: boolean;
		/** The line below the state, for example "2 absent". */
		line?: Snippet;
		/** The words of the button: what a tap does. */
		action: string;
		/** The next step the coordinator must do: the button is orange. */
		next?: boolean;
		busy: boolean;
		onclick: () => void;
		testid: string;
	} = $props();
</script>

<div
	class="flex items-center justify-between gap-3 py-3"
	data-testid={testid}
	data-done={done}
	data-next={next}
>
	<div class="min-w-0">
		<p class="font-medium" role="status">
			{state}
			<span class={done ? '' : 'text-muted-foreground'} aria-hidden="true">{done ? '✓' : '○'}</span>
		</p>
		{#if line}
			<div class="text-sm text-muted-foreground">{@render line()}</div>
		{/if}
	</div>
	<Button
		class="min-h-11 shrink-0"
		variant={next ? 'default' : 'outline'}
		disabled={busy}
		{onclick}
		data-testid="{testid}-button">{action}</Button
	>
</div>
