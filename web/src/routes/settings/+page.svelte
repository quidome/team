<!--
	Settings (attendance at training, story A): the weekly trainings of the season (a day of the week
	with its times, in a window) and the reasons for an absence.
-->
<script lang="ts">
	import { resolve } from '$app/paths';
	import {
		ApiError,
		addReason,
		addWeeklyTraining,
		changeReason,
		changeWeeklyTraining,
		deleteWeeklyTraining,
		proposeWindow,
		type AbsenceReason,
		type WeeklyTraining,
		type WeeklyTrainingInput
	} from '#lib/api/client.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import { weekdays } from '#lib/players.js';
	import { formatDay } from '#lib/schedule.js';
	import { untrack } from 'svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	function message(failure: unknown): string {
		return failure instanceof ApiError
			? failure.message
			: 'Something went wrong. Please try again.';
	}

	// Weekly trainings
	let weekly = $state<WeeklyTraining[]>(
		untrack(() => structuredClone(data.weekly?.weekly_trainings ?? []))
	);
	/** The weekly training in the form: `null` for a new one; the form is closed when `editing` is false. */
	let editing = $state(false);
	let editingId = $state<number | null>(null);
	let form = $state<WeeklyTrainingInput>({
		weekday: 2,
		start: '',
		end: '',
		from: '',
		until: ''
	});
	/** The window is the proposal, not typed: a change of the day proposes again. */
	let windowProposed = $state(false);
	let savingWeekly = $state(false);
	let weeklyError = $state<string | null>(null);

	function dayName(weekday: number): string {
		return weekdays.find((candidate) => candidate.value === weekday)?.label ?? '';
	}

	async function propose() {
		if (form.from && form.until && !windowProposed) return;
		try {
			const proposal = await proposeWindow(form.weekday);
			form.from = proposal.from;
			form.until = proposal.until;
			windowProposed = true;
		} catch {
			// The dates can still be typed.
		}
	}

	function startAdding() {
		const used = new Set(weekly.map((training) => training.weekday));
		const weekday = weekdays.find((candidate) => !used.has(candidate.value))?.value ?? 1;
		form = { weekday, start: '', end: '', from: '', until: '' };
		editingId = null;
		windowProposed = false;
		weeklyError = null;
		editing = true;
		void propose();
	}

	function startChanging(training: WeeklyTraining) {
		const { weekday, start, end, from, until } = training;
		form = { weekday, start, end, from, until };
		editingId = training.id;
		windowProposed = false;
		weeklyError = null;
		editing = true;
	}

	async function runWeekly(action: () => Promise<{ weekly_trainings: WeeklyTraining[] }>) {
		savingWeekly = true;
		weeklyError = null;
		try {
			weekly = (await action()).weekly_trainings;
			return true;
		} catch (failure) {
			weeklyError = message(failure);
			return false;
		} finally {
			savingWeekly = false;
		}
	}

	async function saveWeekly(event: SubmitEvent) {
		event.preventDefault();
		const input = { ...form };
		const id = editingId;
		if (
			await runWeekly(() =>
				id === null ? addWeeklyTraining(input) : changeWeeklyTraining(id, input)
			)
		) {
			editing = false;
		}
	}

	function removeWeekly(training: WeeklyTraining) {
		if (!confirm(`Delete the weekly training on ${dayName(training.weekday)}?`)) return;
		void runWeekly(() => deleteWeeklyTraining(training.id));
	}

	// Reasons
	let reasons = $state<AbsenceReason[]>(untrack(() => structuredClone(data.reasons ?? [])));
	let names = $state<Record<number, string>>(
		untrack(() =>
			Object.fromEntries((data.reasons ?? []).map((reason) => [reason.id, reason.name]))
		)
	);
	let newName = $state('');
	let newValid = $state(false);
	let reasonsError = $state<string | null>(null);
	let savingReason = $state(false);

	function show(list: AbsenceReason[]) {
		reasons = list;
		names = Object.fromEntries(list.map((reason) => [reason.id, reason.name]));
	}

	async function updateReason(reason: AbsenceReason, change: Partial<AbsenceReason>) {
		savingReason = true;
		reasonsError = null;
		try {
			show(await changeReason({ ...reason, name: names[reason.id] ?? reason.name, ...change }));
		} catch (failure) {
			reasonsError = message(failure);
		} finally {
			savingReason = false;
		}
	}

	async function add(event: SubmitEvent) {
		event.preventDefault();
		savingReason = true;
		reasonsError = null;
		try {
			show(await addReason(newName, newValid));
			newName = '';
			newValid = false;
		} catch (failure) {
			reasonsError = message(failure);
		} finally {
			savingReason = false;
		}
	}

	const field =
		'h-11 w-full min-w-0 rounded-md border border-input bg-background px-3 text-base shadow-xs focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none';
</script>

<svelte:head>
	<title>Settings · Team</title>
</svelte:head>

{#if !data.weekly || !data.reasons}
	<section class="py-16 text-center">
		<p class="text-muted-foreground">Log in to see the settings.</p>
		<Button href="/auth/login" class="mt-4" data-sveltekit-reload>Log in</Button>
	</section>
{:else}
	<section class="space-y-6">
		<div class="flex items-center justify-between gap-3">
			<h1 class="text-2xl font-semibold tracking-tight">Settings</h1>
			<Button href={resolve('/')} variant="ghost">Back</Button>
		</div>

		<section
			class="space-y-4 rounded-xl border bg-card p-4 shadow-xs"
			aria-labelledby="weekly-heading"
		>
			<div>
				<h2 id="weekly-heading" class="font-medium">Weekly trainings · {data.weekly.season}</h2>
				<p class="text-sm text-muted-foreground">
					A weekly training suggests a training on each of its days. A change corrects it for its
					whole window; trainings that are opened keep their own date and times.
				</p>
			</div>
			{#if weeklyError}
				<p class="text-sm" role="alert">{weeklyError}</p>
			{/if}
			{#if weekly.length === 0}
				<p class="text-sm text-muted-foreground">No weekly trainings yet.</p>
			{:else}
				<ul class="space-y-2">
					{#each weekly as training (training.id)}
						<li class="space-y-2 rounded-lg border p-3" data-testid="weekly-training">
							<p class="text-sm tabular-nums">
								<span class="font-medium"
									>{dayName(training.weekday)} {training.start}–{training.end}</span
								>
								· {formatDay(training.from)} – {formatDay(training.until)}
							</p>
							<div class="flex gap-2">
								<Button
									type="button"
									variant="outline"
									size="sm"
									disabled={savingWeekly}
									onclick={() => startChanging(training)}
								>
									Change
								</Button>
								<Button
									type="button"
									variant="ghost"
									size="sm"
									disabled={savingWeekly}
									onclick={() => removeWeekly(training)}
								>
									Delete
								</Button>
							</div>
						</li>
					{/each}
				</ul>
			{/if}

			{#if editing}
				<form
					class="space-y-3 rounded-lg border p-3"
					onsubmit={saveWeekly}
					data-testid="weekly-form"
				>
					<label class="block space-y-1">
						<span class="text-sm">Day</span>
						<select class={field} bind:value={form.weekday} onchange={() => void propose()}>
							{#each weekdays as weekday (weekday.value)}
								<option value={weekday.value}>{weekday.label}</option>
							{/each}
						</select>
					</label>
					<div class="flex gap-2">
						<label class="block min-w-0 flex-1 space-y-1">
							<span class="text-sm">Start</span>
							<input class={field} type="time" bind:value={form.start} required />
						</label>
						<label class="block min-w-0 flex-1 space-y-1">
							<span class="text-sm">End</span>
							<input class={field} type="time" bind:value={form.end} required />
						</label>
					</div>
					<div class="grid gap-2 sm:grid-cols-2">
						<label class="block min-w-0 space-y-1">
							<span class="text-sm">First training</span>
							<input
								class={field}
								type="date"
								bind:value={form.from}
								min={data.weekly.season_first_day}
								max={data.weekly.season_last_day}
								oninput={() => (windowProposed = false)}
								required
							/>
						</label>
						<label class="block min-w-0 space-y-1">
							<span class="text-sm">Last training</span>
							<input
								class={field}
								type="date"
								bind:value={form.until}
								min={data.weekly.season_first_day}
								max={data.weekly.season_last_day}
								oninput={() => (windowProposed = false)}
								required
							/>
						</label>
					</div>
					<div class="flex gap-2">
						<Button type="submit" disabled={savingWeekly}>Save weekly training</Button>
						<Button type="button" variant="ghost" onclick={() => (editing = false)}>Cancel</Button>
					</div>
				</form>
			{:else}
				<Button type="button" variant="outline" onclick={startAdding}>Add weekly training</Button>
			{/if}
		</section>

		<section class="space-y-4 rounded-xl border bg-card p-4 shadow-xs" aria-labelledby="reasons">
			<div>
				<h2 id="reasons" class="font-medium">Reasons for an absence</h2>
				<p class="text-sm text-muted-foreground">
					A valid reason (for example school) counts apart in the totals.
				</p>
			</div>
			{#if reasonsError}
				<p class="text-sm" role="alert">{reasonsError}</p>
			{/if}
			<ul class="space-y-2">
				{#each reasons as reason (reason.id)}
					<li
						class={[
							'space-y-2 rounded-lg border p-3',
							reason.hidden && 'border-dashed bg-muted/40'
						]}
						data-testid="reason"
						data-reason={reason.name}
					>
						<div class="flex gap-2">
							<input class={field} aria-label="Name of the reason" bind:value={names[reason.id]} />
							{#if names[reason.id] !== reason.name}
								<Button
									type="button"
									variant="outline"
									disabled={savingReason}
									onclick={() => updateReason(reason, {})}
								>
									Save
								</Button>
							{/if}
						</div>
						<div class="flex flex-wrap items-center gap-4">
							<label class="flex min-h-11 items-center gap-2 text-sm">
								<input
									type="checkbox"
									checked={reason.valid}
									disabled={savingReason}
									onchange={(event) => updateReason(reason, { valid: event.currentTarget.checked })}
								/>
								Valid
							</label>
							<label class="flex min-h-11 items-center gap-2 text-sm">
								<input
									type="checkbox"
									checked={reason.hidden}
									disabled={savingReason}
									onchange={(event) =>
										updateReason(reason, { hidden: event.currentTarget.checked })}
								/>
								Hidden
							</label>
						</div>
					</li>
				{/each}
			</ul>
			<form class="space-y-2" onsubmit={add}>
				<label class="block space-y-1">
					<span class="text-sm">New reason</span>
					<input class={field} bind:value={newName} autocomplete="off" required />
				</label>
				<label class="flex min-h-11 items-center gap-2 text-sm">
					<input type="checkbox" bind:checked={newValid} />
					Valid
				</label>
				<Button type="submit" variant="outline" disabled={savingReason}>Add reason</Button>
			</form>
		</section>
	</section>
{/if}
