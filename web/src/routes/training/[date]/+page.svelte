<!--
	One training (attendance at training, story B): the date and times, "N absent", "All there" or
	"Cancelled", and the players with a role on the date. A date that is not opened shows "Not
	opened" with "Open training". (Marking comes in step 4.)
-->
<script lang="ts">
	import { resolve } from '$app/paths';
	import { ApiError, openTraining, type TrainingDay } from '#lib/api/client.js';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import { fullName, roleLabels } from '#lib/players.js';
	import { formatDay } from '#lib/schedule.js';
	import { untrack } from 'svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	// Replaced by the answer of each action.
	let day = $state<TrainingDay | null>(untrack(() => data.day));

	let busy = $state(false);
	let error = $state<string | null>(null);

	function message(failure: unknown): string {
		return failure instanceof ApiError
			? failure.message
			: 'Something went wrong. Please try again.';
	}

	async function open() {
		if (!day) return;
		busy = true;
		error = null;
		try {
			day = await openTraining(day.date);
		} catch (failure) {
			error = message(failure);
		} finally {
			busy = false;
		}
	}

	function reasonName(page: TrainingDay, id: number): string {
		return page.reasons.find((reason) => reason.id === id)?.name ?? '';
	}

	/** "Tue 6 Oct · 19:45–21:00", or only the date while not opened. */
	function heading(page: TrainingDay): string {
		const date = formatDay(page.date);
		return page.training ? `${date} · ${page.training.start}–${page.training.end}` : date;
	}

	function topLine(page: TrainingDay): string {
		if (!page.training) return 'Not opened';
		if (page.training.cancelled) return 'Cancelled';
		return page.absent === 0 ? 'All there' : `${page.absent} absent`;
	}
</script>

<svelte:head>
	<title>Training · Team</title>
</svelte:head>

{#if !day}
	<section class="py-16 text-center">
		<p class="text-muted-foreground">Log in to see the training.</p>
		<Button href="/auth/login" class="mt-4" data-sveltekit-reload>Log in</Button>
	</section>
{:else}
	{@const page = day}
	<section class="space-y-6">
		<div class="flex items-start justify-between gap-3">
			<div class="min-w-0">
				<h1 class="text-2xl font-semibold tracking-tight" data-testid="training-heading">
					{heading(page)}
				</h1>
				<p class="mt-1 text-sm text-muted-foreground" data-testid="training-top">
					{topLine(page)}
				</p>
			</div>
			<Button href={resolve('/attendance')} variant="ghost">Back</Button>
		</div>

		{#if error}
			<p class="text-sm" role="alert">{error}</p>
		{/if}

		{#if !page.training}
			<div class="space-y-3 rounded-xl border bg-card p-4 shadow-xs">
				<p class="text-sm">
					This training is not opened. Opening it makes it count, with every player there.
				</p>
				<Button disabled={busy} onclick={open}>Open training</Button>
			</div>
		{:else if page.players.length === 0}
			<p class="text-sm text-muted-foreground" data-testid="no-players">
				No player has a role on this date.
			</p>
		{:else}
			<ul class="space-y-2" aria-label="Players">
				{#each page.players as player (player.id)}
					<li
						class="flex min-h-12 items-center justify-between gap-3 rounded-xl border bg-card px-4 py-2 shadow-xs"
						data-testid="training-player"
					>
						<span class="min-w-0 truncate font-medium">{fullName(player)}</span>
						{#if player.reason !== null && player.reason !== undefined}
							<span class="shrink-0 text-sm" data-testid="player-reason">
								{reasonName(page, player.reason)}
							</span>
						{:else}
							<Badge variant="secondary">{roleLabels[player.role]}</Badge>
						{/if}
					</li>
				{/each}
			</ul>
		{/if}
	</section>
{/if}
