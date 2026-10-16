<!--
	The players of the team in the current season (attendance at training, story A): the players with
	a role today, "Left the team", players of last season, other players, and "Add player".
-->
<script lang="ts">
	import { invalidateAll } from '$app/navigation';
	import { resolve } from '$app/paths';
	import {
		ApiError,
		addPlayer,
		carryPlayers,
		type Player,
		type RoleKind
	} from '#lib/api/client.js';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import { fullName, roleLabels } from '#lib/players.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	let adding = $state(false);
	let firstName = $state('');
	let lastName = $state('');
	let role = $state<RoleKind>('plays_games');
	let from = $state('');
	let busy = $state(false);
	let error = $state<string | null>(null);
	let warning = $state<string | null>(null);
	let othersOpen = $state(false);

	/** "Add all" or "Add to this season": the same role as at the end of last season. */
	async function carry(players: number[]) {
		busy = true;
		error = null;
		try {
			await carryPlayers(players);
			await invalidateAll();
		} catch (failure) {
			error =
				failure instanceof ApiError ? failure.message : 'Something went wrong. Please try again.';
		} finally {
			busy = false;
		}
	}

	function startAdding() {
		adding = true;
		error = null;
		warning = null;
		from = data.players?.proposed_from ?? '';
	}

	async function save(event: SubmitEvent) {
		event.preventDefault();
		busy = true;
		error = null;
		warning = null;
		try {
			const saved = await addPlayer(firstName, lastName.trim() ? lastName : null, {
				role,
				from
			});
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

{#snippet row(player: Player)}
	<li>
		<a
			href={resolve('/players/[id]', { id: String(player.id) })}
			class="flex min-h-12 items-center justify-between gap-3 rounded-xl border bg-card px-4 py-3 shadow-xs"
			data-testid="player"
		>
			<span class="min-w-0 truncate font-medium">{fullName(player)}</span>
			{#if player.role}
				<Badge variant="secondary">{roleLabels[player.role]}</Badge>
			{/if}
		</a>
	</li>
{/snippet}

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
				<fieldset class="space-y-2">
					<legend class="text-sm">Role</legend>
					<div class="flex gap-2">
						{#each Object.entries(roleLabels) as [value, label] (value)}
							<label
								class="flex min-h-11 flex-1 items-center gap-2 rounded-md border px-3 has-checked:border-primary has-checked:bg-primary/10"
							>
								<input type="radio" name="role" {value} bind:group={role} />
								<span class="text-sm">{label}</span>
							</label>
						{/each}
					</div>
				</fieldset>
				<label class="block space-y-1">
					<span class="text-sm">In the team from</span>
					<input
						class={field}
						type="date"
						bind:value={from}
						min={data.players?.season_first_day}
						max={data.players?.season_last_day}
						required
					/>
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
			<Button onclick={startAdding}>Add player</Button>
		{/if}

		{#if data.players.last_season.length > 0}
			<section
				aria-labelledby="last-season-heading"
				class="space-y-3 rounded-xl border bg-card p-4 shadow-xs"
				data-testid="last-season"
			>
				<div class="flex items-center justify-between gap-3">
					<h2 id="last-season-heading" class="font-medium">Players of last season</h2>
					<Button
						size="sm"
						disabled={busy}
						onclick={() => carry(data.players?.last_season.map((player) => player.id) ?? [])}
					>
						Add all
					</Button>
				</div>
				{#if error && !adding}
					<p class="text-sm" role="alert">{error}</p>
				{/if}
				<ul class="space-y-2">
					{#each data.players.last_season as player (player.id)}
						<li class="flex min-h-12 items-center justify-between gap-3">
							<span class="min-w-0 truncate">
								{fullName(player)}
								{#if player.role}
									<span class="text-sm text-muted-foreground">· {roleLabels[player.role]}</span>
								{/if}
							</span>
							<Button
								variant="outline"
								size="sm"
								disabled={busy}
								aria-label="Add {fullName(player)} to this season"
								onclick={() => carry([player.id])}
							>
								Add to this season
							</Button>
						</li>
					{/each}
				</ul>
			</section>
		{/if}

		{#if data.players.players.length === 0}
			<p class="text-sm text-muted-foreground" data-testid="no-players">
				No players in the team yet.
			</p>
		{:else}
			<ul class="space-y-2" aria-label="Players">
				{#each data.players.players as player (player.id)}
					{@render row(player)}
				{/each}
			</ul>
		{/if}

		{#if data.players.left.length > 0}
			<section aria-labelledby="left-heading" class="space-y-2">
				<h2 id="left-heading" class="text-lg font-semibold tracking-tight">Left the team</h2>
				<ul class="space-y-2" data-testid="left">
					{#each data.players.left as player (player.id)}
						{@render row(player)}
					{/each}
				</ul>
			</section>
		{/if}

		{#if data.players.others.length > 0}
			<section class="space-y-2">
				<Button
					variant="ghost"
					class="w-full justify-between"
					aria-expanded={othersOpen}
					onclick={() => (othersOpen = !othersOpen)}
				>
					Other players ({data.players.others.length})
					<span aria-hidden="true">{othersOpen ? '−' : '+'}</span>
				</Button>
				{#if othersOpen}
					<ul class="space-y-2" data-testid="others">
						{#each data.players.others as player (player.id)}
							{@render row(player)}
						{/each}
					</ul>
				{/if}
			</section>
		{/if}
	</section>
{/if}
