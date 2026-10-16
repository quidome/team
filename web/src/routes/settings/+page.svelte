<!--
	Settings (attendance at training, story A): the training days of the season with their times, the
	first and the last training date, and the reasons for an absence.
-->
<script lang="ts">
	import { resolve } from '$app/paths';
	import {
		ApiError,
		addReason,
		changeReason,
		proposeTrainingDates,
		saveTrainingSettings,
		type AbsenceReason,
		type TrainingDay
	} from '#lib/api/client.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import { weekdays } from '#lib/players.js';
	import { untrack } from 'svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	// Training days
	let days = $state<TrainingDay[]>(untrack(() => structuredClone(data.settings?.days ?? [])));
	let first = $state(untrack(() => data.settings?.first ?? ''));
	let last = $state(untrack(() => data.settings?.last ?? ''));
	let savingDays = $state(false);
	let daysError = $state<string | null>(null);
	let daysSaved = $state(false);
	/** The dates are the proposal, not typed: a change of days proposes again. */
	let proposed = $state(false);

	function message(failure: unknown): string {
		return failure instanceof ApiError
			? failure.message
			: 'Something went wrong. Please try again.';
	}

	function addDay() {
		const used = new Set(days.map((day) => day.weekday));
		const weekday = weekdays.find((candidate) => !used.has(candidate.value))?.value ?? 1;
		days.push({ weekday, start: '', end: '' });
		daysSaved = false;
		void propose();
	}

	function removeDay(index: number) {
		days.splice(index, 1);
		daysSaved = false;
		void propose();
	}

	/** Before the first save, fill the dates with the proposal for the chosen days. */
	async function propose() {
		if (data.settings?.saved || days.length === 0) return;
		if ((first || last) && !proposed) return;
		try {
			const proposal = await proposeTrainingDates(days.map((day) => day.weekday));
			first = proposal.first;
			last = proposal.last;
			proposed = true;
		} catch {
			// The dates can still be typed.
		}
	}

	async function saveDays(event: SubmitEvent) {
		event.preventDefault();
		savingDays = true;
		daysError = null;
		daysSaved = false;
		try {
			const saved = await saveTrainingSettings(first, last, days);
			days = structuredClone(saved.days);
			first = saved.first ?? first;
			last = saved.last ?? last;
			// Saved dates are not a proposal: a later change of days keeps them.
			proposed = false;
			daysSaved = true;
		} catch (failure) {
			daysError = message(failure);
		} finally {
			savingDays = false;
		}
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

{#if !data.settings || !data.reasons}
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

		<form
			class="space-y-4 rounded-xl border bg-card p-4 shadow-xs"
			onsubmit={saveDays}
			aria-labelledby="training-heading"
		>
			<div>
				<h2 id="training-heading" class="font-medium">Training days · {data.settings.season}</h2>
				<p class="text-sm text-muted-foreground">
					A change applies from today; earlier trainings keep their days.
				</p>
			</div>

			{#each days as day, index (index)}
				<div class="space-y-2 rounded-lg border p-3" data-testid="training-day">
					<label class="block space-y-1">
						<span class="text-sm">Day</span>
						<select
							class={field}
							bind:value={day.weekday}
							onchange={() => {
								daysSaved = false;
								void propose();
							}}
						>
							{#each weekdays as weekday (weekday.value)}
								<option value={weekday.value}>{weekday.label}</option>
							{/each}
						</select>
					</label>
					<div class="flex gap-2">
						<label class="block min-w-0 flex-1 space-y-1">
							<span class="text-sm">Start</span>
							<input class={field} type="time" bind:value={day.start} required />
						</label>
						<label class="block min-w-0 flex-1 space-y-1">
							<span class="text-sm">End</span>
							<input class={field} type="time" bind:value={day.end} required />
						</label>
					</div>
					<Button type="button" variant="ghost" size="sm" onclick={() => removeDay(index)}>
						Remove day
					</Button>
				</div>
			{/each}
			<Button type="button" variant="outline" onclick={addDay}>Add training day</Button>

			<div class="grid gap-2 sm:grid-cols-2">
				<label class="block min-w-0 space-y-1">
					<span class="text-sm">First training</span>
					<input
						class={field}
						type="date"
						bind:value={first}
						oninput={() => (proposed = false)}
						required
					/>
				</label>
				<label class="block min-w-0 space-y-1">
					<span class="text-sm">Last training</span>
					<input
						class={field}
						type="date"
						bind:value={last}
						oninput={() => (proposed = false)}
						required
					/>
				</label>
			</div>

			{#if daysError}
				<p class="text-sm" role="alert">{daysError}</p>
			{/if}
			{#if daysSaved}
				<p class="text-sm" role="status" data-testid="days-saved">Saved.</p>
			{/if}
			<Button type="submit" disabled={savingDays}>Save training days</Button>
		</form>

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
