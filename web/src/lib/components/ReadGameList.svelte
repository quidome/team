<!--
	Reads the game list on demand (story 3a): the read action, the result line, the time of the last
	good reading, a failure, and a held-back reading with "These games are really gone". After 3
	nights without a good reading it says that the games may be out of date (story 5).
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

	let {
		readings,
		onRead
	}: {
		readings: LatestReadings;
		/** Called after a good reading, for example to show the new schedule. */
		onRead?: () => void | Promise<void>;
	} = $props();

	let latest = $state<Reading | null>(untrack(() => readings.latest ?? null));
	let lastGood = $state(untrack(() => readings.last_good ?? null));
	/** The held-back reading to confirm; a failed reading after it does not hide it. */
	let held = $state<Reading | null>(untrack(() => readings.held_back ?? null));
	/** The time of the last good reading, when the nightly readings failed for 3 nights. */
	let outOfDateSince = $state(untrack(() => readings.out_of_date_since ?? null));
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
				held = null;
				outOfDateSince = null;
				await onRead?.();
			}
			if (reading.result === 'held_back') held = reading;
		} catch (failure) {
			error =
				failure instanceof ApiError ? failure.message : 'Something went wrong. Please try again.';
		} finally {
			busy = false;
		}
	}
</script>

<div class="space-y-3">
	{#if outOfDateSince}
		<p
			class="rounded-xl border border-destructive/40 bg-destructive/10 p-4 text-sm"
			role="alert"
			data-testid="out-of-date"
		>
			The games may be out of date since {dutchTime(outOfDateSince)}.
		</p>
	{/if}

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

	{#if held?.hold}
		{@const hold = held.hold}
		{@const heldId = held.id}
		<div
			class="space-y-2 rounded-xl border border-destructive/40 bg-destructive/10 p-4"
			role="alert"
			data-testid="reading-held-back"
		>
			<p class="text-sm">
				{#if hold.reason === 'empty'}
					The game list has no games now.
				{:else}
					The game list has far fewer games than before ({hold.now} instead of
					{hold.before}).
				{/if}
				The games are kept.
			</p>
			<Button variant="outline" disabled={busy} onclick={() => run(() => confirmGone(heldId))}>
				These games are really gone
			</Button>
		</div>
	{/if}

	{#if error}
		<p class="text-sm" role="alert">{error}</p>
	{/if}
</div>
