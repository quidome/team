<!--
	The panel "Administration" (the administration panel): every mark of an event that can be set
	now, one `MarkRow` each. The page shows it only when it has a row. A panel without an orange
	button means the administration of the event is done.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();
</script>

<section
	aria-labelledby="administration-heading"
	class="rounded-xl border bg-muted px-4 pt-3 pb-1"
	data-testid="administration"
>
	<h2 id="administration-heading" class="text-sm font-semibold text-muted-foreground">
		Administration
	</h2>
	<div class="divide-y">{@render children()}</div>
</section>
