<!--
	The team of the current season (attendance at training, story A): the players in the team with
	their role, "Change role" and "Remove from the team", "Left the team" with "Add to the team",
	"Add player", and in a new season the players of last season with "Add all".
-->
<script lang="ts">
	import { invalidateAll } from '$app/navigation';
	import { resolve } from '$app/paths';
	import {
		ApiError,
		addPlayer,
		addRole,
		carryPlayers,
		changeRoleFrom,
		leaveTeam,
		type Member,
		type RoleKind,
		type TeamPage
	} from '#lib/api/client.js';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import { dayAfter, fullName, later, roleLabels } from '#lib/players.js';
	import { formatDay } from '#lib/schedule.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	// Replaced by the answer of each action.
	let team = $derived(data.team);

	/** "new" for a new player, else the id of a player from the list. */
	let adding = $state(false);
	let choice = $state<string>('new');
	let firstName = $state('');
	let lastName = $state('');
	let role = $state<RoleKind>('plays_games');
	let from = $state('');

	/** The open player and the form that is open for it. */
	let open = $state<number | null>(null);
	let action = $state<'change' | 'leave' | 'return' | null>(null);
	let actionRole = $state<RoleKind>('plays_games');
	let actionDate = $state('');

	let busy = $state(false);
	let error = $state<string | null>(null);
	let warning = $state<string | null>(null);

	function message(failure: unknown): string {
		return failure instanceof ApiError
			? failure.message
			: 'Something went wrong. Please try again.';
	}

	async function run(work: () => Promise<TeamPage | null>) {
		busy = true;
		error = null;
		try {
			const page = await work();
			if (page) team = page;
			else await invalidateAll();
			open = null;
			action = null;
		} catch (failure) {
			error = message(failure);
		} finally {
			busy = false;
		}
	}

	function otherRole(kind: RoleKind): RoleKind {
		return kind === 'plays_games' ? 'only_trains' : 'plays_games';
	}

	/** The role that covers today, or else the coming one. */
	function currentRole(member: Member, today: string) {
		return (
			member.roles.find((r) => r.from <= today && (!r.until || today <= r.until)) ??
			member.roles.find((r) => r.from > today) ??
			null
		);
	}

	function toggle(member: Member) {
		open = open === member.id ? null : member.id;
		action = null;
		error = null;
	}

	function startAdding() {
		adding = true;
		choice = team?.candidates[0] ? String(team.candidates[0].id) : 'new';
		firstName = '';
		lastName = '';
		role = 'plays_games';
		from = team?.proposed_from ?? '';
		error = null;
		warning = null;
	}

	function save(event: SubmitEvent) {
		event.preventDefault();
		warning = null;
		return run(async () => {
			if (choice === 'new') {
				const saved = await addPlayer(firstName, lastName.trim() ? lastName : null, { role, from });
				if (saved.same_name) warning = 'There is already a player with this name.';
				adding = false;
				return null;
			}
			const page = await addRole(Number(choice), { role, from, until: null });
			adding = false;
			return page;
		});
	}

	function startChange(member: Member) {
		action = 'change';
		actionRole = otherRole(member.role);
		actionDate = team?.today ?? '';
		error = null;
	}

	function startLeave(member: Member) {
		const today = team?.today ?? '';
		action = 'leave';
		actionDate = later(today, currentRole(member, today)?.from ?? today);
		error = null;
	}

	function startReturn(member: Member) {
		const today = team?.today ?? '';
		const until = member.roles.at(-1)?.until;
		action = 'return';
		actionRole = member.role;
		actionDate = until ? later(dayAfter(until), today) : today;
		error = null;
	}

	function act(event: SubmitEvent, member: Member) {
		event.preventDefault();
		if (action === 'change') return run(() => changeRoleFrom(member.id, actionRole, actionDate));
		if (action === 'leave') return run(() => leaveTeam(member.id, actionDate));
		if (action === 'return')
			return run(() => addRole(member.id, { role: actionRole, from: actionDate, until: null }));
	}

	function rolesLine(member: Member): string {
		return member.roles
			.map(
				(r) =>
					`${roleLabels[r.role]} · ${r.until ? `${formatDay(r.from)} – ${formatDay(r.until)}` : `from ${formatDay(r.from)}`}`
			)
			.join(', ');
	}

	const field =
		'h-11 w-full min-w-0 rounded-md border border-input bg-background px-3 text-base shadow-xs focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none';
</script>

<svelte:head>
	<title>Team · Team</title>
</svelte:head>

{#snippet roleChoice(name: string, value: RoleKind, set: (kind: RoleKind) => void)}
	<fieldset class="space-y-2">
		<legend class="text-sm">Role</legend>
		<div class="flex gap-2">
			{#each Object.entries(roleLabels) as [kind, label] (kind)}
				<label
					class="flex min-h-11 flex-1 items-center gap-2 rounded-md border px-3 has-checked:border-primary has-checked:bg-primary/10"
				>
					<input
						type="radio"
						{name}
						value={kind}
						checked={value === kind}
						onchange={() => set(kind as RoleKind)}
					/>
					<span class="text-sm">{label}</span>
				</label>
			{/each}
		</div>
	</fieldset>
{/snippet}

{#snippet actionForm(member: Member, page: TeamPage)}
	<form class="space-y-3 pt-3" onsubmit={(event) => act(event, member)} data-testid="member-form">
		{#if action !== 'leave'}
			{@render roleChoice(`role-${member.id}`, actionRole, (kind) => (actionRole = kind))}
		{/if}
		<label class="block space-y-1">
			<span class="text-sm">
				{action === 'leave'
					? 'Last day in the team'
					: action === 'change'
						? 'From'
						: 'In the team from'}
			</span>
			<input
				class={field}
				type="date"
				bind:value={actionDate}
				min={page.season_first_day}
				max={page.season_last_day}
				required
			/>
		</label>
		{#if error}
			<p class="text-sm" role="alert">{error}</p>
		{/if}
		<div class="flex gap-2">
			<Button type="submit" disabled={busy}>
				{action === 'leave'
					? 'Remove from the team'
					: action === 'change'
						? 'Change role'
						: 'Add to the team'}
			</Button>
			<Button type="button" variant="ghost" onclick={() => (action = null)}>Cancel</Button>
		</div>
	</form>
{/snippet}

{#snippet row(member: Member, page: TeamPage, left: boolean)}
	<li class="rounded-xl border bg-card shadow-xs" data-testid={left ? 'left-player' : 'member'}>
		<button
			type="button"
			class="flex min-h-12 w-full items-center justify-between gap-3 px-4 py-3 text-left"
			aria-expanded={open === member.id}
			onclick={() => toggle(member)}
		>
			<span class="min-w-0 truncate font-medium">{fullName(member)}</span>
			<Badge variant={left ? 'outline' : 'secondary'}>{roleLabels[member.role]}</Badge>
		</button>
		{#if open === member.id}
			<div class="border-t px-4 pt-3 pb-4">
				<p class="text-sm text-muted-foreground tabular-nums" data-testid="member-roles">
					{rolesLine(member)}
				</p>
				{#if action}
					{@render actionForm(member, page)}
				{:else}
					<div class="flex flex-wrap gap-2 pt-3">
						{#if left}
							<Button variant="outline" onclick={() => startReturn(member)}>Add to the team</Button>
						{:else}
							<Button variant="outline" onclick={() => startChange(member)}>Change role</Button>
							<Button variant="outline" onclick={() => startLeave(member)}>
								Remove from the team
							</Button>
						{/if}
					</div>
				{/if}
			</div>
		{/if}
	</li>
{/snippet}

{#if !team}
	<section class="py-16 text-center">
		<p class="text-muted-foreground">Log in to see the team.</p>
		<Button href="/auth/login" class="mt-4" data-sveltekit-reload>Log in</Button>
	</section>
{:else}
	{@const page = team}
	<section class="space-y-6">
		<div class="flex items-start justify-between gap-3">
			<div class="min-w-0">
				{#if page.team_name}
					<h1 class="text-2xl font-semibold tracking-tight" data-testid="team-heading">
						{page.team_name} · {page.season}
					</h1>
				{:else}
					<h1 class="text-2xl font-semibold tracking-tight" data-testid="team-heading">
						Team not chosen yet
					</h1>
					<p class="mt-1 text-sm text-muted-foreground">
						Season {page.season} ·
						<a href={resolve('/choose')} class="font-medium underline underline-offset-4"
							>Choose the team</a
						>
					</p>
				{/if}
			</div>
			<Button href={resolve('/')} variant="ghost">Back</Button>
		</div>

		{#if warning}
			<p class="text-sm" role="status" data-testid="same-name">{warning}</p>
		{/if}

		{#if adding}
			<form
				class="space-y-4 rounded-xl border bg-card p-4 shadow-xs"
				onsubmit={save}
				data-testid="add-player"
			>
				<h2 class="font-medium">Add player</h2>
				<label class="block space-y-1">
					<span class="text-sm">Player</span>
					<select class={field} bind:value={choice}>
						{#each page.candidates as candidate (candidate.id)}
							<option value={String(candidate.id)}>{fullName(candidate)}</option>
						{/each}
						<option value="new">New player</option>
					</select>
				</label>
				{#if choice === 'new'}
					<label class="block space-y-1">
						<span class="text-sm">First name</span>
						<input class={field} bind:value={firstName} autocomplete="off" required />
					</label>
					<label class="block space-y-1">
						<span class="text-sm">Last name (optional)</span>
						<input class={field} bind:value={lastName} autocomplete="off" />
					</label>
				{/if}
				{@render roleChoice('role', role, (kind) => (role = kind))}
				<label class="block space-y-1">
					<span class="text-sm">In the team from</span>
					<input
						class={field}
						type="date"
						bind:value={from}
						min={page.season_first_day}
						max={page.season_last_day}
						required
					/>
				</label>
				{#if error}
					<p class="text-sm" role="alert">{error}</p>
				{/if}
				<div class="flex gap-2">
					<Button type="submit" disabled={busy}>Save</Button>
					<Button type="button" variant="ghost" onclick={() => (adding = false)}>Cancel</Button>
				</div>
			</form>
		{:else}
			<Button onclick={startAdding}>Add player</Button>
		{/if}

		{#if page.last_season.length > 0}
			<section
				aria-labelledby="last-season-heading"
				class="space-y-3 rounded-xl border bg-card p-4 shadow-xs"
				data-testid="last-season"
			>
				<div class="flex items-center justify-between gap-3">
					<h2 id="last-season-heading" class="font-medium">Players of last season</h2>
					<Button
						size="sm"
						disabled={busy}
						onclick={() => run(() => carryPlayers(page.last_season.map((member) => member.id)))}
					>
						Add all
					</Button>
				</div>
				<ul class="space-y-1">
					{#each page.last_season as member (member.id)}
						<li class="text-sm">
							{fullName(member)}
							<span class="text-muted-foreground">· {roleLabels[member.role]}</span>
						</li>
					{/each}
				</ul>
			</section>
		{/if}

		{#if error && !adding && open === null}
			<p class="text-sm" role="alert">{error}</p>
		{/if}

		{#if page.members.length === 0}
			<p class="text-sm text-muted-foreground" data-testid="no-players">
				No players in the team yet.
			</p>
		{:else}
			<ul class="space-y-2" aria-label="Players in the team">
				{#each page.members as member (member.id)}
					{@render row(member, page, false)}
				{/each}
			</ul>
		{/if}

		{#if page.left.length > 0}
			<section aria-labelledby="left-heading" class="space-y-2">
				<h2 id="left-heading" class="text-lg font-semibold tracking-tight">Left the team</h2>
				<ul class="space-y-2" data-testid="left">
					{#each page.left as member (member.id)}
						{@render row(member, page, true)}
					{/each}
				</ul>
			</section>
		{/if}
	</section>
{/if}
