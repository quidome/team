<!--
	The team of the current season (attendance at training, story A): the players in the team with
	their role, "Change role" and "Remove from the team", "Left the team" with "Add to the team",
	"Add player", and in a new season the players of last season with "Add all". "Remove from the team"
	asks first when it also removes a role, and "Last day today" shows on the row. After a removal,
	"Undo" stays until the coordinator leaves the page.
-->
<script lang="ts">
	import { invalidateAll } from '$app/navigation';
	import { resolve } from '$app/paths';
	import {
		ApiError,
		addPlayer,
		addRole,
		carryPlayers,
		changeRole,
		leaveTeam,
		undoRemoval,
		type Member,
		type RoleKind,
		type RoleView,
		type TeamPage,
		type UndoRemoval
	} from '#lib/api/client.js';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import { dayAfter, fullName, roleLabels } from '#lib/players.js';
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

	/** "Add all" is open, with its "In the team from". */
	let addingAll = $state(false);
	let allFrom = $state('');

	/** The open player and the form that is open for it. */
	let open = $state<number | null>(null);
	let action = $state<'change' | 'leave' | 'return' | null>(null);
	let actionRole = $state<RoleKind>('plays_games');
	let actionDate = $state('');

	/** "Undo" for each player removed while the page is open: its name and what to send back. */
	let undos = $state<Record<number, { name: string; undo: UndoRemoval }>>({});

	let busy = $state(false);
	let error = $state<string | null>(null);
	let warning = $state<string | null>(null);

	function message(failure: unknown): string {
		return failure instanceof ApiError
			? failure.message
			: 'Something went wrong. Please try again.';
	}

	/** Runs an action. Its answer replaces the page; `null` loads it again; `undefined` means the
	 * coordinator stopped it (the form stays open). */
	async function run(work: () => Promise<TeamPage | null | undefined>) {
		busy = true;
		error = null;
		try {
			const page = await work();
			if (page === undefined) return;
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

	function startAddingAll() {
		addingAll = true;
		allFrom = team?.proposed_from ?? '';
		error = null;
	}

	function saveAll(event: SubmitEvent) {
		event.preventDefault();
		const ids = team?.last_season.map((member) => member.id) ?? [];
		return run(async () => {
			const page = await carryPlayers(ids, allFrom);
			ids.forEach(forget);
			addingAll = false;
			return page;
		});
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
			forget(Number(choice));
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

	function startLeave() {
		action = 'leave';
		actionDate = team?.today ?? '';
		error = null;
	}

	function startReturn(member: Member) {
		action = 'return';
		actionRole = member.role;
		actionDate = team?.today ?? '';
		error = null;
	}

	/** "Remove from the team"; when it also removes roles, only after a confirmation that names
	 * them (the server says which). Keeps what "Undo" sends back. */
	async function leave(member: Member, from: string): Promise<TeamPage | undefined> {
		let answer;
		try {
			answer = await leaveTeam(member.id, from, []);
		} catch (failure) {
			if (!(failure instanceof ApiError) || failure.code !== 'confirm_removal') throw failure;
			const roles = (failure.body as { roles?: RoleView[] }).roles ?? [];
			const named = roles.map((r) => `"${roleText(r)}"`).join(' and ');
			if (!confirm(`This also removes ${named}.`)) return undefined;
			answer = await leaveTeam(
				member.id,
				from,
				roles.map((r) => r.id)
			);
		}
		undos = { ...undos, [member.id]: { name: fullName(member), undo: answer.undo } };
		return answer.team;
	}

	/** Another change of the player ends its "Undo". */
	function forget(id: number) {
		undos = Object.fromEntries(Object.entries(undos).filter(([key]) => Number(key) !== id));
	}

	function undo(id: number) {
		const entry = undos[id];
		if (!entry) return;
		open = null;
		return run(async () => {
			try {
				return await undoRemoval(id, entry.undo);
			} finally {
				forget(id);
			}
		});
	}

	/** The players removed while the page is open who are in no group, so have no row. */
	let removed = $derived.by(() => {
		const page = team;
		if (!page) return [];
		const shown = new Set([...page.members, ...page.left].map((m) => m.id));
		return Object.entries(undos)
			.map(([id, entry]) => ({ id: Number(id), name: entry.name }))
			.filter(({ id }) => !shown.has(id));
	});

	function act(event: SubmitEvent, member: Member) {
		event.preventDefault();
		if (action === 'change')
			return run(async () => {
				const page = await changeRole(member.id, actionRole, actionDate);
				forget(member.id);
				return page;
			});
		if (action === 'leave') return run(() => leave(member, actionDate));
		if (action === 'return')
			return run(async () => {
				const page = await addRole(member.id, { role: actionRole, from: actionDate, until: null });
				forget(member.id);
				return page;
			});
	}

	/** "Only trains · Tue 1 Sept – Sun 11 Oct", or "Plays games · from Sun 1 Nov". */
	function roleText(r: RoleView): string {
		const dates = r.until
			? `${formatDay(r.from)} – ${formatDay(r.until)}`
			: `from ${formatDay(r.from)}`;
		return `${roleLabels[r.role]} · ${dates}`;
	}

	function rolesLine(member: Member): string {
		return member.roles.map(roleText).join(', ');
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

{#snippet teamFrom(value: string, set: (date: string) => void, page: TeamPage)}
	<label class="block space-y-1">
		<span class="text-sm">In the team from</span>
		<span class="flex gap-2">
			<input
				class={field}
				type="date"
				{value}
				oninput={(event) => set(event.currentTarget.value)}
				min={page.season_first_day}
				max={page.season_last_day}
				required
			/>
			<Button type="button" variant="outline" onclick={() => set(page.today)}>Today</Button>
		</span>
	</label>
{/snippet}

{#snippet actionForm(member: Member, page: TeamPage)}
	<form class="space-y-3 pt-3" onsubmit={(event) => act(event, member)} data-testid="member-form">
		{#if action !== 'leave'}
			{@render roleChoice(`role-${member.id}`, actionRole, (kind) => (actionRole = kind))}
		{/if}
		<label class="block space-y-1">
			<span class="text-sm">
				{action === 'leave'
					? 'Not in the team from'
					: action === 'change'
						? 'From'
						: 'In the team from'}
			</span>
			<span class="flex gap-2">
				<input
					class={field}
					type="date"
					bind:value={actionDate}
					min={page.season_first_day}
					max={page.season_last_day}
					required
				/>
				{#if action === 'leave'}
					<Button
						type="button"
						variant="outline"
						onclick={() => (actionDate = dayAfter(page.today))}
					>
						Tomorrow
					</Button>
				{/if}
			</span>
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
			<span class="flex shrink-0 items-center gap-2">
				{#if member.last_day_today}
					<span class="text-xs text-muted-foreground" data-testid="last-day">Last day today</span>
				{/if}
				<Badge variant={left ? 'outline' : 'secondary'}>{roleLabels[member.role]}</Badge>
			</span>
		</button>
		{#if undos[member.id]}
			<div class="px-4 pb-3">
				<Button size="sm" variant="outline" disabled={busy} onclick={() => undo(member.id)}>
					Undo
				</Button>
			</div>
		{/if}
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
							<Button variant="outline" onclick={startLeave}>Remove from the team</Button>
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
				{@render teamFrom(from, (date) => (from = date), page)}
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
					{#if !addingAll}
						<Button size="sm" disabled={busy} onclick={startAddingAll}>Add all</Button>
					{/if}
				</div>
				{#if addingAll}
					<form class="space-y-3" onsubmit={saveAll} data-testid="add-all">
						{@render teamFrom(allFrom, (date) => (allFrom = date), page)}
						{#if error}
							<p class="text-sm" role="alert">{error}</p>
						{/if}
						<div class="flex gap-2">
							<Button type="submit" disabled={busy}>Add all</Button>
							<Button type="button" variant="ghost" onclick={() => (addingAll = false)}>
								Cancel
							</Button>
						</div>
					</form>
				{/if}
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

		{#if error && !adding && !addingAll && open === null}
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

		{#if removed.length > 0}
			<ul class="space-y-2" aria-label="Removed just now">
				{#each removed as player (player.id)}
					<li
						class="flex min-h-12 items-center justify-between gap-3 rounded-xl border bg-card px-4 py-2 text-sm shadow-xs"
						data-testid="removed"
					>
						<span class="min-w-0">
							<span class="font-medium">{player.name}</span>
							<span class="text-muted-foreground">· not in the team this season</span>
						</span>
						<Button size="sm" variant="outline" disabled={busy} onclick={() => undo(player.id)}>
							Undo
						</Button>
					</li>
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
