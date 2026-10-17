<!--
	Attendance (attendance at training, stories B and C): every training day and training of the
	chosen season (the current one by default, `?season=` in the address), "Coming" above "Past". A
	tap on a row opens the training (and so makes it exist). "Cancel training" on a day that is not
	opened, "Show cancelled", and "Add training" (only in the current season). Above the list: the
	training days not opened, and the totals of each player, then "Left the team"; a tap on a player
	shows the trainings it missed.
-->
<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import {
		ApiError,
		addTraining,
		cancelTraining,
		loadAttendance,
		loadTraining,
		loadTrainings,
		openTraining,
		type PlayerTotals,
		type TrainingRow
	} from '#lib/api/client.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import { formatDay } from '#lib/schedule.js';
	import { fullName } from '#lib/players.js';
	import { countsText, notOpenedText, ofTrainings, statusText } from '#lib/trainings.js';
	import { onMount } from 'svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	// From the load (a new season in the address loads again); replaced after a cancel on this page.
	let list = $derived(data.list);
	let overview = $derived(data.overview);

	const showCancelledKey = 'team.attendance.showCancelled';
	let showCancelled = $state(true);
	onMount(() => {
		try {
			showCancelled = localStorage.getItem(showCancelledKey) !== 'false';
		} catch {
			// No storage: the default stays.
		}
	});
	function setShowCancelled(value: boolean) {
		showCancelled = value;
		try {
			localStorage.setItem(showCancelledKey, String(value));
		} catch {
			// No storage: only this visit.
		}
	}

	// The players whose missed trainings are shown.
	let expanded = $state<number[]>([]);
	function toggle(player: number) {
		expanded = expanded.includes(player)
			? expanded.filter((id) => id !== player)
			: [...expanded, player];
	}

	let adding = $state(false);
	let addDate = $state('');
	let addStart = $state('');
	let addEnd = $state('');

	let busy = $state(false);
	let error = $state<string | null>(null);

	function message(failure: unknown): string {
		return failure instanceof ApiError
			? failure.message
			: 'Something went wrong. Please try again.';
	}

	function shown(rows: TrainingRow[]): TrainingRow[] {
		return showCancelled ? rows : rows.filter((row) => row.status !== 'cancelled');
	}

	/** A tap on a row: open the training when it is not opened yet, then show it. */
	async function open(row: TrainingRow) {
		busy = true;
		error = null;
		try {
			if (!row.opened) await openTraining(row.date);
			await goto(resolve('/training/[date]', { date: row.date }));
		} catch (failure) {
			error = message(failure);
		} finally {
			busy = false;
		}
	}

	async function cancel(row: TrainingRow) {
		busy = true;
		error = null;
		try {
			await cancelTraining(row.date);
			const season = list?.season;
			[list, overview] = await Promise.all([
				loadTrainings(fetch, season),
				loadAttendance(fetch, season)
			]);
		} catch (failure) {
			error = message(failure);
		} finally {
			busy = false;
		}
	}

	function startAdding() {
		adding = true;
		addDate = '';
		addStart = '';
		addEnd = '';
		error = null;
	}

	/** A date is chosen: fill in the times of a weekly training on that day, if there is one. */
	async function dateChosen() {
		if (!addDate) return;
		try {
			const proposed = (await loadTraining(fetch, addDate)).proposed_times;
			if (proposed) {
				addStart = proposed.start;
				addEnd = proposed.end;
			}
		} catch {
			// The times can still be typed.
		}
	}

	function chooseSeason(season: string) {
		goto(resolve(`/attendance?season=${encodeURIComponent(season)}`));
	}

	async function add(event: SubmitEvent) {
		event.preventDefault();
		busy = true;
		error = null;
		try {
			await addTraining(addDate, addStart, addEnd);
			await goto(resolve('/training/[date]', { date: addDate }));
		} catch (failure) {
			error = message(failure);
			busy = false;
		}
	}

	const field =
		'h-11 w-full min-w-0 rounded-md border border-input bg-background px-3 text-base shadow-xs focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none';
</script>

<svelte:head>
	<title>Attendance · Team</title>
</svelte:head>

{#snippet rows(items: TrainingRow[], testid: string)}
	<ul class="space-y-2" data-testid={testid}>
		{#each shown(items) as row (row.date)}
			<li
				class="flex items-stretch gap-2 rounded-xl border bg-card shadow-xs"
				data-testid="training-row"
				data-date={row.date}
			>
				<button
					type="button"
					class="flex min-h-12 min-w-0 flex-1 items-center justify-between gap-3 px-4 py-2 text-left disabled:opacity-60"
					disabled={busy}
					onclick={() => open(row)}
				>
					<span class="min-w-0 tabular-nums">
						<span class="font-medium">{formatDay(row.date)}</span>
						<span class="text-muted-foreground">· {row.start}</span>
						{#if row.extra}
							<span class="text-muted-foreground">· extra</span>
						{/if}
					</span>
					<span
						class="shrink-0 text-sm {row.status === 'not_opened' || row.status === 'cancelled'
							? 'text-muted-foreground'
							: ''}"
						data-testid="training-status"
					>
						{statusText(row)}
					</span>
				</button>
				{#if row.status === 'not_opened'}
					<div class="flex items-center pr-2">
						<Button
							variant="ghost"
							size="sm"
							disabled={busy}
							onclick={() => cancel(row)}
							aria-label="Cancel training of {formatDay(row.date)}"
						>
							Cancel
						</Button>
					</div>
				{/if}
			</li>
		{/each}
	</ul>
{/snippet}

{#snippet totals(players: PlayerTotals[], testid: string)}
	<ul class="space-y-2" data-testid={testid}>
		{#each players as player (player.id)}
			{@const shownMissed = expanded.includes(player.id)}
			<li class="rounded-xl border bg-card shadow-xs" data-testid="player-totals">
				<button
					type="button"
					class="flex min-h-12 w-full flex-col gap-0.5 px-4 py-2 text-left"
					aria-expanded={shownMissed}
					onclick={() => toggle(player.id)}
				>
					<span class="flex w-full items-baseline justify-between gap-3">
						<span class="min-w-0 font-medium">{fullName(player)}</span>
						<span class="shrink-0 text-sm text-muted-foreground tabular-nums">
							{ofTrainings(player.trainings)}
						</span>
					</span>
					<span class="text-sm tabular-nums" data-testid="totals-counts">{countsText(player)}</span>
				</button>
				{#if shownMissed}
					<div class="border-t px-4 py-2 text-sm" data-testid="missed">
						{#if player.missed.length === 0}
							<p class="text-muted-foreground">No missed trainings.</p>
						{:else}
							<ul class="space-y-1">
								{#each player.missed as missed (missed.date)}
									<li class="tabular-nums">{formatDay(missed.date)} · {missed.reason}</li>
								{/each}
							</ul>
						{/if}
					</div>
				{/if}
			</li>
		{/each}
	</ul>
{/snippet}

{#if data.invalidSeason}
	<section class="space-y-4 py-16 text-center">
		<p data-testid="invalid-season">This season cannot be read.</p>
		<Button href={resolve('/attendance')} variant="outline">Current season</Button>
	</section>
{:else if !list || !overview}
	<section class="py-16 text-center">
		<p class="text-muted-foreground">Log in to see the trainings.</p>
		<Button href="/auth/login" class="mt-4" data-sveltekit-reload>Log in</Button>
	</section>
{:else}
	{@const page = list}
	{@const current = overview.season === overview.current_season}
	<section class="space-y-6">
		<div class="flex items-center justify-between gap-3">
			<h1 class="text-2xl font-semibold tracking-tight">Attendance · {page.season}</h1>
			<Button href={resolve('/')} variant="ghost">Back</Button>
		</div>

		{#if overview.seasons.length > 1}
			<label class="flex items-center gap-3 text-sm">
				Season
				<select
					class="h-11 min-w-0 rounded-md border border-input bg-background px-3 text-base shadow-xs focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
					value={overview.season}
					onchange={(event) => chooseSeason(event.currentTarget.value)}
					data-testid="season-choice"
				>
					{#each overview.seasons as season (season)}
						<option value={season}>{season}</option>
					{/each}
				</select>
			</label>
		{/if}

		{#if overview.not_opened > 0 && overview.first_not_opened}
			<p class="text-sm" data-testid="not-opened">
				<a
					href={resolve('/training/[date]', { date: overview.first_not_opened })}
					class="font-medium underline underline-offset-4"
				>
					{notOpenedText(overview.not_opened)}
				</a>
			</p>
		{/if}

		<section aria-labelledby="totals-heading" class="space-y-2">
			<h2 id="totals-heading" class="text-lg font-semibold tracking-tight">Totals</h2>
			{#if overview.trainings === 0}
				<p class="text-sm text-muted-foreground" data-testid="no-trainings-yet">
					No trainings yet this season.
				</p>
			{:else}
				{@render totals(overview.team, 'team-totals')}
				{#if overview.left.length > 0}
					<h3 class="pt-2 font-semibold">Left the team</h3>
					{@render totals(overview.left, 'left-totals')}
				{/if}
			{/if}
		</section>

		<h2 class="text-lg font-semibold tracking-tight">Trainings</h2>

		{#if adding}
			<form
				class="space-y-3 rounded-xl border bg-card p-4 shadow-xs"
				onsubmit={add}
				data-testid="add-training"
			>
				<h2 class="font-medium">Add training</h2>
				<label class="block space-y-1">
					<span class="text-sm">Date</span>
					<input
						class={field}
						type="date"
						bind:value={addDate}
						onchange={dateChosen}
						min={page.season_first_day}
						max={page.season_last_day}
						required
					/>
				</label>
				<div class="grid grid-cols-2 gap-2">
					<label class="block min-w-0 space-y-1">
						<span class="text-sm">Start</span>
						<input class={field} type="time" bind:value={addStart} required />
					</label>
					<label class="block min-w-0 space-y-1">
						<span class="text-sm">End</span>
						<input class={field} type="time" bind:value={addEnd} required />
					</label>
				</div>
				{#if error}
					<p class="text-sm" role="alert">{error}</p>
				{/if}
				<div class="flex gap-2">
					<Button type="submit" disabled={busy}>Save</Button>
					<Button type="button" variant="ghost" onclick={() => (adding = false)}>Cancel</Button>
				</div>
			</form>
		{:else if current}
			<Button onclick={startAdding}>Add training</Button>
		{/if}

		{#if error && !adding}
			<p class="text-sm" role="alert">{error}</p>
		{/if}

		<label class="flex min-h-11 items-center gap-3 text-sm">
			<input
				type="checkbox"
				role="switch"
				class="size-5"
				checked={showCancelled}
				onchange={(event) => setShowCancelled(event.currentTarget.checked)}
			/>
			Show cancelled
		</label>

		{#if page.coming.length === 0 && page.past.length === 0}
			<p class="text-sm text-muted-foreground" data-testid="no-trainings">
				No training days or trainings this season. Add a weekly training in
				<a href={resolve('/settings')} class="font-medium underline underline-offset-4">Settings</a
				>.
			</p>
		{/if}

		{#if page.coming.length > 0}
			<section aria-labelledby="coming-heading" class="space-y-2">
				<h3 id="coming-heading" class="font-semibold">Coming</h3>
				{@render rows(page.coming, 'coming')}
			</section>
		{/if}

		{#if page.past.length > 0}
			<section aria-labelledby="past-heading" class="space-y-2">
				<h3 id="past-heading" class="font-semibold">Past</h3>
				{@render rows(page.past, 'past')}
			</section>
		{/if}
	</section>
{/if}
