<script lang="ts">
	import { invalidateAll } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { loadSchedule } from '#lib/api/client.js';
	import ChooseTeam from '#lib/components/ChooseTeam.svelte';
	import ReadGameList from '#lib/components/ReadGameList.svelte';
	import Schedule from '#lib/components/Schedule.svelte';
	import { Button } from '#lib/components/ui/button/index.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	// Replaced after a reading on this page, without loading the page again: that would build the
	// "Game list" card again and lose its result line.
	let schedule = $derived(data.schedule);
	let hasReading = $derived(Boolean(data.readings?.last_good));

	async function refreshSchedule() {
		schedule = await loadSchedule(fetch);
		hasReading = true;
	}
</script>

<svelte:head>
	<title>Team</title>
</svelte:head>

{#if data.selection?.state === 'not_chosen'}
	<section class="space-y-5">
		<div>
			<h1 class="text-2xl font-semibold tracking-tight">Choose your club and team</h1>
			<p class="mt-1 text-sm text-muted-foreground">
				The tool reads your team's games from the game list.
			</p>
		</div>
		<ChooseTeam onSaved={invalidateAll} />
	</section>
{:else if data.selection?.state === 'team_needed'}
	<section class="space-y-5">
		<div>
			<h1 class="text-2xl font-semibold tracking-tight">Choose your team</h1>
			{#if data.selection.reason === 'new_season'}
				<p class="mt-1 text-sm" data-testid="new-season">
					A new season started: choose the team{data.selection.last?.team ? ' again' : ''}.
				</p>
			{/if}
			{#if data.selection.last?.team}
				<p class="mt-1 text-sm text-muted-foreground" data-testid="last-season">
					In {data.selection.last.season} the team was {data.selection.last.club.name}
					{data.selection.last.team.name}.
				</p>
			{/if}
		</div>
		<ChooseTeam club={data.selection.club} clubKept onSaved={invalidateAll} />
		{#if schedule?.last_season}
			<section aria-labelledby="last-season-heading" class="space-y-3 pt-4">
				<div>
					<h2 id="last-season-heading" class="text-lg font-semibold tracking-tight">
						Last season · {schedule.season}
					</h2>
					<p class="text-sm text-muted-foreground" data-testid="last-season-team">
						{schedule.club.name}
						{schedule.team.name}
					</p>
				</div>
				<Schedule {schedule} hasReading pastOpen />
			</section>
		{/if}
	</section>
{:else if data.selection?.state === 'chosen'}
	<section class="space-y-6">
		<div class="flex items-start justify-between gap-3">
			<div class="min-w-0">
				<p class="text-sm text-muted-foreground">Your team · {data.selection.season}</p>
				<h1 class="text-2xl font-semibold tracking-tight" data-testid="chosen-team">
					{data.selection.club.name}
					{data.selection.team.name}
				</h1>
			</div>
			<Button href={resolve('/choose')} variant="outline">Change</Button>
		</div>
		{#if data.readings}
			<div class="rounded-xl border bg-card p-5 shadow-xs">
				<h2 class="mb-3 font-medium">Game list</h2>
				{#key data.readings}
					<ReadGameList readings={data.readings} onRead={refreshSchedule} />
				{/key}
			</div>
		{/if}
		{#if schedule}
			<section aria-labelledby="schedule-heading" class="space-y-3">
				<h2 id="schedule-heading" class="text-lg font-semibold tracking-tight">Schedule</h2>
				<Schedule {schedule} {hasReading} />
			</section>
		{/if}
	</section>
{:else}
	<section class="flex min-h-[60dvh] flex-col items-center justify-center text-center">
		<span class="mb-5 size-12 rounded-full bg-primary" aria-hidden="true"></span>
		<h1 class="text-3xl font-semibold tracking-tight">Team</h1>
		<p class="mt-2 max-w-xs text-muted-foreground">
			Games, attendance and duties for your team, in one place.
		</p>
		<Button href="/auth/login" size="lg" class="mt-8 w-full max-w-xs" data-sveltekit-reload>
			Log in
		</Button>
	</section>
{/if}
