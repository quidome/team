<!--
	The start page (story B of start page and season; the quieter start page): the header (the team
	and today's date, in July and August with the season), what still needs the coordinator (one row
	for each open item; an item of more games opens their list below it), the duty games of the last
	date with duties, then the coming events of this week and next week grouped by date (each row
	from `StartEventRow`), the messages to copy when they have something, and the links. Without a
	club: only the choose step; in a new season without a team: the date and the choose step above
	the parts. Not logged in: "Log in".
-->
<script lang="ts">
	import { goto, invalidateAll } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { ApiError, openTraining, type StartEvent, type TrainingRow } from '#lib/api/client.js';
	import ChooseTeam from '#lib/components/ChooseTeam.svelte';
	import StartEventRow from '#lib/components/StartEventRow.svelte';
	import { Button } from '#lib/components/ui/button/index.js';
	import { formatDay, formatLongDay } from '#lib/schedule.js';
	import {
		gameItemText,
		gameLink,
		groupStartByDate,
		openItemLink,
		openItemText,
		trainingLink
	} from '#lib/start.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	let busy = $state(false);
	/** The open item whose list of games is open. */
	let expanded = $state<string | null>(null);
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

	function key(item: StartEvent): string {
		return item.event.game ? `g${item.event.game.id}` : `t${item.event.date}`;
	}
</script>

<svelte:head>
	<title>Team</title>
</svelte:head>

{#snippet links()}
	<nav aria-label="Team" class="flex flex-wrap gap-x-5 pt-2 text-sm" data-testid="team-links">
		<a href={resolve('/season')} class="py-2.5 underline underline-offset-4">Season</a>
		<a href={resolve('/attendance')} class="py-2.5 underline underline-offset-4">Attendance</a>
		<a href={resolve('/duties')} class="py-2.5 underline underline-offset-4">Duties</a>
		<a href={resolve('/report')} class="py-2.5 underline underline-offset-4">Report</a>
		<a href={resolve('/more')} class="py-2.5 underline underline-offset-4">More</a>
	</nav>
{/snippet}

{#snippet days(events: StartEvent[], season: string | undefined, label: string)}
	{#each groupStartByDate(events) as day (day.date)}
		<section aria-labelledby="{label}-day-{day.date}">
			<h2
				id="{label}-day-{day.date}"
				class="pt-4 pb-1 text-sm font-semibold text-muted-foreground"
				data-testid="day"
			>
				{formatDay(day.date)}
			</h2>
			<ul>
				{#each day.events as item (key(item))}
					<StartEventRow {item} {busy} onOpen={open} gameSeason={season} />
				{/each}
			</ul>
		</section>
	{/each}
{/snippet}

{#snippet parts()}
	{#if data.startError}
		<p class="text-sm" role="alert" data-testid="start-error">
			The start page could not be loaded. Load the page again.
		</p>
	{:else if data.start}
		{@const start = data.start}
		<section aria-label="Open items" data-testid="open-items">
			{#if start.open_items.length === 0}
				<p
					class="flex min-h-12 items-center border-b text-muted-foreground"
					data-testid="nothing-open"
				>
					Nothing needs you.
				</p>
			{:else}
				<ul>
					{#each start.open_items as item (item.kind)}
						{@const link = openItemLink(item)}
						{@const itemClass = [
							'flex min-h-12 w-full items-center gap-3 border-b py-2 text-left font-medium',
							// The dot and the arrow come from CSS, so the text of the row is only its words.
							"before:size-2 before:shrink-0 before:rounded-full before:bg-team before:content-['']",
							'after:ml-auto after:pl-2 after:font-normal after:text-muted-foreground',
							link
								? "after:content-['›']"
								: expanded === item.kind
									? "after:content-['▴']"
									: "after:content-['▾']"
						]}
						<li>
							{#if link}
								<!-- eslint-disable svelte/no-navigation-without-resolve -- openItemLink() builds the address with resolve() -->
								<a href={link} class={itemClass} data-testid="open-item" data-kind={item.kind}
									>{openItemText(item)}</a
								>
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
									>{openItemText(item)}</button
								>
								{#if expanded === item.kind && 'games' in item}
									<ul id="open-item-{item.kind}" class="border-b py-1 pl-5">
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

		<section aria-label="Events" data-testid="next">
			{#if start.last_games.length > 0}
				<div data-testid="last-games">
					{@render days(start.last_games, start.last_games_season ?? undefined, 'last')}
				</div>
			{/if}
			{#if start.next.length === 0}
				<p class="pt-4 text-sm text-muted-foreground" data-testid="no-coming-events">
					No coming events.
				</p>
			{:else}
				{@render days(start.next, data.selection?.season, 'next')}
			{/if}
		</section>

		{#if start.messages.week || start.messages.committee}
			<div class="flex flex-wrap gap-x-6" data-testid="messages">
				{#if start.messages.week}
					<a
						href={resolve('/messages/week')}
						class="py-2.5 font-semibold text-primary underline-offset-4 hover:underline"
						>Message of the week<span aria-hidden="true">&nbsp;›</span></a
					>
				{/if}
				{#if start.messages.committee}
					<a
						href={resolve('/messages/committee')}
						class="py-2.5 font-semibold text-primary underline-offset-4 hover:underline"
						>Committee message<span aria-hidden="true">&nbsp;›</span></a
					>
				{/if}
			</div>
		{/if}
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
		{#if data.start}
			<p class="text-sm text-muted-foreground" data-testid="today">
				{formatLongDay(data.start.today)}
			</p>
		{/if}
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
	<section class="space-y-5">
		<div class="min-w-0">
			<h1 class="text-2xl font-semibold tracking-tight" data-testid="chosen-team">
				{data.selection.club.name}
				{data.selection.team.name}{#if data.start?.show_season}<span
						class="font-normal text-muted-foreground"
						data-testid="season">&nbsp;· {data.selection.season}</span
					>{/if}
			</h1>
			{#if data.start}
				<p class="text-sm text-muted-foreground" data-testid="today">
					{formatLongDay(data.start.today)}
				</p>
			{/if}
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
