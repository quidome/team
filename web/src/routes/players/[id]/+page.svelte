<!--
	One player (attendance at training, story A): the name and "Hidden", the history of all seasons
	with "Correct" for a role of the current season, and "Delete player".
-->
<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import {
		ApiError,
		changePlayer,
		correctRole,
		deletePlayer,
		type HistoryRole,
		type PlayerDetail,
		type RoleKind
	} from '#lib/api/client.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import { formatDay } from '#lib/schedule.js';
	import { fullName, nowThereText, roleLabels } from '#lib/players.js';
	import { untrack } from 'svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	// Replaced by the answer of each save.
	let player = $state<PlayerDetail | null>(untrack(() => data.player));
	let firstName = $state(untrack(() => data.player?.first_name ?? ''));
	let lastName = $state(untrack(() => data.player?.last_name ?? ''));
	let hidden = $state(untrack(() => data.player?.hidden ?? false));

	/** The role in the "Correct" form. */
	let correcting = $state<number | null>(null);
	let role = $state<RoleKind>('plays_games');
	let from = $state('');
	let until = $state('');

	let busy = $state(false);
	let error = $state<string | null>(null);
	let warning = $state<string | null>(null);
	let notice = $state<string | null>(null);

	function message(failure: unknown): string {
		return failure instanceof ApiError
			? failure.message
			: 'Something went wrong. Please try again.';
	}

	async function save(event: SubmitEvent) {
		event.preventDefault();
		if (!player) return;
		busy = true;
		error = null;
		warning = null;
		try {
			const saved = await changePlayer(
				player.id,
				firstName,
				lastName.trim() ? lastName : null,
				hidden
			);
			player = saved.player;
			if (saved.same_name) warning = 'There is already a player with this name.';
		} catch (failure) {
			error = message(failure);
		} finally {
			busy = false;
		}
	}

	function startCorrecting(entry: HistoryRole) {
		correcting = entry.id;
		role = entry.role;
		from = entry.from;
		until = entry.until ?? '';
		error = null;
	}

	async function correct(event: SubmitEvent) {
		event.preventDefault();
		if (!player || correcting === null) return;
		busy = true;
		error = null;
		notice = null;
		try {
			player = await correctRole(player.id, correcting, {
				role,
				from,
				until: until || null
			});
			correcting = null;
			const counted = player.now_there[0];
			notice = counted ? nowThereText(counted.trainings) : null;
		} catch (failure) {
			error = message(failure);
		} finally {
			busy = false;
		}
	}

	async function remove() {
		if (!player) return;
		if (!confirm(`Delete ${fullName(player)}? This cannot be undone.`)) return;
		busy = true;
		error = null;
		try {
			await deletePlayer(player.id);
			await goto(resolve('/players'), { invalidateAll: true });
		} catch (failure) {
			error = message(failure);
			busy = false;
		}
	}

	function dates(entry: HistoryRole): string {
		return entry.until
			? `${formatDay(entry.from)} – ${formatDay(entry.until)}`
			: `from ${formatDay(entry.from)}`;
	}

	const field =
		'h-11 w-full min-w-0 rounded-md border border-input bg-background px-3 text-base shadow-xs focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none';
</script>

<svelte:head>
	<title>{player ? fullName(player) : 'Player'} · Team</title>
</svelte:head>

{#if !data.loggedIn}
	<section class="py-16 text-center">
		<p class="text-muted-foreground">Log in to see the players.</p>
		<Button href="/auth/login" class="mt-4" data-sveltekit-reload>Log in</Button>
	</section>
{:else if !player}
	<section class="space-y-4 py-8">
		<p class="text-muted-foreground">This player does not exist.</p>
		<Button href={resolve('/players')} variant="outline">Players</Button>
	</section>
{:else}
	{@const shown = player}
	<section class="space-y-6">
		<div class="flex items-center justify-between gap-3">
			<h1 class="min-w-0 truncate text-2xl font-semibold tracking-tight" data-testid="player-name">
				{fullName(shown)}
			</h1>
			<Button href={resolve('/players')} variant="ghost">Back</Button>
		</div>

		{#if error && correcting === null}
			<p class="text-sm" role="alert">{error}</p>
		{/if}
		{#if warning}
			<p class="text-sm" role="status" data-testid="same-name">{warning}</p>
		{/if}
		{#if notice}
			<p class="text-sm" role="status" data-testid="now-there">{notice}</p>
		{/if}

		<form class="space-y-4 rounded-xl border bg-card p-4 shadow-xs" onsubmit={save}>
			<h2 class="font-medium">Player</h2>
			<label class="block space-y-1">
				<span class="text-sm">First name</span>
				<input class={field} bind:value={firstName} autocomplete="off" required />
			</label>
			<label class="block space-y-1">
				<span class="text-sm">Last name (optional)</span>
				<input class={field} bind:value={lastName} autocomplete="off" />
			</label>
			<label class="flex min-h-11 items-center gap-3 text-sm">
				<input type="checkbox" role="switch" bind:checked={hidden} class="size-5" />
				<span>
					Hidden
					<span class="block text-muted-foreground">
						Not in the list of players (unless "Show hidden players" is on), and not offered in "Add
						player" and "Add all". The history and the team stay.
					</span>
				</span>
			</label>
			<Button type="submit" variant="outline" disabled={busy}>Save</Button>
		</form>

		<section class="space-y-3 rounded-xl border bg-card p-4 shadow-xs" aria-labelledby="history">
			<h2 id="history" class="font-medium">History</h2>
			{#if shown.history.length === 0}
				<p class="text-sm text-muted-foreground">Never in the team.</p>
			{:else}
				<ul class="space-y-3" data-testid="history">
					{#each shown.history as entry (entry.id)}
						<li class="space-y-2" data-testid="history-role">
							<div class="flex min-h-11 items-center justify-between gap-3">
								<p class="text-sm tabular-nums">
									<span class="font-medium">{roleLabels[entry.role]}</span>
									· {dates(entry)}
									{#if entry.season !== shown.season}
										<span class="text-muted-foreground">· {entry.season}</span>
									{/if}
								</p>
								{#if entry.correctable && correcting !== entry.id}
									<Button variant="outline" size="sm" onclick={() => startCorrecting(entry)}>
										Correct
									</Button>
								{/if}
							</div>
							{#if correcting === entry.id}
								<form class="space-y-3" onsubmit={correct} data-testid="correct-form">
									<fieldset class="space-y-2">
										<legend class="text-sm">Role</legend>
										<div class="flex gap-2">
											{#each Object.entries(roleLabels) as [value, label] (value)}
												<label
													class="flex min-h-11 flex-1 items-center gap-2 rounded-md border px-3 has-checked:border-primary has-checked:bg-primary/10"
												>
													<input type="radio" name="correct-role" {value} bind:group={role} />
													<span class="text-sm">{label}</span>
												</label>
											{/each}
										</div>
									</fieldset>
									<label class="block space-y-1">
										<span class="text-sm">From</span>
										<input
											class={field}
											type="date"
											bind:value={from}
											min={shown.season_first_day}
											max={shown.season_last_day}
											required
										/>
									</label>
									<label class="block space-y-1">
										<span class="text-sm">Until (empty while the role continues)</span>
										<input
											class={field}
											type="date"
											bind:value={until}
											min={shown.season_first_day}
											max={shown.season_last_day}
										/>
									</label>
									{#if error}
										<p class="text-sm" role="alert">{error}</p>
									{/if}
									<div class="flex gap-2">
										<Button type="submit" disabled={busy}>Save</Button>
										<Button type="button" variant="ghost" onclick={() => (correcting = null)}>
											Cancel
										</Button>
									</div>
								</form>
							{/if}
						</li>
					{/each}
				</ul>
			{/if}
		</section>

		<Button variant="ghost" class="text-destructive" disabled={busy} onclick={remove}>
			Delete player
		</Button>
	</section>
{/if}
