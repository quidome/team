<!--
	Report (decision 0033): for the current season, one row for each player with the attendance at
	trainings and games as a percentage and the duties as done + planned, the players who left
	below it, and a line with what does not count yet (training days not opened, games not
	checked). A tap on a name opens the details of that player under the row: there, absent valid
	and absent not valid, and for each duty kind done, planned and the corrections after a game.
	All numbers come from the server; the page calculates nothing.
-->
<script lang="ts">
	import { resolve } from '$app/paths';
	import type { Report, ReportRow } from '#lib/api/client.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import { copyText } from '#lib/copy.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const report = $derived(data.report);

	const kinds = [
		{ key: 'drive', name: 'Drive' },
		{ key: 'table', name: 'Table' },
		{ key: 'referee', name: 'Referee' }
	] as const;

	function share(value: ReportRow['trainings']): string {
		return value ? `${value.percent}%` : '–';
	}

	function duty(row: ReportRow, key: (typeof kinds)[number]['key']): string {
		const total = row.duties?.[key];
		return total ? `${total.done} + ${total.planned}` : '–';
	}

	function count(n: number, one: string, more: string): string {
		return `${n} ${n === 1 ? one : more}`;
	}

	type Column = keyof Report['copies'];
	const columns: { key: Column; name: string }[] = [
		{ key: 'trainings', name: 'Trainings' },
		{ key: 'games', name: 'Games' },
		{ key: 'drive', name: 'Drive' },
		{ key: 'table', name: 'Table' },
		{ key: 'referee', name: 'Referee' }
	];

	// The last "Copy": its column and whether the browser took the text.
	let copied = $state<{ column: Column; state: 'copied' | 'failed' } | null>(null);
	async function copy(column: Column, text: string) {
		copied = { column, state: await copyText(text) };
	}

	// The players whose details are open; more than one can be open.
	let open = $state<number[]>([]);
	function toggle(player: number) {
		open = open.includes(player) ? open.filter((id) => id !== player) : [...open, player];
	}

	type ShareView = NonNullable<ReportRow['trainings']>;

	/** "9 trainings: 6 there (67%) · 2 absent, valid (22%) · 1 absent, not valid (11%)". */
	function shareLine(value: ShareView, one: string, more: string): string {
		return (
			`${count(value.counted, one, more)}: ${value.there} there (${value.percent}%) · ` +
			`${value.absent_valid} absent, valid (${value.percent_valid}%) · ` +
			`${value.absent_not_valid} absent, not valid (${value.percent_not_valid}%)`
		);
	}

	type DutyLine = { name: string; text: string; corrections: string[] };

	/** The duty kinds of `row` with a duty or a correction, each with its corrections. */
	function dutyLines(row: ReportRow): DutyLine[] {
		const duties = row.duties;
		if (!duties) return [];
		return kinds.flatMap(({ key, name }) => {
			const total = duties[key];
			const any =
				total.done + total.planned + total.done_by_other + total.done_for_other + total.not_done;
			if (any === 0) return [];
			const corrections: [number, string][] = [
				[total.done_by_other, 'done by another player'],
				[total.done_for_other, 'done for another player'],
				[total.not_done, 'not done']
			];
			return [
				{
					name,
					text: `${total.done} done · ${total.planned} planned`,
					corrections: corrections.filter(([n]) => n > 0).map(([n, text]) => `${n} ${text}`)
				}
			];
		});
	}
</script>

<svelte:head>
	<title>Report · Team</title>
</svelte:head>

{#snippet table(rows: ReportRow[], testid: string, copies: Report['copies'] | null)}
	<div class="rounded-xl border bg-card shadow-xs">
		<table class="w-full table-fixed text-sm" data-testid={testid}>
			<thead>
				<tr class="border-b text-left">
					<th scope="col" class="w-[28%] px-2 py-2 font-medium"
						><span class="sr-only">Player</span></th
					>
					{#each columns as column (column.key)}
						{@const text = copies?.[column.key]}
						<th
							scope="col"
							class="px-0.5 py-2 text-right align-top text-[11px] font-medium break-words hyphens-auto"
						>
							<span class="block">{column.name}</span>
							{#if text}
								<button
									type="button"
									class="mt-1 min-h-8 rounded-md border px-1.5 text-xs font-normal"
									aria-label={`Copy ${column.name}`}
									onclick={() => copy(column.key, text)}
									data-testid={`copy-${column.key}`}>Copy</button
								>
							{/if}
						</th>
					{/each}
				</tr>
			</thead>
			<tbody>
				{#each rows as row (row.player)}
					{@const shown = open.includes(row.player)}
					{@const duties = dutyLines(row)}
					<tr class="border-b last:border-b-0" data-testid="report-row">
						<th scope="row" class="truncate p-0 text-left font-medium">
							<button
								type="button"
								class="min-h-11 w-full truncate px-2 py-2 text-left underline-offset-4 hover:underline"
								aria-expanded={shown}
								onclick={() => toggle(row.player)}>{row.name}</button
							>
						</th>
						<td class="px-0.5 py-2 text-right text-[13px] whitespace-nowrap tabular-nums"
							>{share(row.trainings)}</td
						>
						<td class="px-0.5 py-2 text-right text-[13px] whitespace-nowrap tabular-nums"
							>{share(row.games)}</td
						>
						{#each kinds as kind (kind.key)}
							<td class="px-0.5 py-2 text-right text-[13px] whitespace-nowrap tabular-nums"
								>{duty(row, kind.key)}</td
							>
						{/each}
					</tr>
					{#if shown}
						<tr class="border-b bg-muted/40 last:border-b-0" data-testid="report-details">
							<td colspan="6" class="space-y-1 px-3 py-2 text-sm">
								<p class="font-medium">{row.name}</p>
								{#if row.trainings}
									<p class="tabular-nums" data-testid="details-trainings">
										{shareLine(row.trainings, 'training', 'trainings')}
									</p>
								{/if}
								{#if row.games}
									<p class="tabular-nums" data-testid="details-games">
										{shareLine(row.games, 'game', 'games')}
									</p>
								{/if}
								{#each duties as line (line.name)}
									<div data-testid="details-duty">
										<p class="tabular-nums">{line.name}: {line.text}</p>
										{#each line.corrections as correction (correction)}
											<p class="pl-4 text-muted-foreground tabular-nums">{correction}</p>
										{/each}
									</div>
								{/each}
								{#if !row.trainings && !row.games && duties.length === 0}
									<p class="text-muted-foreground">Nothing counts yet this season.</p>
								{/if}
								{#if row.trainings || row.games}
									<p>
										<a
											href={resolve(`/attendance?player=${row.player}&from=report`)}
											class="font-medium underline underline-offset-4"
											data-testid="details-missed">Missed trainings and games</a
										>
									</p>
								{/if}
							</td>
						</tr>
					{/if}
				{/each}
			</tbody>
		</table>
	</div>
{/snippet}

{#if data.loadError}
	<p class="text-sm" role="alert" data-testid="report-error">
		The report could not be loaded. Load the page again.
	</p>
{:else if !report}
	<section class="py-16 text-center">
		<p class="text-muted-foreground">Log in to see the report.</p>
		<Button href="/auth/login" class="mt-4" data-sveltekit-reload>Log in</Button>
	</section>
{:else}
	{@const incomplete = report.incomplete}
	<section class="space-y-6">
		<div class="flex items-center justify-between gap-3">
			<h1 class="text-2xl font-semibold tracking-tight">Report {report.season}</h1>
			<Button href={resolve('/')} variant="ghost">Back</Button>
		</div>

		{#if incomplete.not_opened > 0 || incomplete.not_checked > 0}
			<p class="text-sm" data-testid="report-incomplete">
				<!-- eslint-disable svelte/no-navigation-without-resolve -- the path is resolved; only the query `from` is added (resolve() takes no query on a route with a parameter) -->
				{#if incomplete.not_opened > 0 && incomplete.first_not_opened}
					<a
						href={`${resolve('/training/[date]', { date: incomplete.first_not_opened })}?from=report`}
						class="font-medium underline underline-offset-4"
						data-testid="report-not-opened"
					>
						{count(incomplete.not_opened, 'training day not opened', 'training days not opened')}
					</a>
				{/if}
				{#if incomplete.not_opened > 0 && incomplete.not_checked > 0}
					<span class="text-muted-foreground" aria-hidden="true">·</span>
				{/if}
				{#if incomplete.not_checked > 0 && incomplete.first_not_checked}
					<a
						href={`${resolve('/game/[season]/[id]', {
							season: incomplete.first_not_checked.season,
							id: String(incomplete.first_not_checked.id)
						})}?from=report`}
						class="font-medium underline underline-offset-4"
						data-testid="report-not-checked"
					>
						{count(incomplete.not_checked, 'game not checked', 'games not checked')}
					</a>
				{/if}
				<!-- eslint-enable svelte/no-navigation-without-resolve -->
			</p>
		{/if}

		{#if report.team.length === 0 && report.left.length === 0}
			<p class="text-sm text-muted-foreground" data-testid="report-empty">
				No trainings, games or duties yet this season.
			</p>
		{:else}
			{#if report.team.length > 0}
				<!-- The answer to "Copy" is above the table, close to the buttons in its headings. -->
				{#if copied}
					{@const text = report.copies[copied.column]}
					<div class="space-y-2" aria-live="polite" data-testid="copy-state">
						{#if copied.state === 'copied'}
							<p class="text-sm text-muted-foreground">Copied</p>
						{:else if text}
							<p class="text-sm">Select the text and copy it.</p>
							<pre
								class="rounded-xl border bg-card p-4 font-sans text-sm break-words whitespace-pre-wrap shadow-xs select-all"
								data-testid="copy-text">{text}</pre>
						{/if}
					</div>
				{/if}
				{@render table(report.team, 'report-team', report.copies)}
			{/if}
			{#if report.left.length > 0}
				<section aria-labelledby="left-heading" class="space-y-2">
					<h2 id="left-heading" class="text-lg font-semibold tracking-tight">Left the team</h2>
					{@render table(report.left, 'report-left', null)}
				</section>
			{/if}
		{/if}
	</section>
{/if}
