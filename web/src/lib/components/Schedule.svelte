<!--
	The coordinator's schedule (story 3b): the quick filter, the coming games grouped by date, and
	the past games on request. A game no longer in the game list is crossed out.
-->
<script lang="ts">
	import { untrack } from 'svelte';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import * as ToggleGroup from '#lib/components/ui/toggle-group/index.js';
	import type { Schedule } from '#lib/api/client.js';
	import {
		filterGames,
		filters,
		formatDay,
		groupByDate,
		noGamesLine,
		type Filter
	} from '#lib/schedule.js';
	import MapPin from '@lucide/svelte/icons/map-pin';

	let {
		schedule,
		hasReading,
		pastOpen = false
	}: {
		schedule: Schedule;
		/** False: the game list was never read for this schedule. */
		hasReading: boolean;
		/** Show the past games from the start. */
		pastOpen?: boolean;
	} = $props();

	let filter = $state<Filter>('all');
	let showPast = $state(untrack(() => pastOpen));
	const coming = $derived(groupByDate(filterGames(schedule.coming, filter)));
	const past = $derived(groupByDate(filterGames(schedule.past, filter)));
	const empty = $derived(schedule.coming.length === 0 && schedule.past.length === 0);
</script>

{#snippet games(groups: ReturnType<typeof groupByDate>)}
	<div class="space-y-6">
		{#each groups as group (group.date)}
			<section aria-labelledby="day-{group.date}">
				<h3
					id="day-{group.date}"
					class="sticky top-[calc(3.5rem+env(safe-area-inset-top))] z-[1] -mx-4 bg-background/95 px-4 py-2 text-sm font-semibold text-muted-foreground backdrop-blur"
					data-testid="day"
				>
					{formatDay(group.date)}
				</h3>
				<ul class="space-y-2">
					{#each group.games as game (game.id)}
						{@const ofTeam = game.reason === 'team'}
						{@const gone = !game.in_list}
						<li
							class={[
								'flex gap-3 rounded-xl border bg-card p-3 shadow-xs',
								ofTeam && 'border-l-4 border-l-team bg-team-soft',
								gone && 'bg-muted shadow-none'
							]}
							data-testid="game"
							data-of-team={ofTeam}
						>
							<div class="w-14 shrink-0 pt-0.5 text-sm font-semibold tabular-nums">
								{#if game.start_time}
									{game.start_time}
								{:else}
									<span class="text-xs leading-tight font-normal text-muted-foreground italic">
										time not known yet
									</span>
								{/if}
							</div>
							<div class="min-w-0 flex-1">
								<p class={['text-sm leading-snug', gone && 'text-muted-foreground line-through']}>
									<span class="block font-medium">{game.home}</span>
									<span class="block font-medium">
										<span class="sr-only">against</span>
										{game.away}
									</span>
								</p>
								<p class="mt-1 flex items-center gap-1 text-xs text-muted-foreground">
									<MapPin class="size-3.5 shrink-0" aria-hidden="true" />
									<span class="truncate">{game.hall}, {game.town}</span>
								</p>
								{#if gone}
									<Badge variant="outline" class="mt-2 border-gone/40 text-gone">
										no longer in the game list
									</Badge>
								{/if}
							</div>
							{#if ofTeam}
								<Badge class="self-start bg-team text-primary-foreground">Team</Badge>
							{/if}
						</li>
					{/each}
				</ul>
			</section>
		{/each}
	</div>
{/snippet}

<div class="space-y-5">
	{#if empty}
		<p class="text-sm text-muted-foreground" data-testid="schedule-empty">
			{#if schedule.last_season}
				The tool has no games of {schedule.season}.
			{:else if hasReading}
				The game list has no games for this season yet.
			{:else}
				The game list has not been read yet.
			{/if}
		</p>
	{:else}
		<ToggleGroup.Root
			type="single"
			variant="outline"
			class="grid w-full grid-cols-3"
			aria-label="Filter games"
			bind:value={
				() => filter,
				(value) => {
					// A tap on the chosen filter keeps it; there is always one.
					if (value) filter = value as Filter;
				}
			}
		>
			{#each filters as option (option.value)}
				<ToggleGroup.Item
					value={option.value}
					class="h-auto min-h-10 px-2 py-2 text-xs leading-tight whitespace-normal data-[state=on]:border-primary data-[state=on]:bg-primary data-[state=on]:text-primary-foreground sm:text-sm"
				>
					{option.label}
				</ToggleGroup.Item>
			{/each}
		</ToggleGroup.Root>

		{#if coming.length === 0}
			<p class="text-sm text-muted-foreground" data-testid="no-coming-games">
				{noGamesLine(filter, 'coming')}
			</p>
		{:else}
			{@render games(coming)}
		{/if}

		{#if schedule.past.length > 0}
			<Button
				variant="outline"
				class="w-full"
				aria-expanded={showPast}
				onclick={() => (showPast = !showPast)}
			>
				{showPast ? 'Hide past games' : 'Show past games'}
			</Button>
			{#if showPast}
				<div data-testid="past-games">
					{#if past.length === 0}
						<p class="text-sm text-muted-foreground">{noGamesLine(filter, 'past')}</p>
					{:else}
						{@render games(past)}
					{/if}
				</div>
			{/if}
		{/if}
	{/if}
</div>
