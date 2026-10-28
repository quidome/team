<!--
	The start page (story B of start page and season): what still needs the coordinator ("Open
	items"), the last training and the last games with duty places ("Last"), and the games of the
	team, the games with places of the team and the trainings of this week and next week ("Next"),
	then the links. An open item of more games opens their list below it. Without a club: only the choose step; in a new season without
	a team: the choose step above the three parts. Not logged in: "Log in".
-->
<script lang="ts">
	import { goto, invalidateAll } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { ApiError, openTraining, type EventView, type TrainingRow } from '#lib/api/client.js';
	import ChooseTeam from '#lib/components/ChooseTeam.svelte';
	import EventRow from '#lib/components/EventRow.svelte';
	import { Button } from '#lib/components/ui/button/index.js';
	import { formatDay } from '#lib/schedule.js';
	import { groupByDate } from '#lib/season.js';
	import { gameItemText, gameLink, openItemLink, openItemText, trainingLink } from '#lib/start.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	let busy = $state(false);
	/** The open item whose list of games is open. */
	let expanded = $state<string | null>(null);

	/** "Last": the last training and the last games, newest first. */
	function lastEvents(last: EventView | null | undefined, games: EventView[]): EventView[] {
		return [...(last ? [last] : []), ...games].sort((a, b) => b.date.localeCompare(a.date));
	}
	let error = $state<string | null>(null);

	/** A tap on a training: open it when it is not opened yet, then show it; "Back" returns here. */
	async function open(row: TrainingRow) {
		busy = true;
		error = null;
		try {
			if (!row.opened) await openTraining(row.date);
			// eslint-disable-next-line svelte/no-navigation-without-resolve -- trainingLink() builds the address with resolve()
			await goto(trainingLink(row.date));
		} catch (failure) {
			error =
				failure instanceof ApiError ? failure.message : 'Something went wrong. Please try again.';
		} finally {
			busy = false;
		}
	}
</script>

<svelte:head>
	<title>Team</title>
</svelte:head>

{#snippet links()}
	<nav aria-label="Team" class="text-sm" data-testid="team-links">
		<a href={resolve('/season')} class="font-medium underline underline-offset-4">Season</a>
		<span class="text-muted-foreground" aria-hidden="true">·</span>
		<a href={resolve('/attendance')} class="font-medium underline underline-offset-4">Attendance</a>
		<span class="text-muted-foreground" aria-hidden="true">·</span>
		<a href={resolve('/duties')} class="font-medium underline underline-offset-4">Duties</a>
		<span class="text-muted-foreground" aria-hidden="true">·</span>
		<a href={resolve('/team')} class="font-medium underline underline-offset-4">Team</a>
		<span class="text-muted-foreground" aria-hidden="true">·</span>
		<a href={resolve('/players')} class="font-medium underline underline-offset-4">Players</a>
		<span class="text-muted-foreground" aria-hidden="true">·</span>
		<a href={resolve('/settings')} class="font-medium underline underline-offset-4">Settings</a>
	</nav>
{/snippet}

{#snippet parts()}
	{#if data.startError}
		<p class="text-sm" role="alert" data-testid="start-error">
			The start page could not be loaded. Load the page again.
		</p>
	{:else if data.start}
		{@const start = data.start}
		<section aria-labelledby="open-items-heading" class="space-y-2" data-testid="open-items">
			<h2 id="open-items-heading" class="text-lg font-semibold tracking-tight">Open items</h2>
			{#if start.open_items.length === 0}
				<p class="text-sm text-muted-foreground" data-testid="nothing-open">Nothing open.</p>
			{:else}
				<ul class="space-y-2">
					{#each start.open_items as item (item.kind)}
						{@const link = openItemLink(item)}
						{@const itemClass =
							'flex min-h-12 w-full items-center rounded-xl border bg-card px-4 py-2 text-left text-sm font-medium shadow-xs'}
						<li>
							{#if link}
								<!-- eslint-disable svelte/no-navigation-without-resolve -- openItemLink() builds the address with resolve() -->
								<a href={link} class={itemClass} data-testid="open-item" data-kind={item.kind}>
									{openItemText(item)}
								</a>
								<!-- eslint-enable svelte/no-navigation-without-resolve -->
							{:else}
								<button
									type="button"
									class={itemClass}
									aria-expanded={expanded === item.kind}
									aria-controls="open-item-{item.kind}"
									data-testid="open-item"
									data-kind={item.kind}
									onclick={() => (expanded = expanded === item.kind ? null : item.kind)}
								>
									{openItemText(item)}
								</button>
								{#if expanded === item.kind && (item.kind === 'schedule_not_entered' || item.kind === 'open_places')}
									<ul id="open-item-{item.kind}" class="mt-2 space-y-1 pl-4">
										{#each item.games as game (game.id)}
											<li>
												<!-- eslint-disable svelte/no-navigation-without-resolve -- gameLink() builds the address with resolve() -->
												<a
													href={gameLink(game.season, game.id)}
													class="flex min-h-11 items-center text-sm underline underline-offset-4"
													data-testid="open-item-game"
												>
													{gameItemText(game)}
												</a>
												<!-- eslint-enable svelte/no-navigation-without-resolve -->
											</li>
										{/each}
									</ul>
								{/if}
							{/if}
						</li>
					{/each}
				</ul>
			{/if}
		</section>

		{#if error}
			<p class="text-sm" role="alert">{error}</p>
		{/if}

		{@const last = lastEvents(start.last, start.last_games)}
		{#if last.length > 0}
			<section aria-labelledby="last-heading" class="space-y-2" data-testid="last">
				<h2 id="last-heading" class="text-lg font-semibold tracking-tight">Last</h2>
				{#each groupByDate(last) as day (day.date)}
					<section aria-labelledby="last-day-{day.date}">
						<h3
							id="last-day-{day.date}"
							class="py-1 text-sm font-semibold text-muted-foreground"
							data-testid="day"
						>
							{formatDay(day.date)}
						</h3>
						<ul class="space-y-2">
							{#each day.events as event (event.game ? `g${event.game.id}` : `t${event.date}`)}
								<EventRow
									{event}
									{busy}
									onOpen={open}
									gameSeason={start.last_games_season ?? undefined}
									gameFrom="start"
								/>
							{/each}
						</ul>
					</section>
				{/each}
			</section>
		{/if}

		<section aria-labelledby="next-heading" class="space-y-2" data-testid="next">
			<h2 id="next-heading" class="text-lg font-semibold tracking-tight">Next</h2>
			{#if start.next.length === 0}
				<p class="text-sm text-muted-foreground" data-testid="no-coming-events">
					No coming events.
				</p>
			{:else}
				<div class="space-y-4">
					{#each groupByDate(start.next) as day (day.date)}
						<section aria-labelledby="next-day-{day.date}">
							<h3
								id="next-day-{day.date}"
								class="py-1 text-sm font-semibold text-muted-foreground"
								data-testid="day"
							>
								{formatDay(day.date)}
							</h3>
							<ul class="space-y-2">
								{#each day.events as event (event.game ? `g${event.game.id}` : `t${event.date}`)}
									<EventRow
										{event}
										{busy}
										onOpen={open}
										gameSeason={data.selection?.season}
										gameFrom="start"
									/>
								{/each}
							</ul>
						</section>
					{/each}
				</div>
			{/if}
		</section>
	{/if}
{/snippet}

{#if data.selection?.state === 'not_chosen'}
	<section class="space-y-5">
		<div>
			<h1 class="text-2xl font-semibold tracking-tight">Choose your club and team</h1>
			<p class="mt-1 text-sm text-muted-foreground">
				The tool reads your team's games from the game list.
			</p>
		</div>
		{@render links()}
		<ChooseTeam onSaved={invalidateAll} />
	</section>
{:else if data.selection?.state === 'team_needed'}
	<section class="space-y-6">
		<div>
			<h1 class="text-2xl font-semibold tracking-tight">Choose your team</h1>
			{#if data.selection.reason === 'new_season'}
				<p class="mt-1 text-sm" data-testid="new-season">
					A new season started: choose the team{data.selection.last?.team ? ' again' : ''}.
				</p>
			{/if}
			{#if data.selection.last?.team}
				<p class="mt-1 text-sm text-muted-foreground" data-testid="last-season">
					In {data.selection.last.season} the team was {data.selection.last.club.name}
					{data.selection.last.team.name}.
				</p>
			{/if}
		</div>
		<ChooseTeam club={data.selection.club} clubKept onSaved={invalidateAll} />
		{@render parts()}
		{@render links()}
	</section>
{:else if data.selection?.state === 'chosen'}
	<section class="space-y-6">
		<div class="flex items-start justify-between gap-3">
			<div class="min-w-0">
				<p class="text-sm text-muted-foreground">Your team · {data.selection.season}</p>
				<h1 class="text-2xl font-semibold tracking-tight" data-testid="chosen-team">
					{data.selection.club.name}
					{data.selection.team.name}
				</h1>
			</div>
			<Button href={resolve('/choose')} variant="outline">Change</Button>
		</div>
		{@render parts()}
		{@render links()}
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
