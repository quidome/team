<!--
	Reads the game list on demand (story 3a): the read action, the result line, the time of the last
	good reading, a failure, and a held-back reading with "These games are really gone".
-->
<script lang="ts">
	import { untrack } from 'svelte';
	import { Button } from '#lib/components/ui/button/index.js';
	import {
		ApiError,
		confirmGone,
		messageFor,
		readNow,
		type LatestReadings,
		type Reading
	} from '#lib/api/client.js';
	import { dutchTime, resultLine } from '#lib/reading.js';

	let { readings }: { readings: LatestReadings } = $props();

	let latest = $state<Reading | null>(untrack(() => readings.latest ?? null));
	let lastGood = $state(untrack(() => readings.last_good ?? null));
	/** The reading of the last tap on this page; the result line shows only for it. */
	let tapped = $state<Reading | null>(null);
	let busy = $state(false);
	let error = $state<string | null>(null);

	async function run(action: () => Promise<Reading>) {
		busy = true;
		error = null;
		try {
			const reading = await action();
			latest = reading;
			tapped = reading;
			if (reading.result === 'good' && reading.games_read != null) {
				lastGood = { finished_at: reading.finished_at, games_read: reading.games_read };
			}
		} catch (failure) {
			error =
				failure instanceof ApiError ? failure.message : 'Something went wrong. Please try again.';
		} finally {
			busy = false;
		}
	}
</script>

<div class="space-y-3">
	<Button onclick={() => run(readNow)} disabled={busy}>
		{busy ? 'Reading…' : 'Read the game list'}
	</Button>

	{#if tapped?.result === 'good'}
		<p class="text-sm font-medium" role="status" data-testid="reading-result">
			{resultLine(tapped)}
		</p>
	{/if}

	{#if lastGood}
		<p class="text-sm text-muted-foreground" data-testid="last-read">
			Last read: {dutchTime(lastGood.finished_at)}
		</p>
	{/if}

	{#if latest?.result === 'failed'}
		<p class="text-sm" role="alert" data-testid="reading-failed">
			The last reading failed: {messageFor({ error: latest.error }, 200)}
		</p>
	{/if}

	{#if latest?.result === 'held_back' && latest.hold}
		{@const held = latest}
		<div
			class="space-y-2 rounded-xl border border-destructive/40 bg-destructive/10 p-4"
			role="alert"
			data-testid="reading-held-back"
		>
			<p class="text-sm">
				{#if latest.hold.reason === 'empty'}
					The game list has no games now.
				{:else}
					The game list has far fewer games than before ({latest.hold.now} instead of
					{latest.hold.before}).
				{/if}
				The games are kept.
			</p>
			<Button variant="outline" disabled={busy} onclick={() => run(() => confirmGone(held.id))}>
				These games are really gone
			</Button>
		</div>
	{/if}

	{#if error}
		<p class="text-sm" role="alert">{error}</p>
	{/if}
</div>
