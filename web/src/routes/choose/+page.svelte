<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import ChooseTeam from '#lib/components/ChooseTeam.svelte';
	import { Button } from '#lib/components/ui/button/index.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const club = $derived(
		data.selection && data.selection.state !== 'not_chosen' ? data.selection.club : null
	);
</script>

<svelte:head>
	<title>Club and team · Team</title>
</svelte:head>

{#if !data.selection}
	<section class="py-16 text-center">
		<p class="text-muted-foreground">Log in to choose the club and team.</p>
		<Button href="/auth/login" class="mt-4" data-sveltekit-reload>Log in</Button>
	</section>
{:else}
	<section class="space-y-5">
		<div class="flex items-center justify-between gap-3">
			<h1 class="text-2xl font-semibold tracking-tight">Club and team</h1>
			<Button href={resolve('/')} variant="ghost">Back</Button>
		</div>
		<ChooseTeam
			{club}
			clubKept={data.selection.state === 'team_needed'}
			onSaved={() => goto(resolve('/'), { invalidateAll: true })}
		/>
	</section>
{/if}
