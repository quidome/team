<!--
	One part of the page "Season" ("Coming" or "Past"): the events under a heading per week and per
	date. Game rows look as in the schedule; a tap on a training row opens the training, and "Cancel"
	cancels a training day that is not opened.
-->
<script lang="ts">
	import type { EventView, TrainingRow } from '#lib/api/client.js';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import { formatDay } from '#lib/schedule.js';
	import { groupByWeek, weekHeading } from '#lib/season.js';
	import { statusText } from '#lib/trainings.js';
	import MapPin from '@lucide/svelte/icons/map-pin';

	let {
		events,
		part,
		busy,
		onOpen,
		onCancel
	}: {
		events: EventView[];
		part: 'coming' | 'past';
		busy: boolean;
		onOpen: (row: TrainingRow) => void;
		/** "Cancel" on a training day that is not opened. */
		onCancel: (row: TrainingRow) => void;
	} = $props();

	const weeks = $derived(groupByWeek(events));
</script>

<div class="space-y-6">
	{#each weeks as { week, days } (week.week_first_day)}
		<section aria-labelledby="{part}-week-{week.week_first_day}" class="space-y-3">
			<h3
				id="{part}-week-{week.week_first_day}"
				class="text-base font-semibold tracking-tight"
				data-testid="week"
			>
				{weekHeading(week)}
			</h3>
			{#each days as day (day.date)}
				<section aria-labelledby="{part}-day-{day.date}">
					<h4
						id="{part}-day-{day.date}"
						class="sticky top-[calc(3.5rem+env(safe-area-inset-top))] z-[1] -mx-4 bg-background/95 px-4 py-2 text-sm font-semibold text-muted-foreground backdrop-blur"
						data-testid="day"
					>
						{formatDay(day.date)}
					</h4>
					<ul class="space-y-2">
						{#each day.events as event (event.game ? `g${event.game.id}` : `t${event.date}`)}
							{#if event.game}
								{@const game = event.game}
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
										<p
											class={['text-sm leading-snug', gone && 'text-muted-foreground line-through']}
										>
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
							{:else if event.training}
								{@const row = event.training}
								<li
									class="flex items-stretch gap-2 rounded-xl border bg-card shadow-xs"
									data-testid="training-row"
									data-date={row.date}
								>
									<button
										type="button"
										class="flex min-h-12 min-w-0 flex-1 items-center justify-between gap-3 px-4 py-2 text-left disabled:opacity-60"
										disabled={busy}
										onclick={() => onOpen(row)}
									>
										<span class="min-w-0 tabular-nums">
											<span class="sr-only">{formatDay(row.date)}</span>
											<span class="font-medium">Training</span>
											<span class="text-muted-foreground">· {row.start}</span>
											{#if row.extra}
												<span class="text-muted-foreground">· extra</span>
											{/if}
										</span>
										<span
											class="shrink-0 text-sm {row.status === 'not_opened' ||
											row.status === 'cancelled'
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
												onclick={() => onCancel(row)}
												aria-label="Cancel training of {formatDay(row.date)}"
											>
												Cancel
											</Button>
										</div>
									{/if}
								</li>
							{/if}
						{/each}
					</ul>
				</section>
			{/each}
		</section>
	{/each}
</div>
