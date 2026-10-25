<!--
	One part of the page "Season" ("Coming" or "Past"): the events under a heading per week and per
	date, each row an `EventRow`.
-->
<script lang="ts">
	import type { EventView, TrainingRow } from '#lib/api/client.js';
	import EventRow from '#lib/components/EventRow.svelte';
	import { formatDay } from '#lib/schedule.js';
	import { groupByWeek, weekHeading } from '#lib/season.js';

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
							<EventRow {event} {busy} {onOpen} {onCancel} />
						{/each}
					</ul>
				</section>
			{/each}
		</section>
	{/each}
</div>
