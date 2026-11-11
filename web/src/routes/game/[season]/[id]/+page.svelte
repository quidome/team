<!--
	The page of a game (story A of duties): the game, and its duty places. Below the heading the
	panel "Administration" (the administration panel): one row for each mark that can be set now,
	in this order: "Not played" (when set) with "Undo"; on a not played game from its start time
	with driver places "No drive counts" / "The drives count" with "Count the drives" / "Undo";
	on a home game of the club "Schedule not entered" / "Schedule entered" with "Mark as entered" /
	"Undo"; on a game of the team from its start time "Not checked" / "Checked" with "Mark as
	checked" / "Undo", the count below it and after the tap "2 players with no answer are absent.".
	At a home game of the club the 4 places (Referee 1, Referee 2, Table 1, Table 2), each a
	pull-down list of "—" and the club teams that saves on change, and the places of the team. At
	an away game of the team the number of drivers, proposed until the coordinator sets one ("Use
	proposal" goes back). The places of the team (story B): a tap opens the choice of a player, with
	the totals of that kind and the clashes; a tap on a player saves at once; "Clear". On a past
	game "Someone else" (the same choice, never blocked by a clash) and "Nobody". A save that fails
	shows "Not saved. Try again." and the stored state. At the top of a game of the team: who comes
	(`GameAttendance`, story A of attendance at games). A driver whose answer is "not coming" shows
	"· not coming" at the place ("· absent" after the date) and "Not coming" in the choice of a
	driver (story B). "Mark as not played" as the last button; a not played game shows the list of
	who comes closed behind a tap. A checked game, and a game from before the release, show only
	the count, and "Show the list" opens the list.
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
		setChecked,
		setDrivers,
		setDriversDrove,
		setNotPlayed,
		setOutcome,
		setPlace,
		setScheduleEntered,
		unsetChecked,
		type AnswerView,
		type DutyKind,
		type GamePage,
		type PlaceOfTeam
	} from '#lib/api/client.js';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import AdminPanel from '#lib/components/AdminPanel.svelte';
	import GameAttendance from '#lib/components/GameAttendance.svelte';
	import MarkRow from '#lib/components/MarkRow.svelte';
	import { countText, noAnswerWarning, settledText } from '#lib/attendance.js';
	import {
		clashText,
		doerOf,
		driverNote,
		driversLine,
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

	/** The list of who comes is open on a not played, checked or older game ("Show the list"). */
	let listShown = $state(false);
	/** After "Checked": how many players with no answer became absent. */
	let settled = $state<string | null>(null);

	let busy = $state(false);
	let error = $state<string | null>(null);
	let notSaved = $state(false);

	/** Where the game was opened from: `?from=start`, `?from=changes` or `?from=report`, else
	 * "Season". */
	const from = $derived(current.url.searchParams.get('from'));
	function backOf(season: string): ResolvedPathname {
		if (from === 'start') return resolve('/');
		if (from === 'changes') return resolve('/changes');
		if (from === 'report') return resolve('/report');
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

	/** The answers shown by the open list (`GameAttendance`), with the page they belong to: its
	 * taps save without a new page. */
	let shown = $state.raw<{
		page: GamePage;
		answers: Record<number, AnswerView | null>;
	} | null>(null);
	/** The answer of each player: from the open list, else as stored, so a closed list (checked,
	 * not played, before the release) still gives the notes at the driver places. */
	const answers = $derived.by((): Record<number, AnswerView | null> => {
		if (game && shown?.page === game) return shown.answers;
		const stored: Record<number, AnswerView | null> = {};
		for (const row of game?.attendance?.rows ?? []) stored[row.player] = row.answer ?? null;
		return stored;
	});

	/** A player on a driver place, or in its choice, whose answer is "not coming"; never on a not
	 * played game, which warns about nothing (story E). */
	function notComingDriver(kind: DutyKind, player: number): boolean {
		return !game?.not_played && kind === 'drive' && answers[player] === 'not_coming';
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
	{@const drivesRow = page.not_played && page.started && (driverPart?.places.length ?? 0) > 0}
	{@const scheduleRow = !!schedule && (schedule.required || schedule.entered)}
	{@const checkedRow = !!page.attendance && page.started}
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

		{#if page.not_played || drivesRow || scheduleRow || checkedRow}
			<AdminPanel>
				{#if page.not_played}
					<MarkRow
						state="Not played"
						done={true}
						action="Undo"
						{busy}
						onclick={() => {
							listShown = false;
							save(page, (id) => setNotPlayed(page.season, id, false));
						}}
						testid="mark-not-played"
					>
						{#snippet line()}Its duties and attendance do not count.{/snippet}
					</MarkRow>
				{/if}
				{#if drivesRow}
					<MarkRow
						state={page.drivers_drove ? 'The drives count' : 'No drive counts'}
						done={page.drivers_drove}
						action={page.drivers_drove ? 'Undo' : 'Count the drives'}
						{busy}
						onclick={() =>
							save(page, (id) => setDriversDrove(page.season, id, !page.drivers_drove))}
						testid="mark-drives"
					/>
				{/if}
				{#if scheduleRow && schedule}
					<MarkRow
						state={schedule.entered ? 'Schedule entered' : 'Schedule not entered'}
						done={schedule.entered}
						action={schedule.entered ? 'Undo' : 'Mark as entered'}
						next={!schedule.entered}
						{busy}
						onclick={() =>
							save(page, (id) => setScheduleEntered(page.season, id, !schedule.entered))}
						testid="mark-entered"
					/>
				{/if}
				{#if checkedRow && page.attendance}
					{@const checked = page.attendance.checked}
					{@const count = countText(page.attendance.count, page.started)}
					{@const warning = noAnswerWarning(page.attendance.count)}
					<MarkRow
						state={checked ? 'Checked' : 'Not checked'}
						done={checked}
						action={checked ? 'Undo' : 'Mark as checked'}
						next={!checked}
						{busy}
						onclick={() => {
							if (checked) {
								settled = null;
								listShown = false;
								save(page, (id) => unsetChecked(page.season, id));
							} else
								save(page, async (id) => {
									const answer = await setChecked(page.season, id);
									settled = settledText(answer.settled);
									listShown = false;
									return answer.page;
								});
						}}
						testid="mark-checked"
					>
						{#snippet line()}
							{#if count}<span class="block">{count}</span>{/if}
							{#if driverPart && driverPart.places.length > 0}<span
									class="block"
									data-testid="checked-drivers">{driversLine(driverPart)}</span
								>{/if}
							{#if !checked && warning}<span class="block text-gone" data-testid="no-answer-warning"
									>{warning}</span
								>{/if}
							{#if checked && settled}<span class="block" data-testid="settled">{settled}</span
								>{/if}
						{/snippet}
					</MarkRow>
				{/if}
			</AdminPanel>
		{/if}

		{#each placesWarnings(page.game) as warning (warning)}
			<p class="text-sm" role="status" data-testid="places-warning">{warning}</p>
		{/each}

		{#if notSaved}
			<p class="text-sm" role="alert" data-testid="not-saved">Not saved. Try again.</p>
		{/if}
		{#if error}
			<p class="text-sm" role="alert" data-testid="save-error">{error}</p>
		{/if}

		{#if page.attendance && (page.not_played || page.attendance.checked || page.attendance.before_release) && !listShown}
			<div class="flex flex-wrap items-center gap-3" data-testid="closed-list">
				{#if countText(page.attendance.count, page.started)}
					<p class="text-sm font-semibold" data-testid="closed-count">
						{countText(page.attendance.count, page.started)}
					</p>
				{/if}
				<Button variant="outline" onclick={() => (listShown = true)} data-testid="show-list">
					Show the list
				</Button>
			</div>
		{:else if page.attendance}
			{#key `${page.game.id}-${page.attendance.checked}`}
				<GameAttendance
					season={page.season}
					id={String(page.game.id)}
					date={page.game.date}
					started={page.started}
					attendance={page.attendance}
					onanswers={(list) => (shown = { page, answers: list })}
				/>
			{/key}
		{/if}

		{#if schedule}
			<div class="space-y-3 rounded-xl border bg-card p-4 shadow-xs" data-testid="schedule">
				<h2 class="font-medium">Table and referee</h2>
				{#key saves}
					<div class="grid grid-cols-2 gap-2">
						{#each schedule.places as place (`${place.kind}-${place.number}`)}
							{@const warning = `place-plays-${place.kind}-${place.number}`}
							<div class="min-w-0 space-y-1" data-testid="place">
								<label class="block space-y-1">
									<span class="text-sm">{placeNames[place.kind]} {place.number}</span>
									<select
										class={field}
										disabled={busy}
										aria-describedby={place.plays ? warning : undefined}
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
								{#if place.plays}
									<p id={warning} class="text-xs text-gone" data-testid="place-plays">
										This team plays this game.
									</p>
								{/if}
							</div>
						{/each}
					</div>
				{/key}
				<p class="text-sm font-medium" data-testid="team-places">
					{teamPlacesText(page.team, schedule)}
				</p>
			</div>
		{/if}

		{#if teamPlaces(page).length > 0}
			<div class="space-y-3 rounded-xl border bg-card p-4 shadow-xs" data-testid="team-duties">
				<h2 class="font-medium">Players on the places</h2>
				<ul class="space-y-2">
					{#each teamPlaces(page) as place (`${place.kind}-${place.number}`)}
						{@const name = placeName(place.kind, place.number)}
						{@const on = place.player}
						{@const doer = on ? doerOf(on) : null}
						{@const onChoice = page.players.find((player) => player.id === doer)}
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
									{on
										? playerText(on) +
											driverNote(on, notComingDriver(place.kind, on.id), page.started)
										: 'open'}
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
									{#if on && page.started && !correcting}
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
									{#if !on || !page.started || correcting}
										<ul class="space-y-1" aria-label="Choose a player for {name}">
											{#each page.players as player (player.id)}
												{@const reason = notOffered(player, place, correcting)}
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
														{#if notComingDriver(place.kind, player.id)}
															<span class="block text-xs" data-testid="choice-not-coming"
																>Not coming</span
															>
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

		{#if !page.not_played}
			<div>
				<Button
					variant="outline"
					disabled={busy}
					onclick={() => save(page, (id) => setNotPlayed(page.season, id, true))}
					data-testid="set-not-played"
				>
					Mark as not played
				</Button>
			</div>
		{/if}
	</section>
{/if}
