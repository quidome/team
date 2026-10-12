<script lang="ts">
	import { invalidateAll } from '$app/navigation';
	import { resolve } from '$app/paths';
	import ChooseTeam from '#lib/components/ChooseTeam.svelte';
	import ReadGameList from '#lib/components/ReadGameList.svelte';
	import { Button } from '#lib/components/ui/button/index.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
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
	</section>
{:else if data.selection?.state === 'chosen'}
	<section class="space-y-6">
		<div class="rounded-xl border bg-card p-5 shadow-xs">
			<p class="text-sm text-muted-foreground">Your team · {data.selection.season}</p>
			<h1 class="mt-1 text-2xl font-semibold tracking-tight" data-testid="chosen-team">
				{data.selection.club.name}
				{data.selection.team.name}
			</h1>
			{#if data.selection.club.town}
				<p class="text-sm text-muted-foreground">{data.selection.club.town}</p>
			{/if}
			<Button href={resolve('/choose')} variant="outline" class="mt-4">Change</Button>
		</div>
		{#if data.readings}
			<div class="rounded-xl border bg-card p-5 shadow-xs">
				<h2 class="mb-3 font-medium">Game list</h2>
				{#key data.readings}
					<ReadGameList readings={data.readings} />
				{/key}
			</div>
		{/if}
		<div class="rounded-xl border bg-card p-5 shadow-xs">
			<h2 class="font-medium">Your schedule comes here</h2>
			<p class="mt-1 text-sm text-muted-foreground">
				Soon the tool shows your team's games. Until then, a sample shows how the schedule will
				look.
			</p>
			<Button href={resolve('/sample')} class="mt-4">See the sample schedule</Button>
		</div>
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
