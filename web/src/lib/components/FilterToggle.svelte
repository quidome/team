<!--
	The quick filter of the schedule and the list of changes: all, games of the team, possible duty
	games. There is always one choice.
-->
<script lang="ts">
	import * as ToggleGroup from '#lib/components/ui/toggle-group/index.js';
	import { filters, type Filter } from '#lib/schedule.js';

	let { filter = $bindable(), label }: { filter: Filter; label: string } = $props();
</script>

<ToggleGroup.Root
	type="single"
	variant="outline"
	class="grid w-full grid-cols-3"
	aria-label={label}
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
