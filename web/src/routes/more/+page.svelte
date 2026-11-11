<!--
	More (the quieter start page, story 5): the links the coordinator uses less than each week,
	away from the start page.
-->
<script lang="ts">
	import { resolve } from '$app/paths';
	import { Button } from '#lib/components/ui/button/index.js';
</script>

<svelte:head>
	<title>More · Team</title>
</svelte:head>

<section class="space-y-6">
	<div class="flex items-center justify-between gap-3">
		<h1 class="text-2xl font-semibold tracking-tight">More</h1>
		<Button href={resolve('/')} variant="ghost">Back</Button>
	</div>
	<nav aria-label="More" data-testid="more-links">
		<ul class="divide-y rounded-xl border bg-card shadow-xs">
			<li>
				<a href={resolve('/team')} class="flex min-h-12 items-center justify-between px-4"
					>Team <span aria-hidden="true" class="text-muted-foreground">›</span></a
				>
			</li>
			<li>
				<a href={resolve('/players')} class="flex min-h-12 items-center justify-between px-4"
					>Players <span aria-hidden="true" class="text-muted-foreground">›</span></a
				>
			</li>
			<li>
				<a href={resolve('/settings')} class="flex min-h-12 items-center justify-between px-4"
					>Settings <span aria-hidden="true" class="text-muted-foreground">›</span></a
				>
			</li>
		</ul>
	</nav>
</section>
