<!--
	The choose step: search the club by name, then choose one of its own teams. Starts at the team
	choice when `club` is given (a new season, or a change of team).
-->
<script lang="ts">
	import { onMount, untrack } from 'svelte';
	import { Button } from '#lib/components/ui/button/index.js';
	import {
		ApiError,
		loadTeams,
		saveSelection,
		searchClubs,
		type ClubTeams,
		type ClubView
	} from '#lib/api/client.js';

	let {
		club = null,
		clubKept = false,
		onSaved
	}: {
		club?: ClubView | null;
		/** True when `club` is already kept and only the team is missing (no "Save club" for it). */
		clubKept?: boolean;
		onSaved: () => void | Promise<void>;
	} = $props();

	let chosenClub = $state<ClubView | null>(untrack(() => club));
	let query = $state('');
	let results = $state<ClubView[] | null>(null);
	let teams = $state<ClubTeams | null>(null);
	let busy = $state(false);
	let error = $state<string | null>(null);
	let retry: (() => Promise<void>) | null = null;
	const keptWithoutTeam = $derived(clubKept && chosenClub?.number === club?.number);

	async function attempt(action: () => Promise<void>) {
		busy = true;
		error = null;
		retry = action;
		try {
			await action();
		} catch (failure) {
			error =
				failure instanceof ApiError ? failure.message : 'Something went wrong. Please try again.';
		} finally {
			busy = false;
		}
	}

	function search(event: SubmitEvent) {
		event.preventDefault();
		const search = query;
		void attempt(async () => {
			results = await searchClubs(search);
		});
	}

	function chooseClub(next: ClubView) {
		chosenClub = next;
		teams = null;
		void attempt(async () => {
			teams = await loadTeams(next.number);
		});
	}

	function save(team: number | null) {
		const current = chosenClub;
		if (!current) return;
		void attempt(async () => {
			await saveSelection(current.number, team);
			await onSaved();
		});
	}

	function otherClub() {
		chosenClub = null;
		teams = null;
		error = null;
	}

	onMount(() => {
		if (chosenClub) chooseClub(chosenClub);
	});
</script>

<div class="space-y-4" aria-busy={busy}>
	{#if !chosenClub}
		<form class="space-y-2" onsubmit={search}>
			<label for="club-search" class="text-sm font-medium">Club name</label>
			<div class="flex gap-2">
				<input
					id="club-search"
					type="search"
					bind:value={query}
					autocomplete="off"
					enterkeyhint="search"
					placeholder="For example: drakes"
					class="h-10 min-w-0 flex-1 rounded-lg border bg-background px-3 text-base outline-none focus-visible:ring-3 focus-visible:ring-ring/50"
				/>
				<Button type="submit" size="lg" class="h-10 px-4" disabled={busy}>Search</Button>
			</div>
		</form>

		{#if results && results.length === 0}
			<p class="text-sm text-muted-foreground" role="status">No club found. Try another name.</p>
		{:else if results}
			<ul class="space-y-2" aria-label="Clubs found">
				{#each results as result (result.number)}
					<li>
						<button
							type="button"
							class="flex min-h-12 w-full flex-col items-start justify-center rounded-xl border bg-card px-4 py-2 text-left shadow-xs hover:bg-muted"
							onclick={() => chooseClub(result)}
							data-testid="club-result"
						>
							<span class="font-medium">{result.name}</span>
							{#if result.town}
								<span class="text-sm text-muted-foreground">{result.town}</span>
							{/if}
						</button>
					</li>
				{/each}
			</ul>
		{/if}
	{:else}
		<div class="flex items-start justify-between gap-3">
			<div class="min-w-0">
				<h2 class="truncate text-lg font-semibold">{chosenClub.name}</h2>
				{#if chosenClub.town}
					<p class="text-sm text-muted-foreground">{chosenClub.town}</p>
				{/if}
			</div>
			<Button variant="outline" onclick={otherClub}>Other club</Button>
		</div>

		{#if teams && teams.teams.length === 0 && keptWithoutTeam}
			<p class="text-sm" role="status">
				{chosenClub.name} has no games for {teams.season} yet. The club is kept; choose the team here
				when games appear.
			</p>
		{:else if teams && teams.teams.length === 0}
			<p class="text-sm" role="status">
				{chosenClub.name} has no games for {teams.season} yet. Save the club now, and choose the team
				when games appear.
			</p>
			<Button size="lg" class="w-full" onclick={() => save(null)} disabled={busy}>Save club</Button>
		{:else if teams}
			<p class="text-sm text-muted-foreground">Choose your team for {teams.season}.</p>
			<ul class="grid grid-cols-2 gap-2" aria-label="Teams">
				{#each teams.teams as team (team.number)}
					<li>
						<Button
							variant="outline"
							class="h-12 w-full text-base"
							onclick={() => save(team.number)}
							disabled={busy}
							data-testid="team-choice"
						>
							{team.name}
						</Button>
					</li>
				{/each}
			</ul>
		{/if}
	{/if}

	{#if busy}
		<p class="text-sm text-muted-foreground" role="status">Reading the game list…</p>
	{/if}

	{#if error}
		<div
			class="space-y-2 rounded-xl border border-destructive/40 bg-destructive/10 p-4"
			role="alert"
		>
			<p class="text-sm">{error}</p>
			<Button variant="outline" onclick={() => retry && attempt(retry)}>Try again</Button>
		</div>
	{/if}
</div>
