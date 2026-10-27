<!--
	The labels of the page "Season": "What" (games of the team, possible duty games, trainings),
	"When" (coming, past) and "Duties" (entered, not entered, open places). Several can be on; a
	group with none on shows all of that group.
-->
<script lang="ts">
	import * as ToggleGroup from '#lib/components/ui/toggle-group/index.js';
	import {
		dutyLabels,
		whatLabels,
		whenLabels,
		type Duties,
		type Labels,
		type What,
		type When
	} from '#lib/season.js';

	let { labels = $bindable() }: { labels: Labels } = $props();

	const item =
		'h-auto min-h-10 px-2 py-2 text-xs leading-tight whitespace-normal data-[state=on]:border-primary data-[state=on]:bg-primary data-[state=on]:text-primary-foreground sm:text-sm';
</script>

<div class="space-y-2" data-testid="labels">
	<ToggleGroup.Root
		type="multiple"
		variant="outline"
		class="grid w-full grid-cols-3"
		aria-label="What"
		bind:value={() => labels.what, (value) => (labels = { ...labels, what: value as What[] })}
	>
		{#each whatLabels as option (option.value)}
			<ToggleGroup.Item value={option.value} class={item}>{option.label}</ToggleGroup.Item>
		{/each}
	</ToggleGroup.Root>
	<ToggleGroup.Root
		type="multiple"
		variant="outline"
		class="grid w-full grid-cols-2"
		aria-label="When"
		bind:value={() => labels.when, (value) => (labels = { ...labels, when: value as When[] })}
	>
		{#each whenLabels as option (option.value)}
			<ToggleGroup.Item value={option.value} class={item}>{option.label}</ToggleGroup.Item>
		{/each}
	</ToggleGroup.Root>
	<ToggleGroup.Root
		type="multiple"
		variant="outline"
		class="grid w-full grid-cols-3"
		aria-label="Duties"
		bind:value={() => labels.duties, (value) => (labels = { ...labels, duties: value as Duties[] })}
	>
		{#each dutyLabels as option (option.value)}
			<ToggleGroup.Item value={option.value} class={item}>{option.label}</ToggleGroup.Item>
		{/each}
	</ToggleGroup.Root>
</div>
