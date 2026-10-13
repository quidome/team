<!--
	The changes not seen yet (story 4): the quick filter, each change with its old and new values,
	one summary line for many new possible duty games, and "seen" for one or all.
-->
<script lang="ts">
	import { invalidateAll } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { ApiError, markSeen, type Change } from '#lib/api/client.js';
	import FilterToggle from '#lib/components/FilterToggle.svelte';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import {
		details,
		gameOf,
		kindLine,
		noChangesLine,
		seenAllLabel,
		splitSummary
	} from '#lib/changes.js';
	import { filterGames, type Filter } from '#lib/schedule.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	let filter = $state<Filter>('all');
	let summaryOpen = $state(false);
	let busy = $state(false);
	let error = $state<string | null>(null);

	const shown = $derived(filterGames(data.changes ?? [], filter));
	const split = $derived(splitSummary(shown));

	async function seen(changes: Change[]) {
		busy = true;
		error = null;
		try {
			await markSeen(changes);
			// A change that a reading updated meanwhile stays, with its new values.
			await invalidateAll();
		} catch (failure) {
			error =
				failure instanceof ApiError ? failure.message : 'Something went wrong. Please try again.';
		} finally {
			busy = false;
		}
	}
</script>

<svelte:head>
	<title>Changes · Team</title>
</svelte:head>

{#snippet item(change: Change)}
	{@const game = gameOf(change)}
	{@const ofTeam = change.reason === 'team'}
	<li
		class={[
			'flex gap-3 rounded-xl border bg-card p-3 shadow-xs',
			ofTeam && 'border-l-4 border-l-team bg-team-soft'
		]}
		data-testid="change"
		data-of-team={ofTeam}
	>
		<div class="min-w-0 flex-1 space-y-1">
			{#if game}
				<p class="text-sm leading-snug font-medium">
					<span class="block">{game.home}</span>
					<span class="block"><span class="sr-only">against</span> {game.away}</span>
				</p>
			{/if}
			<p
				class={['text-sm font-semibold', change.kind === 'gone' && 'text-gone']}
				data-testid="change-kind"
			>
				{kindLine(change)}
			</p>
			{#each details(change) as line (line)}
				<p class="text-xs text-muted-foreground tabular-nums" data-testid="change-detail">
					{line}
				</p>
			{/each}
		</div>
		<div class="flex shrink-0 flex-col items-end gap-2">
			{#if ofTeam}
				<Badge class="bg-team text-primary-foreground">Team</Badge>
			{/if}
			<Button
				variant="outline"
				size="sm"
				disabled={busy}
				aria-label="Seen: {game ? `${game.home} against ${game.away}` : 'this change'}"
				onclick={() => seen([change])}
			>
				Seen
			</Button>
		</div>
	</li>
{/snippet}

{#if !data.changes}
	<section class="py-16 text-center">
		<p class="text-muted-foreground">Log in to see the changes.</p>
		<Button href="/auth/login" class="mt-4" data-sveltekit-reload>Log in</Button>
	</section>
{:else}
	<section class="space-y-5">
		<div class="flex items-center justify-between gap-3">
			<h1 class="text-2xl font-semibold tracking-tight">Changes</h1>
			<Button href={resolve('/')} variant="ghost">Back</Button>
		</div>

		{#if data.changes.length === 0}
			<p class="text-sm text-muted-foreground" data-testid="no-changes">
				{noChangesLine('all')}
			</p>
		{:else}
			<FilterToggle bind:filter label="Filter changes" />

			{#if error}
				<p class="text-sm" role="alert">{error}</p>
			{/if}

			{#if shown.length === 0}
				<p class="text-sm text-muted-foreground" data-testid="no-changes">
					{noChangesLine(filter)}
				</p>
			{:else}
				<ul class="space-y-2">
					{#each split.single as change (change.id)}
						{@render item(change)}
					{/each}
				</ul>

				{#if split.newDutyGames.length > 0}
					<div class="space-y-2">
						<Button
							variant="outline"
							class="w-full justify-between"
							aria-expanded={summaryOpen}
							data-testid="new-games-summary"
							onclick={() => (summaryOpen = !summaryOpen)}
						>
							{split.newDutyGames.length} new games
							<span class="text-xs text-muted-foreground">{summaryOpen ? 'Hide' : 'Show'}</span>
						</Button>
						{#if summaryOpen}
							<ul class="space-y-2" data-testid="new-games">
								{#each split.newDutyGames as change (change.id)}
									{@render item(change)}
								{/each}
							</ul>
						{/if}
					</div>
				{/if}

				<Button class="w-full" disabled={busy} onclick={() => seen(shown)}>
					{seenAllLabel(filter, shown.length)}
				</Button>
			{/if}
		{/if}
	</section>
{/if}
