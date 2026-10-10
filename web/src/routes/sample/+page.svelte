<script lang="ts">
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import * as ToggleGroup from '#lib/components/ui/toggle-group/index.js';
	import {
		filterGames,
		formatDay,
		groupByDate,
		sampleGames,
		type Filter
	} from '#lib/sample/games.js';
	import BellDot from '@lucide/svelte/icons/bell-dot';
	import Clock from '@lucide/svelte/icons/clock';
	import MapPin from '@lucide/svelte/icons/map-pin';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	let filter = $state<Filter>('all');
	const groups = $derived(groupByDate(filterGames(sampleGames, filter)));

	const filters: { value: Filter; label: string }[] = [
		{ value: 'all', label: 'All' },
		{ value: 'team', label: 'Games of the team' },
		{ value: 'duty', label: 'Possible duty games' }
	];
</script>

<svelte:head>
	<title>Sample schedule · Team</title>
</svelte:head>

{#if !data.me}
	<section class="py-16 text-center">
		<p class="text-muted-foreground">Log in to see the sample schedule.</p>
		<Button href="/auth/login" class="mt-4" data-sveltekit-reload>Log in</Button>
	</section>
{:else}
	<section class="space-y-5">
		<div>
			<div class="flex items-center gap-2">
				<h1 class="text-2xl font-semibold tracking-tight">Schedule</h1>
				<Badge variant="outline">Sample</Badge>
			</div>
			<p class="mt-1 text-sm text-muted-foreground">Example games, not real data.</p>
		</div>

		<div class="flex flex-wrap items-center justify-between gap-2 text-sm">
			<span class="flex items-center gap-1.5 text-muted-foreground">
				<Clock class="size-4" aria-hidden="true" />
				Last read: today 03:00
			</span>
			<span
				class="flex items-center gap-1.5 rounded-full bg-accent px-3 py-1 font-medium text-accent-foreground"
				data-testid="changes-not-seen"
			>
				<BellDot class="size-4" aria-hidden="true" />
				2 changes not seen
			</span>
		</div>

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

		<div class="space-y-6">
			{#each groups as group (group.date)}
				<section aria-labelledby="day-{group.date}">
					<h2
						id="day-{group.date}"
						class="sticky top-14 z-[1] -mx-4 bg-background/95 px-4 py-2 text-sm font-semibold text-muted-foreground backdrop-blur"
					>
						{formatDay(group.date)}
					</h2>
					<ul class="space-y-2">
						{#each group.games as game (game.id)}
							<li
								class={[
									'flex gap-3 rounded-xl border bg-card p-3 shadow-xs',
									game.ofTeam && 'border-l-4 border-l-team bg-team-soft',
									game.gone && 'bg-muted shadow-none'
								]}
								data-testid="game"
								data-of-team={game.ofTeam}
							>
								<div class="w-14 shrink-0 pt-0.5 text-sm font-semibold tabular-nums">
									{#if game.time}
										{game.time}
									{:else}
										<span class="text-xs leading-tight font-normal text-muted-foreground italic">
											time not known yet
										</span>
									{/if}
								</div>
								<div class="min-w-0 flex-1">
									<p
										class={[
											'text-sm leading-snug',
											game.gone && 'text-muted-foreground line-through'
										]}
									>
										<span class="block font-medium">{game.home}</span>
										<span class="block font-medium">
											<span class="text-muted-foreground" aria-hidden="true">–</span>
											<span class="sr-only">against</span>
											{game.away}
										</span>
									</p>
									<p class="mt-1 flex items-center gap-1 text-xs text-muted-foreground">
										<MapPin class="size-3.5 shrink-0" aria-hidden="true" />
										<span class="truncate">{game.hall}, {game.town}</span>
									</p>
									{#if game.gone}
										<Badge variant="outline" class="mt-2 border-gone/40 text-gone">
											no longer in the game list
										</Badge>
									{/if}
								</div>
								{#if game.ofTeam}
									<Badge class="self-start bg-team text-primary-foreground">Team</Badge>
								{/if}
							</li>
						{/each}
					</ul>
				</section>
			{/each}
		</div>
	</section>
{/if}
