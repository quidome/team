<!--
	One training (attendance at training, story B): the date and times, "N absent", "All there" or
	"Cancelled", and the players with a role on the date, each "there" at first. A tap on a player
	opens the reasons ("There" on top for an absent player); a tap on a reason saves at once. The
	taps of one player are sent one after the other; a tap that fails shows the stored state again
	with "Not saved. Try again." Below: "Change", "Cancel training" or "Undo cancel", "Delete
	training". A date that is not opened shows "Open training".
-->
<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import {
		ApiError,
		cancelTraining,
		changeTraining,
		deleteTraining,
		loadTraining,
		markAbsent,
		markThere,
		openTraining,
		uncancelTraining,
		type TrainingDay
	} from '#lib/api/client.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import { fullName } from '#lib/players.js';
	import { formatDay } from '#lib/schedule.js';
	import { onMount } from 'svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	// Replaced by the answer of each action, and again when the page is shown again.
	let day = $derived(data.day);

	type Marks = Record<number, number | null>;
	function marksOf(page: TrainingDay | null): Marks {
		return Object.fromEntries((page?.players ?? []).map((p) => [p.id, p.reason ?? null]));
	}
	/** The marks the server stored. */
	let stored = $derived(marksOf(day));
	/** The marks on the screen: a tap shows at once, before its answer. */
	let shown = $derived(marksOf(day));

	/** The player whose reasons are open. */
	let picking = $state<number | null>(null);
	// Not shown on the screen, so not reactive.
	/** For each player, the request that runs now; the next tap waits for it. */
	const queues: Record<number, Promise<void>> = {};
	/** For each player, the number of its newest tap. */
	const newest: Record<number, number> = {};
	let waiting = $state(0);
	let notSaved = $state(false);

	let changing = $state(false);
	let changeDate = $state('');
	let changeStart = $state('');
	let changeEnd = $state('');

	let busy = $state(false);
	let error = $state<string | null>(null);

	function message(failure: unknown): string {
		return failure instanceof ApiError
			? failure.message
			: 'Something went wrong. Please try again.';
	}

	/** A tap on a reason (or on "There" with `null`): shown at once, saved in the player's queue. */
	function mark(player: number, reason: number | null) {
		const page = day;
		if (!page) return;
		picking = null;
		notSaved = false;
		shown = { ...shown, [player]: reason };
		const tap = (newest[player] ?? 0) + 1;
		newest[player] = tap;
		waiting += 1;
		const before = queues[player] ?? Promise.resolve();
		const request = before.then(async () => {
			try {
				const answer =
					reason === null
						? await markThere(page.date, player)
						: await markAbsent(page.date, player, reason);
				stored = { ...stored, [player]: answer.reason ?? null };
				if (newest[player] === tap) shown = { ...shown, [player]: answer.reason ?? null };
			} catch {
				// The newest tap decides what the row shows; an older failed tap only reports.
				if (newest[player] === tap) shown = { ...shown, [player]: stored[player] ?? null };
				notSaved = true;
			} finally {
				waiting -= 1;
			}
		});
		queues[player] = request;
	}

	/** Shown again (the phone comes back to the page): the stored marks, unless a tap is waiting. */
	onMount(() => {
		const visible = async () => {
			if (document.visibilityState !== 'visible' || waiting > 0 || !day) return;
			try {
				day = await loadTraining(fetch, day.date);
			} catch {
				// The page keeps what it shows.
			}
		};
		document.addEventListener('visibilitychange', visible);
		return () => document.removeEventListener('visibilitychange', visible);
	});

	async function run(work: () => Promise<TrainingDay>) {
		busy = true;
		error = null;
		try {
			day = await work();
		} catch (failure) {
			error = message(failure);
		} finally {
			busy = false;
		}
	}

	function startChanging(page: TrainingDay) {
		changing = true;
		changeDate = page.date;
		changeStart = page.training?.start ?? '';
		changeEnd = page.training?.end ?? '';
		error = null;
	}

	async function change(event: SubmitEvent) {
		event.preventDefault();
		const page = day;
		if (!page) return;
		busy = true;
		error = null;
		try {
			const moved = await changeTraining(page.date, {
				date: changeDate,
				start: changeStart,
				end: changeEnd
			});
			changing = false;
			if (moved.date === page.date) day = moved;
			else await goto(resolve('/training/[date]', { date: moved.date }), { replaceState: true });
		} catch (failure) {
			error = message(failure);
		} finally {
			busy = false;
		}
	}

	async function remove() {
		const page = day;
		if (!page) return;
		if (!confirm(`Delete the training of ${formatDay(page.date)}? Its attendance is removed.`))
			return;
		busy = true;
		error = null;
		try {
			await deleteTraining(page.date);
			await goto(resolve('/attendance'));
		} catch (failure) {
			error = message(failure);
			busy = false;
		}
	}

	function reasonName(page: TrainingDay, id: number): string {
		return page.reasons.find((reason) => reason.id === id)?.name ?? '';
	}

	/** "Tue 6 Oct · 19:45–21:00", or only the date while not opened. */
	function heading(page: TrainingDay): string {
		const date = formatDay(page.date);
		return page.training ? `${date} · ${page.training.start}–${page.training.end}` : date;
	}

	function topLine(page: TrainingDay, marks: Marks): string {
		if (!page.training) return 'Not opened';
		if (page.training.cancelled) return 'Cancelled';
		const absent = Object.values(marks).filter((reason) => reason !== null).length;
		return absent === 0 ? 'All there' : `${absent} absent`;
	}

	const field =
		'h-11 w-full min-w-0 rounded-md border border-input bg-background px-3 text-base shadow-xs focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none';
</script>

<svelte:head>
	<title>Training · Team</title>
</svelte:head>

{#if !day}
	<section class="py-16 text-center">
		<p class="text-muted-foreground">Log in to see the training.</p>
		<Button href="/auth/login" class="mt-4" data-sveltekit-reload>Log in</Button>
	</section>
{:else}
	{@const page = day}
	{@const offered = page.reasons.filter((reason) => !reason.hidden)}
	<section class="space-y-6">
		<div class="flex items-start justify-between gap-3">
			<div class="min-w-0">
				<h1 class="text-2xl font-semibold tracking-tight" data-testid="training-heading">
					{heading(page)}
				</h1>
				<p class="mt-1 text-sm text-muted-foreground" data-testid="training-top">
					{topLine(page, shown)}
				</p>
			</div>
			<Button href={resolve('/attendance')} variant="ghost">Back</Button>
		</div>

		{#if notSaved}
			<p class="text-sm" role="alert" data-testid="not-saved">Not saved. Try again.</p>
		{/if}
		{#if error && !changing}
			<p class="text-sm" role="alert">{error}</p>
		{/if}

		{#if !page.training}
			<div class="space-y-3 rounded-xl border bg-card p-4 shadow-xs">
				<p class="text-sm">
					This training is not opened. Opening it makes it count, with every player there.
				</p>
				<Button disabled={busy} onclick={() => run(() => openTraining(page.date))}>
					Open training
				</Button>
			</div>
		{:else}
			{#if page.training.cancelled}
				<div
					class="flex items-center justify-between gap-3 rounded-xl border bg-card p-4 shadow-xs"
				>
					<p class="text-sm font-medium">Cancelled</p>
					<Button
						variant="outline"
						disabled={busy}
						onclick={() => run(() => uncancelTraining(page.date))}
					>
						Undo cancel
					</Button>
				</div>
			{/if}

			{#if page.players.length === 0}
				<p class="text-sm text-muted-foreground" data-testid="no-players">
					No player has a role on this date.
				</p>
			{:else}
				{#if offered.length === 0}
					<p class="text-sm text-muted-foreground" data-testid="no-reasons">
						No reasons. Add one in
						<a href={resolve('/settings')} class="font-medium underline underline-offset-4"
							>Reasons</a
						>.
					</p>
				{/if}
				<ul class="space-y-2" aria-label="Players">
					{#each page.players as player (player.id)}
						{@const reason = shown[player.id] ?? null}
						<li class="rounded-xl border bg-card shadow-xs" data-testid="training-player">
							<button
								type="button"
								class="flex min-h-12 w-full items-center justify-between gap-3 px-4 py-2 text-left"
								aria-expanded={picking === player.id}
								onclick={() => (picking = picking === player.id ? null : player.id)}
							>
								<span class="min-w-0 truncate font-medium">{fullName(player)}</span>
								<span
									class="shrink-0 text-sm {reason === null
										? 'text-muted-foreground'
										: 'font-medium'}"
									data-testid="player-mark"
								>
									{reason === null ? 'there' : reasonName(page, reason)}
								</span>
							</button>
							{#if picking === player.id}
								<div
									class="flex flex-wrap gap-2 border-t px-4 pt-3 pb-4"
									data-testid="reasons"
									role="group"
									aria-label="Reasons for {fullName(player)}"
								>
									{#if reason !== null}
										<Button variant="secondary" onclick={() => mark(player.id, null)}>There</Button>
									{/if}
									{#each offered as option (option.id)}
										<Button
											variant={option.id === reason ? 'default' : 'outline'}
											onclick={() => mark(player.id, option.id)}
										>
											{option.name}
										</Button>
									{/each}
								</div>
							{/if}
						</li>
					{/each}
				</ul>
			{/if}

			{#if changing}
				<form
					class="space-y-3 rounded-xl border bg-card p-4 shadow-xs"
					onsubmit={change}
					data-testid="change-training"
				>
					<h2 class="font-medium">Change</h2>
					<label class="block space-y-1">
						<span class="text-sm">Date</span>
						<input class={field} type="date" bind:value={changeDate} required />
					</label>
					<div class="grid grid-cols-2 gap-2">
						<label class="block min-w-0 space-y-1">
							<span class="text-sm">Start</span>
							<input class={field} type="time" bind:value={changeStart} required />
						</label>
						<label class="block min-w-0 space-y-1">
							<span class="text-sm">End</span>
							<input class={field} type="time" bind:value={changeEnd} required />
						</label>
					</div>
					{#if error}
						<p class="text-sm" role="alert">{error}</p>
					{/if}
					<div class="flex gap-2">
						<Button type="submit" disabled={busy}>Save</Button>
						<Button type="button" variant="ghost" onclick={() => (changing = false)}>Cancel</Button>
					</div>
				</form>
			{/if}

			<div class="flex flex-wrap gap-2">
				{#if !changing}
					<Button variant="outline" disabled={busy} onclick={() => startChanging(page)}>
						Change
					</Button>
				{/if}
				{#if !page.training.cancelled}
					<Button
						variant="outline"
						disabled={busy}
						onclick={() => run(() => cancelTraining(page.date))}
					>
						Cancel training
					</Button>
				{/if}
				<Button variant="ghost" class="text-destructive" disabled={busy} onclick={remove}>
					Delete training
				</Button>
			</div>
		{/if}
	</section>
{/if}
