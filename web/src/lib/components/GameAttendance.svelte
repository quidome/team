<!--
	Who comes to a game of the team (story A of attendance at games): the count, which stays in view,
	the groups "No answer", "Coming" and "Not coming", and "Extra players". Each row has "Coming" and
	"Not coming"; a tap saves at once, a tap on the chosen one goes back to "no answer". A row with
	"not coming" has the link "Reason". The rows keep the order of the first load while the page is
	open. The taps of one player are sent one after the other; a tap that fails shows the stored
	state again with "Not saved. Try again." Above the count, "Too few players: 6 of 7" (story B), and
	for answers from before a move the note with "Mark answers as checked". From the start of the game
	(`started`) the words are "There" and "Absent" (story C); on a checked game a tap on the chosen
	state does nothing.
	`onanswers` gives the page the answer shown of each player, for the notes at the driver places;
	`oncount` the count shown, for the row "Not checked" of the panel "Administration".
-->
<script lang="ts">
	import {
		clearAnswer,
		loadGame,
		setAnswer,
		setAnswersChecked,
		setExtraPlayers,
		type AnswerView,
		type AttendanceView,
		type CountView,
		type GameAttendanceView
	} from '#lib/api/client.js';
	import { countText } from '#lib/attendance.js';
	import { formatDay } from '#lib/schedule.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import { untrack } from 'svelte';

	let {
		season,
		id,
		date,
		started,
		attendance,
		onanswers,
		oncount
	}: {
		season: string;
		id: string;
		/** The date of the game now. */
		date: string;
		/** The game has started: the words "There" and "Absent". */
		started: boolean;
		attendance: GameAttendanceView;
		onanswers?: (answers: Record<number, AnswerView | null>) => void;
		/** The count shown, after each tap: the row "Not checked" of the page follows it. */
		oncount?: (count: CountView) => void;
	} = $props();

	type Row = { answer: AnswerView | null; reason: number | null };
	type Rows = Record<number, Row>;

	/** The queue of the extra players; a player's queue has the id of the player. */
	const EXTRA = 0;
	/** The queue of "Mark answers as checked". */
	const CHECKED = -1;

	// The first load: later answers of the server change the states, not the order.
	const first = untrack(() => attendance);
	const reasons = first.reasons;
	const words = untrack(() =>
		started
			? { coming: 'There', notComing: 'Absent' }
			: { coming: 'Coming', notComing: 'Not coming' }
	);
	const groups = (
		[
			['No answer', null],
			[words.coming, 'coming'],
			[words.notComing, 'not_coming']
		] as const
	)
		.map(([title, answer]) => ({
			title,
			rows: first.rows.filter((row) => (row.answer ?? null) === answer)
		}))
		.filter((group) => group.rows.length > 0);

	const firstRows = (): Rows =>
		Object.fromEntries(
			first.rows.map((row) => [
				row.player,
				{ answer: row.answer ?? null, reason: row.reason ?? null }
			])
		);
	/** The states the server stored. */
	let stored = firstRows();
	let storedExtra = first.extra_players;
	/** The states on the screen: a tap shows at once, before its answer. */
	let shown = $state(firstRows());
	let extra = $state(first.extra_players);
	let count = $state(first.count);
	let tooFew = $state(first.too_few);
	let moved = $state(first.answers_moved);

	$effect(() => {
		oncount?.(count);
	});

	$effect(() => {
		onanswers?.(
			Object.fromEntries(Object.entries(shown).map(([player, row]) => [Number(player), row.answer]))
		);
	});

	/** The player whose reasons are open. */
	let picking = $state<number | null>(null);
	/** For each queue, the request that runs now; the next tap waits for it. */
	const queues: Record<number, Promise<void>> = {};
	/** For each queue, the number of its newest tap. */
	const newest: Record<number, number> = {};
	let waiting = 0;
	/** Two queues ran at once: the count of the last answer may be the older one. */
	let overlapped = false;
	/** Counts every tap, so a reload sent before a tap is not trusted. */
	let taps = 0;
	/** For each queue, the taps still waiting for an answer: "saving…" shows. */
	let saving = $state<Record<number, number>>({});
	let notSaved = $state(false);

	/** Runs `work` in the queue `key`: `done` with its answer, `failed` when it fails; both are told
	 * whether this is the newest tap, which decides what the screen shows. */
	function enqueue(
		key: number,
		work: () => Promise<AttendanceView>,
		done: (answer: AttendanceView, last: boolean) => void,
		failed: () => void
	) {
		notSaved = false;
		const tap = (newest[key] ?? 0) + 1;
		newest[key] = tap;
		taps += 1;
		if (waiting > 0) overlapped = true;
		waiting += 1;
		saving = { ...saving, [key]: (saving[key] ?? 0) + 1 };
		queues[key] = (queues[key] ?? Promise.resolve()).then(async () => {
			try {
				const answer = await work();
				// The count shows when no tap is waiting.
				if (waiting === 1) {
					count = answer.count;
					tooFew = answer.too_few;
					moved = answer.answers_moved;
				}
				done(answer, newest[key] === tap);
			} catch {
				if (newest[key] === tap) failed();
				notSaved = true;
			} finally {
				waiting -= 1;
				saving = { ...saving, [key]: (saving[key] ?? 1) - 1 };
				if (waiting === 0 && overlapped) await refreshCount();
			}
		});
	}

	async function refreshCount() {
		overlapped = false;
		const before = taps;
		try {
			const page = await loadGame(fetch, season, id);
			if (taps === before && waiting === 0 && page.attendance) {
				count = page.attendance.count;
				tooFew = page.attendance.too_few;
				moved = page.attendance.answers_moved;
			}
		} catch {
			// The page keeps what it shows.
		}
	}

	function save(player: number, next: Row) {
		shown[player] = next;
		enqueue(
			player,
			() =>
				next.answer === null
					? clearAnswer(season, id, player)
					: setAnswer(season, id, player, next.answer, next.reason),
			(answer, last) => {
				const row = {
					answer: answer.row?.answer ?? null,
					reason: answer.row?.reason ?? null
				};
				stored[player] = row;
				if (last) shown[player] = row;
			},
			() => (shown[player] = stored[player])
		);
	}

	function tapAnswer(player: number, answer: AnswerView) {
		const row = shown[player];
		picking = null;
		// On a checked game a player is "there" or "absent": a tap on the chosen state does nothing.
		if (first.checked && row.answer === answer) return;
		save(
			player,
			row.answer === answer ? { answer: null, reason: null } : { answer, reason: row.reason }
		);
	}

	function tapReason(player: number, reason: number) {
		const row = shown[player];
		save(player, { answer: row.answer, reason: row.reason === reason ? null : reason });
	}

	function tapExtra(next: number) {
		extra = next;
		enqueue(
			EXTRA,
			() => setExtraPlayers(season, id, next),
			(answer, last) => {
				storedExtra = answer.extra_players;
				if (last) extra = answer.extra_players;
			},
			() => (extra = storedExtra)
		);
	}

	function tapAnswersChecked() {
		enqueue(
			CHECKED,
			() => setAnswersChecked(season, id),
			(answer) => (moved = answer.answers_moved),
			() => {}
		);
	}

	/** The reasons close at a tap on another row. */
	function closeReasons(event: MouseEvent) {
		if (picking === null) return;
		const row = (event.target as Element).closest<HTMLElement>('[data-player]');
		if (row && Number(row.dataset.player) !== picking) picking = null;
	}

	function reasonName(id: number): string {
		return reasons.find((reason) => reason.id === id)?.name ?? '';
	}
</script>

<svelte:window onclick={closeReasons} />

<div class="space-y-3" data-testid="game-attendance">
	<div
		class="sticky top-[calc(3.5rem+env(safe-area-inset-top))] z-[1] -mx-4 bg-background/95 px-4 py-2 backdrop-blur"
	>
		{#if tooFew}
			<p class="text-sm font-semibold" data-testid="too-few">
				Too few players: {count.coming} of {first.enough_players}
			</p>
		{/if}
		<p class="min-h-5 text-sm font-semibold" data-testid="attendance-count">
			{countText(count, started)}
		</p>
		{#if notSaved}
			<p class="text-sm" role="alert" data-testid="attendance-not-saved">Not saved. Try again.</p>
		{/if}
	</div>

	{#if moved}
		<div
			class="flex flex-wrap items-center gap-3 rounded-xl border bg-card p-3"
			data-testid="answers-moved"
		>
			<p class="text-sm">Answers from before the move to {formatDay(date)}.</p>
			<Button variant="outline" disabled={(saving[CHECKED] ?? 0) > 0} onclick={tapAnswersChecked}>
				Mark answers as checked
			</Button>
		</div>
	{/if}

	{#if first.rows.length === 0}
		<p class="text-sm text-muted-foreground" data-testid="no-list-players">
			No players on this date.
		</p>
	{/if}

	{#each groups as group (group.title)}
		<div class="space-y-2" data-testid="attendance-group">
			<h2 class="text-sm font-semibold text-muted-foreground">{group.title}</h2>
			<ul class="space-y-2" aria-label={group.title}>
				{#each group.rows as player (player.player)}
					{@const row = shown[player.player]}
					{@const offered = reasons.filter((reason) => !reason.hidden || reason.id === row.reason)}
					<li
						class="space-y-2 rounded-xl border bg-card px-4 py-2 shadow-xs"
						data-testid="attendance-row"
						data-player={player.player}
					>
						<div class="flex items-center justify-between gap-2">
							<span class="min-w-0 truncate font-medium">
								{player.name}
								{#if player.hidden}
									<span class="text-xs font-normal text-muted-foreground">(hidden)</span>
								{/if}
							</span>
							<span class="flex shrink-0 items-center gap-2">
								{#if (saving[player.player] ?? 0) > 0}
									<span class="text-xs text-muted-foreground" data-testid="saving">saving…</span>
								{/if}
								<Button
									size="lg"
									variant={row.answer === 'coming' ? 'default' : 'outline'}
									aria-pressed={row.answer === 'coming'}
									aria-label="{words.coming}: {player.name}"
									onclick={() => tapAnswer(player.player, 'coming')}
								>
									{words.coming}
								</Button>
								<Button
									size="lg"
									variant={row.answer === 'not_coming' ? 'default' : 'outline'}
									aria-pressed={row.answer === 'not_coming'}
									aria-label="{words.notComing}: {player.name}"
									onclick={() => tapAnswer(player.player, 'not_coming')}
								>
									{words.notComing}
								</Button>
							</span>
						</div>
						{#if row.answer === 'not_coming' && offered.length > 0}
							<button
								type="button"
								class="min-h-8 text-sm font-medium underline underline-offset-4"
								aria-expanded={picking === player.player}
								onclick={() => (picking = picking === player.player ? null : player.player)}
							>
								{row.reason === null ? 'Reason' : `Reason: ${reasonName(row.reason)}`}
							</button>
							{#if picking === player.player}
								<div
									class="flex flex-wrap gap-2"
									data-testid="reasons"
									role="group"
									aria-label="Reasons for {player.name}"
								>
									{#each offered as option (option.id)}
										<Button
											variant={option.id === row.reason ? 'default' : 'outline'}
											aria-pressed={option.id === row.reason}
											onclick={() => tapReason(player.player, option.id)}
										>
											{option.name}
										</Button>
									{/each}
								</div>
							{/if}
						{/if}
					</li>
				{/each}
			</ul>
		</div>
	{/each}

	<div
		class="flex items-center justify-between gap-3 rounded-xl border bg-card px-4 py-2 shadow-xs"
		data-testid="extra-players"
	>
		<span class="font-medium">Extra players</span>
		<span class="flex items-center gap-2">
			{#if (saving[EXTRA] ?? 0) > 0}
				<span class="text-xs text-muted-foreground" data-testid="saving">saving…</span>
			{/if}
			<Button
				size="icon-lg"
				variant="outline"
				aria-label="One extra player less"
				disabled={extra <= 0}
				onclick={() => tapExtra(extra - 1)}
			>
				−
			</Button>
			<span class="w-6 text-center font-medium" data-testid="extra-number">{extra}</span>
			<Button
				size="icon-lg"
				variant="outline"
				aria-label="One extra player more"
				disabled={extra >= first.max_extra_players}
				onclick={() => tapExtra(extra + 1)}
			>
				+
			</Button>
		</span>
	</div>
</div>
