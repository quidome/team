<!--
	The page of a game (story A of duties): the game, and its duty places. At a home game of the
	club the 4 places (Referee 1, Referee 2, Table 1, Table 2), each a pull-down list of "—" and the
	club teams that saves on change, the places of the team, and "Schedule entered". At an away game
	of the team the number of drivers, proposed until the coordinator sets one ("Use proposal" goes
	back). The places of the team (story B): a tap opens the choice of a player, with the totals of
	that kind and the clashes; a tap on a player saves at once; "Clear". On a past game "Someone
	else" (the same choice, never blocked by a clash) and "Nobody". A save that fails shows "Not
	saved. Try again." and the stored state.
-->
<script lang="ts">
	import { page as current } from '$app/state';
	import { resolve } from '$app/paths';
	import type { ResolvedPathname } from '$app/types';
	import {
		ApiError,
		clearAssignment,
		loadGame,
		setAssignment,
		setDrivers,
		setOutcome,
		setPlace,
		setScheduleEntered,
		type DutyKind,
		type GamePage,
		type PlaceOfTeam
	} from '#lib/api/client.js';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import {
		clashText,
		driversNumber,
		notOffered,
		placeName,
		placesWarnings,
		playerText,
		teamPlacesText,
		totalText
	} from '#lib/duties.js';
	import { formatDay } from '#lib/schedule.js';
	import MapPin from '@lucide/svelte/icons/map-pin';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	// Replaced by the answer of a save, and by the stored state after a failed save.
	let game = $derived(data.page);
	/** Counts the saves: the pull-down lists show the stored teams again after each one. */
	let saves = $state(0);

	// The field shows the stored number again after each save; typing changes it.
	let drivers = $derived(String(game?.game.places?.drivers?.places.length ?? 0));
	/** A whole number from 0 to 10, else null: "Save" waits for one. */
	const driversValue = $derived(/^\d{1,2}$/.test(drivers.trim()) ? Number(drivers) : null);
	const driversValid = $derived(driversValue !== null && driversValue <= 10);

	/** The place of the team whose choice is open; `correcting`: "Someone else". */
	let chosen = $state<{ kind: DutyKind; number: number; correcting: boolean } | null>(null);

	let busy = $state(false);
	let error = $state<string | null>(null);
	let notSaved = $state(false);

	/** Where the game was opened from: `?from=start` or `?from=changes`, else "Season". */
	const from = $derived(current.url.searchParams.get('from'));
	function backOf(season: string): ResolvedPathname {
		if (from === 'start') return resolve('/');
		if (from === 'changes') return resolve('/changes');
		return resolve(`/season?season=${encodeURIComponent(season)}`);
	}

	/** Runs a save; a refusal shows its message, any other failure "Not saved. Try again."; both
	 * then show what is stored. */
	async function save(page: GamePage, action: (id: string) => Promise<GamePage>) {
		const id = String(page.game.id);
		busy = true;
		error = null;
		notSaved = false;
		try {
			game = await action(id);
		} catch (failure) {
			if (failure instanceof ApiError && failure.code && failure.code !== 'internal')
				error = failure.message;
			else notSaved = true;
			try {
				game = await loadGame(fetch, page.season, id);
			} catch {
				// The page keeps what it shows.
			}
		} finally {
			saves += 1;
			busy = false;
			chosen = null;
		}
	}

	/** The places of the team: the table and referee places that name it, then the drivers. */
	function teamPlaces(page: GamePage): PlaceOfTeam[] {
		return [
			...(page.game.places?.schedule?.of_team ?? []),
			...(page.game.places?.drivers?.places ?? [])
		];
	}

	function isChosen(place: PlaceOfTeam): boolean {
		return chosen?.kind === place.kind && chosen.number === place.number;
	}

	/** A tap on a player in the choice of `place`. */
	function choose(page: GamePage, place: PlaceOfTeam, player: number, correcting: boolean) {
		save(page, (id) =>
			correcting
				? setOutcome(page.season, id, place.kind, place.number, 'done_by', player)
				: setAssignment(page.season, id, place.kind, place.number, player)
		);
	}

	/** The number of drivers to send: the proposal shown is sent as `null`, so it stays the
	 * proposal (D4). */
	function driversInput(page: GamePage, value: number): number | null {
		const part = page.game.places?.drivers;
		return part?.required && value === part.proposal ? null : value;
	}

	const placeNames = { referee: 'Referee', table: 'Table' } as const;
	const field =
		'h-11 w-full min-w-0 rounded-md border border-input bg-background px-3 text-base shadow-xs focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none';
</script>

<svelte:head>
	<title>Game · Team</title>
</svelte:head>

{#if data.missing}
	<section class="space-y-4 py-16 text-center">
		<p data-testid="game-missing">This game cannot be found.</p>
		<Button href={resolve('/season')} variant="outline">Season</Button>
	</section>
{:else if !game}
	<section class="py-16 text-center">
		<p class="text-muted-foreground">Log in to see the game.</p>
		<Button href="/auth/login" class="mt-4" data-sveltekit-reload>Log in</Button>
	</section>
{:else}
	{@const page = game}
	{@const schedule = page.game.places?.schedule}
	{@const driverPart = page.game.places?.drivers}
	<section class="space-y-6">
		<div class="flex items-start justify-between gap-3">
			<div class="min-w-0">
				<h1 class="text-2xl font-semibold tracking-tight" data-testid="game-heading">
					{formatDay(page.game.date)} · {page.game.start_time ?? 'time not known yet'}
				</h1>
				<p class="mt-1 text-sm leading-snug">
					<span class="block font-medium">{page.game.home}</span>
					<span class="block font-medium"><span class="sr-only">against</span>{page.game.away}</span
					>
				</p>
				<p class="mt-1 flex items-center gap-1 text-xs text-muted-foreground">
					<MapPin class="size-3.5 shrink-0" aria-hidden="true" />
					<span class="truncate">{page.game.hall}, {page.game.town}</span>
				</p>
				{#if page.game.reason === 'team'}
					<Badge class="mt-2 bg-team text-primary-foreground">Team</Badge>
				{/if}
			</div>
			<Button href={backOf(page.season)} variant="ghost">Back</Button>
		</div>

		{#each placesWarnings(page.game) as warning (warning)}
			<p class="text-sm" role="status" data-testid="places-warning">{warning}</p>
		{/each}

		{#if notSaved}
			<p class="text-sm" role="alert" data-testid="not-saved">Not saved. Try again.</p>
		{/if}
		{#if error}
			<p class="text-sm" role="alert" data-testid="save-error">{error}</p>
		{/if}

		{#if schedule}
			<div class="space-y-3 rounded-xl border bg-card p-4 shadow-xs" data-testid="schedule">
				<h2 class="font-medium">Table and referee</h2>
				{#key saves}
					<div class="grid grid-cols-2 gap-2">
						{#each schedule.places as place (`${place.kind}-${place.number}`)}
							<label class="block min-w-0 space-y-1">
								<span class="text-sm">{placeNames[place.kind]} {place.number}</span>
								<select
									class={field}
									disabled={busy}
									value={place.team ? String(place.team.number) : ''}
									onchange={(event) => {
										const value = event.currentTarget.value;
										const team = value === '' ? null : Number(value);
										save(page, (id) => setPlace(page.season, id, place.kind, place.number, team));
									}}
								>
									<option value="">—</option>
									{#each page.club_teams as team (team.number)}
										<option value={String(team.number)}>{team.name}</option>
									{/each}
								</select>
							</label>
						{/each}
					</div>
				{/key}
				<p class="text-sm font-medium" data-testid="team-places">
					{teamPlacesText(page.team, schedule)}
				</p>
				{#if schedule.required || schedule.entered}
					<div class="flex flex-wrap items-center gap-3">
						<p class="text-sm" data-testid="schedule-state">
							{schedule.entered ? 'Schedule entered' : 'Schedule not entered'}
						</p>
						<Button
							variant={schedule.entered ? 'outline' : 'default'}
							aria-pressed={schedule.entered}
							disabled={busy}
							onclick={() =>
								save(page, (id) => setScheduleEntered(page.season, id, !schedule.entered))}
						>
							Schedule entered
						</Button>
					</div>
				{/if}
			</div>
		{/if}

		{#if teamPlaces(page).length > 0}
			<div class="space-y-3 rounded-xl border bg-card p-4 shadow-xs" data-testid="team-duties">
				<h2 class="font-medium">Players on the places</h2>
				<ul class="space-y-2">
					{#each teamPlaces(page) as place (`${place.kind}-${place.number}`)}
						{@const name = placeName(place.kind, place.number)}
						{@const on = place.player}
						{@const onChoice = on ? page.players.find((player) => player.id === on.id) : undefined}
						<li class="space-y-2" data-testid="team-place">
							<button
								type="button"
								class="flex min-h-11 w-full items-center justify-between gap-3 rounded-md border px-3 text-left"
								aria-expanded={isChosen(place)}
								disabled={busy}
								onclick={() =>
									(chosen = isChosen(place)
										? null
										: { kind: place.kind, number: place.number, correcting: false })}
							>
								<span class="text-sm">{name}</span>
								<span class="min-w-0 truncate text-sm font-medium" data-testid="place-player">
									{on ? playerText(on) : 'open'}
								</span>
							</button>
							{#if on?.not_in_team}
								<p class="text-xs text-muted-foreground">Not in the team on this date</p>
							{/if}
							{#each onChoice?.clashes ?? [] as clash (clash)}
								<p class="text-xs text-muted-foreground" data-testid="place-clash">
									{clashText(clash)}
								</p>
							{/each}
							{#if isChosen(place) && chosen}
								{@const correcting = chosen.correcting}
								<div class="space-y-2 rounded-md bg-muted/40 p-2" data-testid="choice">
									{#if on && page.past && !correcting}
										<div class="flex flex-wrap gap-2">
											<Button
												variant="outline"
												disabled={busy}
												onclick={() => chosen && (chosen = { ...chosen, correcting: true })}
											>
												Someone else
											</Button>
											<Button
												variant="outline"
												disabled={busy}
												onclick={() =>
													save(page, (id) =>
														setOutcome(page.season, id, place.kind, place.number, 'not_done')
													)}
											>
												Nobody
											</Button>
										</div>
									{/if}
									{#if !on || !page.past || correcting}
										<ul class="space-y-1" aria-label="Choose a player for {name}">
											{#each page.players as player (player.id)}
												{@const reason = notOffered(player, on, correcting)}
												<li>
													<button
														type="button"
														class="w-full rounded-md px-3 py-2 text-left enabled:hover:bg-accent disabled:text-muted-foreground"
														disabled={busy || reason !== null}
														onclick={() => choose(page, place, player.id, correcting)}
														data-testid="choice-player"
													>
														<span class="flex flex-wrap items-baseline justify-between gap-x-3">
															<span class="font-medium">{player.name}</span>
															<span class="text-xs">{totalText(player, place.kind)}</span>
														</span>
														{#if reason}
															<span class="block text-xs">{reason}</span>
														{/if}
														{#each player.clashes as clash (clash)}
															<span class="block text-xs">{clashText(clash)}</span>
														{/each}
													</button>
												</li>
											{:else}
												<li class="px-3 py-2 text-sm text-muted-foreground">
													No players play games on this date.
												</li>
											{/each}
										</ul>
									{/if}
									{#if on}
										<Button
											variant="outline"
											disabled={busy}
											onclick={() =>
												save(page, (id) =>
													clearAssignment(page.season, id, place.kind, place.number)
												)}
										>
											Clear
										</Button>
									{/if}
								</div>
							{/if}
						</li>
					{/each}
				</ul>
			</div>
		{/if}

		{#if driverPart}
			<form
				class="space-y-3 rounded-xl border bg-card p-4 shadow-xs"
				data-testid="drivers-form"
				onsubmit={(event) => {
					event.preventDefault();
					if (driversValue === null || !driversValid) return;
					const value = driversInput(page, driversValue);
					save(page, (id) => setDrivers(page.season, id, value));
				}}
			>
				<h2 class="font-medium" data-testid="drivers">{driversNumber(driverPart)}</h2>
				{#if !driverPart.proposed}
					<p class="text-sm text-muted-foreground" data-testid="drivers-proposal">
						Proposal: {driverPart.proposal} (one for every 3 players)
					</p>
				{/if}
				<label class="block space-y-1">
					<span class="text-sm">Number of drivers</span>
					<input class={field} type="text" inputmode="numeric" bind:value={drivers} />
				</label>
				<div class="flex gap-2">
					<Button type="submit" disabled={busy || !driversValid}>Save</Button>
					{#if !driverPart.proposed && driverPart.required}
						<Button
							type="button"
							variant="outline"
							disabled={busy}
							onclick={() => save(page, (id) => setDrivers(page.season, id, null))}
						>
							Use proposal
						</Button>
					{/if}
				</div>
			</form>
		{/if}
	</section>
{/if}
