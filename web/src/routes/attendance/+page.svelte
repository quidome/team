<!--
	Attendance (attendance at training, story B): every training day and training of the current
	season, "Coming" above "Past". A tap on a row opens the training (and so makes it exist).
	"Cancel training" on a day that is not opened, "Show cancelled", and "Add training". Story C adds
	the totals.
-->
<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import {
		ApiError,
		addTraining,
		cancelTraining,
		loadTraining,
		loadTrainings,
		openTraining,
		type TrainingRow
	} from '#lib/api/client.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import { formatDay } from '#lib/schedule.js';
	import { statusText } from '#lib/trainings.js';
	import { onMount, untrack } from 'svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	// Replaced after a cancel on this page.
	let list = $state(untrack(() => data.list));

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
			list = await loadTrainings(fetch);
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

{#if !list}
	<section class="py-16 text-center">
		<p class="text-muted-foreground">Log in to see the trainings.</p>
		<Button href="/auth/login" class="mt-4" data-sveltekit-reload>Log in</Button>
	</section>
{:else}
	{@const page = list}
	<section class="space-y-6">
		<div class="flex items-center justify-between gap-3">
			<h1 class="text-2xl font-semibold tracking-tight">Attendance · {page.season}</h1>
			<Button href={resolve('/')} variant="ghost">Back</Button>
		</div>

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
		{:else}
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
				<h2 id="coming-heading" class="text-lg font-semibold tracking-tight">Coming</h2>
				{@render rows(page.coming, 'coming')}
			</section>
		{/if}

		{#if page.past.length > 0}
			<section aria-labelledby="past-heading" class="space-y-2">
				<h2 id="past-heading" class="text-lg font-semibold tracking-tight">Past</h2>
				{@render rows(page.past, 'past')}
			</section>
		{/if}
	</section>
{/if}
