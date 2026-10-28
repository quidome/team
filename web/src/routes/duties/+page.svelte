<!--
	Duties (story B of duties): the table "Totals" of the chosen season (the current one by default,
	`?season=` in the address): one row for each player with the role "plays games" in the season or
	with a duty in it, and for Drive, Table and Referee the duties done and planned ("3 · 1").
-->
<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { Button } from '#lib/components/ui/button/index.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const overview = $derived(data.overview);

	function chooseSeason(season: string) {
		goto(resolve(`/duties?season=${encodeURIComponent(season)}`));
	}

	const kinds = [
		{ key: 'drive', name: 'Drive' },
		{ key: 'table', name: 'Table' },
		{ key: 'referee', name: 'Referee' }
	] as const;
</script>

<svelte:head>
	<title>Duties · Team</title>
</svelte:head>

{#if data.invalidSeason}
	<section class="space-y-4 py-16 text-center">
		<p data-testid="invalid-season">This season cannot be read.</p>
		<Button href={resolve('/duties')} variant="outline">Current season</Button>
	</section>
{:else if !overview}
	<section class="py-16 text-center">
		<p class="text-muted-foreground">Log in to see the duties.</p>
		<Button href="/auth/login" class="mt-4" data-sveltekit-reload>Log in</Button>
	</section>
{:else}
	{@const anyDuty = overview.rows.some((row) =>
		kinds.some(({ key }) => row.totals[key].done + row.totals[key].planned > 0)
	)}
	<section class="space-y-6">
		<div class="flex items-center justify-between gap-3">
			<h1 class="text-2xl font-semibold tracking-tight">Duties · {overview.season}</h1>
			<Button href={resolve('/')} variant="ghost">Back</Button>
		</div>

		{#if overview.seasons.length > 1}
			<label class="flex items-center gap-3 text-sm">
				Season
				<select
					class="h-11 min-w-0 rounded-md border border-input bg-background px-3 text-base shadow-xs focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
					value={overview.season}
					onchange={(event) => chooseSeason(event.currentTarget.value)}
					data-testid="season-choice"
				>
					{#each overview.seasons as season (season)}
						<option value={season}>{season}</option>
					{/each}
				</select>
			</label>
		{/if}

		<section aria-labelledby="totals-heading" class="space-y-2">
			<h2 id="totals-heading" class="text-lg font-semibold tracking-tight">Totals</h2>
			{#if !anyDuty}
				<p class="text-sm text-muted-foreground" data-testid="no-duties-yet">
					No duties yet this season.
				</p>
			{:else}
				<p class="text-sm text-muted-foreground">Done · planned</p>
				<div class="rounded-xl border bg-card shadow-xs">
					<table class="w-full text-sm" data-testid="duty-totals">
						<thead>
							<tr class="border-b text-left">
								<th scope="col" class="px-3 py-2 font-medium">Player</th>
								{#each kinds as kind (kind.key)}
									<th scope="col" class="px-2 py-2 text-right font-medium">{kind.name}</th>
								{/each}
							</tr>
						</thead>
						<tbody>
							{#each overview.rows as row (row.player)}
								<tr class="border-b last:border-b-0" data-testid="duty-row">
									<th scope="row" class="max-w-0 truncate px-3 py-2 text-left font-medium">
										{row.name}
									</th>
									{#each kinds as kind (kind.key)}
										<td class="px-2 py-2 text-right whitespace-nowrap tabular-nums">
											{row.totals[kind.key].done} · {row.totals[kind.key].planned}
										</td>
									{/each}
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			{/if}
		</section>
	</section>
{/if}
