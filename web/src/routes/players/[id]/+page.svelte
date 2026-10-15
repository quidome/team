<!--
	One player (attendance at training, story A): the name, the roles of the season, "Left the team"
	and "Undo", "Back in the team", and "Delete player".
-->
<script lang="ts">
	import { goto, invalidateAll } from '$app/navigation';
	import { resolve } from '$app/paths';
	import {
		ApiError,
		addRole,
		changeRole,
		deletePlayer,
		renamePlayer,
		type RoleKind
	} from '#lib/api/client.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import { formatDay } from '#lib/schedule.js';
	import { fullName, roleLabels } from '#lib/players.js';
	import { untrack } from 'svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	let firstName = $state(untrack(() => data.player?.first_name ?? ''));
	let lastName = $state(untrack(() => data.player?.last_name ?? ''));
	let leaving = $state(false);
	let until = $state('');
	let returning = $state(false);
	let returnRole = $state<RoleKind>('plays_games');
	let returnFrom = $state('');
	let busy = $state(false);
	let error = $state<string | null>(null);
	let warning = $state<string | null>(null);

	/** The newest role of the season: the one that "Left the team" or "Undo" changes. */
	const newest = $derived(data.player?.roles.at(-1) ?? null);

	async function run(action: () => Promise<unknown>) {
		busy = true;
		error = null;
		try {
			await action();
			await invalidateAll();
		} catch (failure) {
			error =
				failure instanceof ApiError ? failure.message : 'Something went wrong. Please try again.';
		} finally {
			busy = false;
		}
	}

	function saveName(event: SubmitEvent) {
		event.preventDefault();
		const player = data.player;
		if (!player) return;
		warning = null;
		return run(async () => {
			const saved = await renamePlayer(player.id, firstName, lastName.trim() ? lastName : null);
			if (saved.same_name) warning = 'There is already a player with this name.';
		});
	}

	function leave(event: SubmitEvent) {
		event.preventDefault();
		const player = data.player;
		const role = newest;
		if (!player || !role) return;
		return run(async () => {
			await changeRole(player.id, role.id, { role: role.role, from: role.from, until });
			leaving = false;
		});
	}

	function undoLeave() {
		const player = data.player;
		const role = newest;
		if (!player || !role) return;
		return run(() =>
			changeRole(player.id, role.id, { role: role.role, from: role.from, until: null })
		);
	}

	function comeBack(event: SubmitEvent) {
		event.preventDefault();
		const player = data.player;
		if (!player) return;
		return run(async () => {
			await addRole(player.id, { role: returnRole, from: returnFrom, until: null });
			returning = false;
		});
	}

	async function remove() {
		const player = data.player;
		if (!player) return;
		if (!confirm(`Delete ${fullName(player)}? This cannot be undone.`)) return;
		busy = true;
		error = null;
		try {
			await deletePlayer(player.id);
			await goto(resolve('/players'), { invalidateAll: true });
		} catch (failure) {
			error =
				failure instanceof ApiError ? failure.message : 'Something went wrong. Please try again.';
			busy = false;
		}
	}

	function startLeaving() {
		until = data.players?.today ?? '';
		leaving = true;
	}

	function startReturning() {
		returnFrom = data.players?.today ?? '';
		returnRole = newest?.role ?? 'plays_games';
		returning = true;
	}

	const field =
		'h-11 w-full min-w-0 rounded-md border border-input bg-background px-3 text-base shadow-xs focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none';
</script>

<svelte:head>
	<title>{data.player ? fullName(data.player) : 'Player'} · Team</title>
</svelte:head>

{#if !data.players}
	<section class="py-16 text-center">
		<p class="text-muted-foreground">Log in to see the players.</p>
		<Button href="/auth/login" class="mt-4" data-sveltekit-reload>Log in</Button>
	</section>
{:else if !data.player}
	<section class="space-y-4 py-8">
		<p class="text-muted-foreground">This player does not exist.</p>
		<Button href={resolve('/players')} variant="outline">Players</Button>
	</section>
{:else}
	{@const player = data.player}
	<section class="space-y-6">
		<div class="flex items-center justify-between gap-3">
			<h1 class="min-w-0 truncate text-2xl font-semibold tracking-tight" data-testid="player-name">
				{fullName(player)}
			</h1>
			<Button href={resolve('/players')} variant="ghost">Back</Button>
		</div>

		{#if error}
			<p class="text-sm" role="alert">{error}</p>
		{/if}
		{#if warning}
			<p class="text-sm" role="status" data-testid="same-name">{warning}</p>
		{/if}

		<form class="space-y-4 rounded-xl border bg-card p-4 shadow-xs" onsubmit={saveName}>
			<h2 class="font-medium">Name</h2>
			<label class="block space-y-1">
				<span class="text-sm">First name</span>
				<input class={field} bind:value={firstName} autocomplete="off" required />
			</label>
			<label class="block space-y-1">
				<span class="text-sm">Last name (optional)</span>
				<input class={field} bind:value={lastName} autocomplete="off" />
			</label>
			<Button type="submit" variant="outline" disabled={busy}>Save name</Button>
		</form>

		<section class="space-y-3 rounded-xl border bg-card p-4 shadow-xs" aria-labelledby="roles">
			<h2 id="roles" class="font-medium">In the team · {data.players.season}</h2>
			{#if player.roles.length === 0}
				<p class="text-sm text-muted-foreground">Not in the team this season.</p>
			{:else}
				<ul class="space-y-1" data-testid="roles">
					{#each player.roles as role (role.id)}
						<li class="text-sm tabular-nums">
							<span class="font-medium">{roleLabels[role.role]}</span>
							· from {formatDay(role.from)}{role.until ? ` until ${formatDay(role.until)}` : ''}
						</li>
					{/each}
				</ul>
			{/if}

			{#if newest && !newest.until}
				{#if leaving}
					<form class="space-y-3" onsubmit={leave}>
						<label class="block space-y-1">
							<span class="text-sm">Last day in the team</span>
							<input class={field} type="date" bind:value={until} required />
						</label>
						<div class="flex gap-2">
							<Button type="submit" disabled={busy}>Left the team</Button>
							<Button type="button" variant="ghost" onclick={() => (leaving = false)}>Cancel</Button
							>
						</div>
					</form>
				{:else}
					<Button variant="outline" onclick={startLeaving}>Left the team</Button>
				{/if}
			{:else}
				<div class="flex flex-wrap gap-2">
					{#if newest?.until}
						<Button variant="outline" disabled={busy} onclick={undoLeave}
							>Undo "Left the team"</Button
						>
					{/if}
					{#if !returning}
						<Button variant="outline" onclick={startReturning}>
							{player.roles.length === 0 ? 'Add to the team' : 'Back in the team'}
						</Button>
					{/if}
				</div>
				{#if returning}
					<form class="space-y-3" onsubmit={comeBack}>
						<fieldset class="space-y-2">
							<legend class="text-sm">Role</legend>
							<div class="flex gap-2">
								{#each Object.entries(roleLabels) as [value, label] (value)}
									<label
										class="flex min-h-11 flex-1 items-center gap-2 rounded-md border px-3 has-checked:border-primary has-checked:bg-primary/10"
									>
										<input type="radio" name="return-role" {value} bind:group={returnRole} />
										<span class="text-sm">{label}</span>
									</label>
								{/each}
							</div>
						</fieldset>
						<label class="block space-y-1">
							<span class="text-sm">In the team from</span>
							<input class={field} type="date" bind:value={returnFrom} required />
						</label>
						<div class="flex gap-2">
							<Button type="submit" disabled={busy}>Save</Button>
							<Button type="button" variant="ghost" onclick={() => (returning = false)}>
								Cancel
							</Button>
						</div>
					</form>
				{/if}
			{/if}
		</section>

		<Button variant="ghost" class="text-destructive" disabled={busy} onclick={remove}>
			Delete player
		</Button>
	</section>
{/if}
