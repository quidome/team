<!--
	Season (story A of start page and season): all events of the chosen season (the current one by
	default, `?season=` in the address): the games of the team, the possible duty games, and the
	training days and trainings, under a heading per week and per date. Labels of "What" and "When"
	narrow the list; the page opens with "Coming" at each visit. A tap on a training opens it;
	"Cancel" on a training day that is not opened, "Show cancelled", and "Add training" (only in the
	current season). The current season has the "Game list" card.
-->
<script lang="ts">
	import { afterNavigate, goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import {
		ApiError,
		addTraining,
		cancelTraining,
		loadSeason,
		loadTraining,
		openTraining,
		type TrainingRow
	} from '#lib/api/client.js';
	import Labels from '#lib/components/Labels.svelte';
	import ReadGameList from '#lib/components/ReadGameList.svelte';
	import SeasonEvents from '#lib/components/SeasonEvents.svelte';
	import { Button } from '#lib/components/ui/button/index.js';
	import {
		emptyLine,
		keepForReturn,
		parts,
		startLabels,
		takeReturn,
		visible
	} from '#lib/season.js';
	import { onMount, tick } from 'svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	// From the load (another season in the address loads again); replaced after a reading or a
	// cancel on this page, without loading the page again: that would build the "Game list" card
	// again and lose its result line.
	let answer = $derived(data.answer);

	let labels = $state(startLabels());
	let busy = $state(false);
	let error = $state<string | null>(null);
	let loadError = $state(false);

	// The same setting as "Attendance" had, so a phone keeps its choice.
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

	let adding = $state(false);
	let addDate = $state('');
	let addStart = $state('');
	let addEnd = $state('');

	// Every visit, and every other season, starts with "Coming", no open form and no error; a return
	// from a training opened here keeps the labels and the place.
	afterNavigate(({ from }) => {
		const kept =
			from?.route.id === '/training/[date]' && answer
				? takeReturn(answer.season, from.params?.date)
				: null;
		labels = kept?.labels ?? startLabels();
		if (kept) tick().then(() => window.scrollTo(0, kept.scrollY));
		adding = false;
		error = null;
		loadError = false;
	});

	function message(failure: unknown): string {
		return failure instanceof ApiError
			? failure.message
			: 'Something went wrong. Please try again.';
	}

	function chooseSeason(season: string) {
		goto(resolve(`/season?season=${encodeURIComponent(season)}`));
	}

	/** The events again, for example after a good reading. */
	async function reload() {
		try {
			answer = await loadSeason(fetch, answer?.season);
			loadError = false;
		} catch {
			loadError = true;
		}
	}

	/** A tap on a training: open it when it is not opened yet, then show it. */
	async function open(row: TrainingRow) {
		busy = true;
		error = null;
		try {
			if (!row.opened) await openTraining(row.date);
			if (answer) keepForReturn(answer.season, row.date, labels, window.scrollY);
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
			answer = await loadSeason(fetch, answer?.season);
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
	<title>Season · Team</title>
</svelte:head>

{#if data.invalidSeason}
	<section class="space-y-4 py-16 text-center">
		<p data-testid="invalid-season">This season cannot be read.</p>
		<Button href={resolve('/season')} variant="outline">Current season</Button>
	</section>
{:else if !answer}
	<section class="py-16 text-center">
		<p class="text-muted-foreground">Log in to see the season.</p>
		<Button href="/auth/login" class="mt-4" data-sveltekit-reload>Log in</Button>
	</section>
{:else}
	{@const page = answer}
	{@const current = page.season === page.current_season}
	{@const empty = page.coming.length === 0 && page.past.length === 0}
	<section class="space-y-6">
		<div class="flex items-center justify-between gap-3">
			<h1 class="text-2xl font-semibold tracking-tight">Season {page.season}</h1>
			<Button href={resolve('/')} variant="ghost">Back</Button>
		</div>

		{#if page.seasons.length > 1}
			<label class="flex items-center gap-3 text-sm">
				Season
				<select
					class="h-11 min-w-0 rounded-md border border-input bg-background px-3 text-base shadow-xs focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
					value={page.season}
					onchange={(event) => chooseSeason(event.currentTarget.value)}
					data-testid="season-choice"
				>
					{#each page.seasons as season (season)}
						<option value={season}>{season}</option>
					{/each}
				</select>
			</label>
		{/if}

		{#if data.readings}
			<div class="rounded-xl border bg-card p-5 shadow-xs" data-testid="game-list-card">
				<h2 class="mb-3 font-medium">Game list</h2>
				{#key data.readings}
					<ReadGameList readings={data.readings} onRead={reload} />
				{/key}
			</div>
		{/if}
		{#if loadError}
			<p class="text-sm" role="alert" data-testid="season-load-error">
				The new games could not be loaded. Load the page again.
			</p>
		{/if}

		{#if page.games === 'team_needed'}
			<p class="text-sm" data-testid="team-needed">
				<a href={resolve('/')} class="font-medium underline underline-offset-4">
					Choose the team to see its games.
				</a>
			</p>
		{/if}

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

		{#if !page.coming.some((event) => event.training) && !page.past.some((event) => event.training)}
			<p class="text-sm text-muted-foreground" data-testid="no-trainings">
				No training days or trainings this season. Add a weekly training in
				<a href={resolve('/settings')} class="font-medium underline underline-offset-4">Settings</a
				>.
			</p>
		{/if}

		{#if empty}
			<p class="text-sm text-muted-foreground" data-testid="season-empty">
				No games or trainings in {page.season} yet.
			</p>
		{:else}
			<Labels bind:labels />
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
			{#each parts(labels) as part (part)}
				{@const events = visible(
					part === 'coming' ? page.coming : page.past,
					labels,
					showCancelled
				)}
				<section aria-labelledby="part-{part}" class="space-y-3" data-testid="part-{part}">
					<h2 id="part-{part}" class="text-lg font-semibold tracking-tight">
						{part === 'coming' ? 'Coming' : 'Past'}
					</h2>
					{#if events.length === 0}
						<p class="text-sm text-muted-foreground" data-testid="part-empty">
							{emptyLine(labels, part)}
						</p>
					{:else}
						<SeasonEvents {events} {part} {busy} onOpen={open} onCancel={cancel} />
					{/if}
				</section>
			{/each}
		{/if}
	</section>
{/if}
