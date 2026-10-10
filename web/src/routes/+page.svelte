<script lang="ts">
	import { Button } from '#lib/components/ui/button/index.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<svelte:head>
	<title>Team</title>
</svelte:head>

{#if data.me}
	<section class="space-y-6">
		<h1 class="text-2xl font-semibold tracking-tight">Hello, {data.me.name}</h1>
		<div class="rounded-xl border bg-card p-5 shadow-xs">
			<h2 class="font-medium">Your schedule comes here</h2>
			<p class="mt-1 text-sm text-muted-foreground">
				Soon the tool reads the club's games from the game list. Until then, a sample shows how the
				schedule will look.
			</p>
			<Button href="/sample" class="mt-4">See the sample schedule</Button>
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
