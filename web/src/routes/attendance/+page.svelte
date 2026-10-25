<!--
	Attendance (attendance at training, story C): the totals of each player in the chosen season (the
	current one by default, `?season=` in the address), then "Left the team"; a tap on a player shows
	the trainings it missed. Above the totals: the training days not opened. The list of trainings
	is on "Season".
-->
<script lang="ts">
	import { afterNavigate, goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import type { PlayerTotals } from '#lib/api/client.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import { formatDay } from '#lib/schedule.js';
	import { fullName } from '#lib/players.js';
	import { countsText, notOpenedText, ofTrainings } from '#lib/trainings.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const overview = $derived(data.overview);

	// The players whose missed trainings are shown.
	let expanded = $state<number[]>([]);
	function toggle(player: number) {
		expanded = expanded.includes(player)
			? expanded.filter((id) => id !== player)
			: [...expanded, player];
	}

	// Another season (or a return to the page) starts with no open missed list.
	afterNavigate(() => {
		expanded = [];
	});

	function chooseSeason(season: string) {
		goto(resolve(`/attendance?season=${encodeURIComponent(season)}`));
	}
</script>

<svelte:head>
	<title>Attendance · Team</title>
</svelte:head>

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
{:else if !overview}
	<section class="py-16 text-center">
		<p class="text-muted-foreground">Log in to see the trainings.</p>
		<Button href="/auth/login" class="mt-4" data-sveltekit-reload>Log in</Button>
	</section>
{:else}
	{@const current = overview.season === overview.current_season}
	<section class="space-y-6">
		<div class="flex items-center justify-between gap-3">
			<h1 class="text-2xl font-semibold tracking-tight">Attendance · {overview.season}</h1>
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
				<!-- eslint-disable svelte/no-navigation-without-resolve -- the path is resolved; only the query `from` is added (resolve() takes no query on a route with a parameter) -->
				<a
					href={`${resolve('/training/[date]', { date: overview.first_not_opened })}?from=attendance`}
					class="font-medium underline underline-offset-4"
				>
					{notOpenedText(overview.not_opened)}
				</a>
				<!-- eslint-enable svelte/no-navigation-without-resolve -->
			</p>
		{/if}

		<section aria-labelledby="totals-heading" class="space-y-2">
			<h2 id="totals-heading" class="text-lg font-semibold tracking-tight">Totals</h2>
			{#if overview.trainings === 0}
				<p class="text-sm text-muted-foreground" data-testid="no-trainings-yet">
					No trainings yet this season.
				</p>
			{:else}
				{#if overview.team.length === 0 && overview.left.length === 0}
					<p class="text-sm text-muted-foreground" data-testid="no-player-totals">
						No player had a role at these trainings.
						{#if current}
							Add players in
							<a href={resolve('/team')} class="font-medium underline underline-offset-4">Team</a>.
						{/if}
					</p>
				{/if}
				{@render totals(overview.team, 'team-totals')}
				{#if overview.left.length > 0}
					<h3 class="pt-2 font-semibold">Left the team</h3>
					{@render totals(overview.left, 'left-totals')}
				{/if}
			{/if}
		</section>
	</section>
{/if}
