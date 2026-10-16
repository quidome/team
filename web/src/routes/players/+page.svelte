<!--
	All players of the club (attendance at training, story A): each with the role today or "Not in
	the team", "Show hidden players", and "Add player" (a name only; the team page gives roles).
-->
<script lang="ts">
	import { invalidateAll } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { ApiError, addPlayer } from '#lib/api/client.js';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import { fullName, roleLabels } from '#lib/players.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	let showHidden = $state(false);
	let adding = $state(false);
	let firstName = $state('');
	let lastName = $state('');
	let busy = $state(false);
	let error = $state<string | null>(null);
	let warning = $state<string | null>(null);

	const shown = $derived(
		(data.players?.players ?? []).filter((player) => showHidden || !player.hidden)
	);

	async function save(event: SubmitEvent) {
		event.preventDefault();
		busy = true;
		error = null;
		warning = null;
		try {
			const saved = await addPlayer(firstName, lastName.trim() ? lastName : null, null);
			if (saved.same_name) warning = 'There is already a player with this name.';
			firstName = '';
			lastName = '';
			adding = false;
			await invalidateAll();
		} catch (failure) {
			error =
				failure instanceof ApiError ? failure.message : 'Something went wrong. Please try again.';
		} finally {
			busy = false;
		}
	}

	const field =
		'h-11 w-full min-w-0 rounded-md border border-input bg-background px-3 text-base shadow-xs focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none';
</script>

<svelte:head>
	<title>Players · Team</title>
</svelte:head>

{#if !data.players}
	<section class="py-16 text-center">
		<p class="text-muted-foreground">Log in to see the players.</p>
		<Button href="/auth/login" class="mt-4" data-sveltekit-reload>Log in</Button>
	</section>
{:else}
	<section class="space-y-6">
		<div class="flex items-center justify-between gap-3">
			<div>
				<p class="text-sm text-muted-foreground">Season {data.players.season}</p>
				<h1 class="text-2xl font-semibold tracking-tight">Players</h1>
			</div>
			<Button href={resolve('/')} variant="ghost">Back</Button>
		</div>

		{#if warning}
			<p class="text-sm" role="status" data-testid="same-name">{warning}</p>
		{/if}

		{#if adding}
			<form
				class="space-y-4 rounded-xl border bg-card p-4 shadow-xs"
				onsubmit={save}
				data-testid="add-player"
			>
				<h2 class="font-medium">Add player</h2>
				<label class="block space-y-1">
					<span class="text-sm">First name</span>
					<input class={field} bind:value={firstName} autocomplete="off" required />
				</label>
				<label class="block space-y-1">
					<span class="text-sm">Last name (optional)</span>
					<input class={field} bind:value={lastName} autocomplete="off" />
				</label>
				{#if error}
					<p class="text-sm" role="alert">{error}</p>
				{/if}
				<div class="flex gap-2">
					<Button type="submit" disabled={busy}>Save</Button>
					<Button type="button" variant="ghost" onclick={() => (adding = false)}>Cancel</Button>
				</div>
			</form>
		{:else}
			<Button onclick={() => ((adding = true), (error = null), (warning = null))}>Add player</Button
			>
		{/if}

		<label class="flex min-h-11 items-center gap-3 text-sm">
			<input type="checkbox" role="switch" bind:checked={showHidden} class="size-5" />
			Show hidden players
		</label>

		{#if shown.length === 0}
			<p class="text-sm text-muted-foreground" data-testid="no-players">No players yet.</p>
		{:else}
			<ul class="space-y-2" aria-label="Players">
				{#each shown as player (player.id)}
					<li>
						<a
							href={resolve('/players/[id]', { id: String(player.id) })}
							class="flex min-h-12 items-center justify-between gap-3 rounded-xl border bg-card px-4 py-3 shadow-xs"
							data-testid="player"
						>
							<span class="min-w-0 truncate font-medium">
								{fullName(player)}
								{#if player.hidden}
									<span class="text-sm font-normal text-muted-foreground">· hidden</span>
								{/if}
							</span>
							{#if player.role_today}
								<Badge variant="secondary">{roleLabels[player.role_today]}</Badge>
							{:else}
								<Badge variant="outline">Not in the team</Badge>
							{/if}
						</a>
					</li>
				{/each}
			</ul>
		{/if}
	</section>
{/if}
