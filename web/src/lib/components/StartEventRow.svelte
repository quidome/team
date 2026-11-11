<!--
	One event of the start page (the quieter start page, story 2): a row without a card, with the
	start time on the left. A game of the team has a soft orange background and an orange bar
	("Home game against …", "Away game at …", the hall and town, the count, the places); another
	game with places of the team a thin blue bar (the two teams and the places); a training no bar.
	An event with nothing left to do (`look` "lighter", from the server) is grey. A tap opens the
	game (with `gameSeason`) or the training.
-->
<script lang="ts">
	import type { StartEvent, TrainingRow } from '#lib/api/client.js';
	import { countText } from '#lib/attendance.js';
	import { placesWarnings, rowPlacesText } from '#lib/duties.js';
	import { formatDay } from '#lib/schedule.js';
	import { gameLink, gameTitle } from '#lib/start.js';
	import { checkedText, statusText } from '#lib/trainings.js';

	let {
		item,
		busy,
		onOpen,
		gameSeason
	}: {
		item: StartEvent;
		busy: boolean;
		onOpen: (row: TrainingRow) => void;
		/** The season of the game: a tap opens its page. No link without it. */
		gameSeason?: string;
	} = $props();

	const lighter = $derived(item.look === 'lighter');
	const rowClass =
		'grid min-h-11 w-full grid-cols-[0.25rem_3.25rem_minmax(0,1fr)] gap-x-3 px-2 py-3 text-left';
</script>

{#if item.event.game}
	{@const game = item.event.game}
	{@const ofTeam = game.reason === 'team'}
	{@const gone = !game.in_list}
	{@const count =
		ofTeam && !game.not_played && game.attendance ? countText(game.attendance, game.started) : ''}
	<li
		class={['border-b', ofTeam && 'rounded-lg border-b-transparent bg-team-soft']}
		data-testid="game"
		data-id={game.id}
		data-of-team={ofTeam}
		data-look={item.look}
	>
		{#snippet content()}
			<span
				class={['rounded-full', ofTeam ? 'bg-team' : 'bg-duty', lighter && 'opacity-40']}
				aria-hidden="true"
			></span>
			<span
				class={['pt-px text-sm tabular-nums', lighter ? 'text-muted-foreground' : 'font-medium']}
			>
				{#if game.start_time}
					{game.start_time}
				{:else}
					<span class="text-xs text-muted-foreground italic">time not known yet</span>
				{/if}
			</span>
			<span class={['min-w-0 space-y-0.5', lighter && 'text-muted-foreground']}>
				<span
					class={['block leading-snug', !lighter && 'font-semibold', gone && 'line-through']}
					data-testid="game-title"
				>
					{gameTitle(item)}
				</span>
				{#if ofTeam}
					<span class="block text-sm text-muted-foreground">
						{game.hall}, {game.town}
						{#if game.not_played}· <span data-testid="game-not-played">Not played</span>{/if}
						{#if game.checked}· <span data-testid="game-checked">Checked</span>{/if}
					</span>
					{#if count}
						<span class="block text-sm" data-testid="game-count">{count}</span>
					{/if}
				{:else if game.not_played || game.checked}
					<span class="block text-sm">
						{#if game.not_played}<span data-testid="game-not-played">Not played</span>{/if}
						{#if game.not_played && game.checked}·{/if}
						{#if game.checked}<span data-testid="game-checked">Checked</span>{/if}
					</span>
				{/if}
				{#if game.places}
					<span class="block text-sm" data-testid="game-places">{rowPlacesText(game.places)}</span>
				{/if}
				{#each placesWarnings(game).filter((warning) => warning !== 'no longer in the game list') as warning (warning)}
					<span class="block text-sm text-gone" data-testid="places-warning">{warning}</span>
				{/each}
				{#if gone}
					<span class="block text-sm text-gone">no longer in the game list</span>
				{/if}
			</span>
		{/snippet}
		{#if gameSeason}
			<!-- eslint-disable svelte/no-navigation-without-resolve -- gameLink() builds the address with resolve() -->
			<a href={gameLink(gameSeason, game.id)} class={rowClass} data-testid="game-link">
				{@render content()}
			</a>
			<!-- eslint-enable svelte/no-navigation-without-resolve -->
		{:else}
			<div class={rowClass}>{@render content()}</div>
		{/if}
	</li>
{:else if item.event.training}
	{@const row = item.event.training}
	<li class="border-b" data-testid="training-row" data-date={row.date} data-look={item.look}>
		<button
			type="button"
			class={[rowClass, 'disabled:opacity-60']}
			disabled={busy}
			onclick={() => onOpen(row)}
		>
			<span aria-hidden="true"></span>
			<span
				class={['pt-px text-sm tabular-nums', lighter ? 'text-muted-foreground' : 'font-medium']}
			>
				<span class="sr-only">{formatDay(row.date)}</span>
				{row.start}
			</span>
			<span
				class={[
					'flex min-w-0 flex-wrap justify-between gap-x-3',
					lighter && 'text-muted-foreground'
				]}
			>
				<span class={!lighter ? 'font-semibold' : ''}>
					Training
					{#if row.extra}<span class="text-muted-foreground">· extra</span>{/if}
				</span>
				<span class="text-sm" data-testid="training-status">
					{statusText(row)}{checkedText(row)}
				</span>
			</span>
		</button>
	</li>
{/if}
