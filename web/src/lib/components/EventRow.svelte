<!--
	One event of "Season" and the start page: a game (as in the schedule: start time, teams, hall
	and town, the badge "Team", crossed out when no longer in the game list; its duty places and
	what changed for them; a tap opens it when `gameSeason` is given) or a training (start time,
	status, "· Checked", "extra"; a tap opens it). "Cancel" shows on a training day that is not
	opened only when `onCancel` is given.
-->
<script lang="ts">
	import { resolve } from '$app/paths';
	import type { EventView, ScheduleGame, TrainingRow } from '#lib/api/client.js';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import { placesWarnings, rowPlacesText } from '#lib/duties.js';
	import { gameLink } from '#lib/start.js';
	import { formatDay } from '#lib/schedule.js';
	import { checkedText, statusText } from '#lib/trainings.js';
	import MapPin from '@lucide/svelte/icons/map-pin';

	let {
		event,
		busy,
		onOpen,
		onCancel,
		gameSeason,
		gameFrom,
		onGame
	}: {
		event: EventView;
		busy: boolean;
		onOpen: (row: TrainingRow) => void;
		/** The season of the games: a tap on a game opens its page. No link without it. */
		gameSeason?: string;
		/** `start`: the page of a game returns to the start page with "Back". */
		gameFrom?: 'start';
		/** Called before the page of a game opens. */
		onGame?: (game: ScheduleGame) => void;
		/** "Cancel" on a training day that is not opened; no button without it. */
		onCancel?: (row: TrainingRow) => void;
	} = $props();
</script>

{#if event.game}
	{@const game = event.game}
	{@const ofTeam = game.reason === 'team'}
	{@const gone = !game.in_list}
	<li
		class={[
			'flex gap-3 rounded-xl border bg-card p-3 shadow-xs',
			ofTeam && 'border-l-4 border-l-team bg-team-soft',
			gone && 'bg-muted shadow-none'
		]}
		data-testid="game"
		data-id={game.id}
		data-of-team={ofTeam}
	>
		{#snippet content()}
			<div class="w-14 shrink-0 pt-0.5 text-sm font-semibold tabular-nums">
				{#if game.start_time}
					{game.start_time}
				{:else}
					<span class="text-xs leading-tight font-normal text-muted-foreground italic">
						time not known yet
					</span>
				{/if}
			</div>
			<div class="min-w-0 flex-1">
				<p class={['text-sm leading-snug', gone && 'text-muted-foreground line-through']}>
					<span class="block font-medium">{game.home}</span>
					<span class="block font-medium">
						<span class="sr-only">against</span>
						{game.away}
					</span>
				</p>
				<p class="mt-1 flex items-center gap-1 text-xs text-muted-foreground">
					<MapPin class="size-3.5 shrink-0" aria-hidden="true" />
					<span class="truncate">{game.hall}, {game.town}</span>
				</p>
				{#if game.places}
					<p class="mt-1 text-xs" data-testid="game-places">{rowPlacesText(game.places)}</p>
				{/if}
				{#each placesWarnings(game).filter((warning) => warning !== 'no longer in the game list') as warning (warning)}
					<p class="mt-1 text-xs text-gone" data-testid="places-warning">{warning}</p>
				{/each}
				{#if gone}
					<Badge variant="outline" class="mt-2 border-gone/40 text-gone">
						no longer in the game list
					</Badge>
				{/if}
			</div>
			{#if ofTeam}
				<Badge class="self-start bg-team text-primary-foreground">Team</Badge>
			{/if}
		{/snippet}
		{#if gameSeason}
			<!-- eslint-disable svelte/no-navigation-without-resolve -- gameLink() builds the address with resolve() -->
			<a
				href={gameFrom
					? gameLink(gameSeason, game.id)
					: resolve('/game/[season]/[id]', { season: gameSeason, id: String(game.id) })}
				onclick={() => onGame?.(game)}
				class="flex min-w-0 flex-1 gap-3 rounded-lg"
				data-testid="game-link"
			>
				{@render content()}
			</a>
			<!-- eslint-enable svelte/no-navigation-without-resolve -->
		{:else}
			<div class="flex min-w-0 flex-1 gap-3">{@render content()}</div>
		{/if}
	</li>
{:else if event.training}
	{@const row = event.training}
	<li
		class="flex items-stretch gap-2 rounded-xl border bg-card shadow-xs"
		data-testid="training-row"
		data-date={row.date}
	>
		<button
			type="button"
			class="flex min-h-12 min-w-0 flex-1 items-center justify-between gap-3 px-4 py-2 text-left disabled:opacity-60"
			disabled={busy}
			onclick={() => onOpen(row)}
		>
			<span class="min-w-0 tabular-nums">
				<span class="sr-only">{formatDay(row.date)}</span>
				<span class="font-medium">Training</span>
				<span class="text-muted-foreground">· {row.start}</span>
				{#if row.extra}
					<span class="text-muted-foreground">· extra</span>
				{/if}
			</span>
			<span
				class="shrink-0 text-sm {row.status === 'not_opened' || row.status === 'cancelled'
					? 'text-muted-foreground'
					: ''}"
				data-testid="training-status"
			>
				{statusText(row)}{checkedText(row)}
			</span>
		</button>
		{#if onCancel && row.status === 'not_opened'}
			<div class="flex items-center pr-2">
				<Button
					variant="ghost"
					size="sm"
					disabled={busy}
					onclick={() => onCancel?.(row)}
					aria-label="Cancel training of {formatDay(row.date)}"
				>
					Cancel
				</Button>
			</div>
		{/if}
	</li>
{/if}
