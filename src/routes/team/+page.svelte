<script>
  import { invalidateAll } from '$app/navigation';
  import AttendancePanel from '$lib/components/AttendancePanel.svelte';

  /** @typedef {import('./$types').PageData} PageData */

  /** @type {PageData} */
  export let data;

  let playerForm = {
    associationId: '',
    birthDate: '',
    name: '',
  };
  let membershipForm = {
    jerseyNumber: '',
    participationType: 'trains_and_plays',
    playerAssociationId: data.players[0]?.associationId ?? '',
    relationship: 'primary',
    status: 'active',
  };
  let playerSaving = false;
  let membershipSaving = false;
  let playerMessage = '';
  let membershipMessage = '';
  let playerError = '';
  let membershipError = '';

  $: currentTeamPlayers = data.players.filter(
    (player) => player.membership?.teamName === data.teamName,
  );

  /** @param {string} path @param {unknown} payload */
  const postJson = async (path, payload) => {
    const response = await fetch(path, {
      body: JSON.stringify(payload),
      headers: { 'content-type': 'application/json' },
      method: 'POST',
    });
    /** @type {{ error?: string }} */
    const body = await response.json().catch(() => ({}));

    if (!response.ok) {
      throw new Error(body.error ?? 'The change could not be stored.');
    }

    return body;
  };

  const registerPlayer = async () => {
    playerSaving = true;
    playerMessage = '';
    playerError = '';

    try {
      await postJson('/api/players', playerForm);
      membershipForm = { ...membershipForm, playerAssociationId: playerForm.associationId };
      playerForm = { associationId: '', birthDate: '', name: '' };
      playerMessage = 'Player registered.';
      await invalidateAll();
    } catch (error) {
      playerError = error instanceof Error ? error.message : 'Player could not be registered.';
    } finally {
      playerSaving = false;
    }
  };

  const saveMembership = async () => {
    membershipSaving = true;
    membershipMessage = '';
    membershipError = '';

    try {
      await postJson('/api/memberships', {
        ...(membershipForm.jerseyNumber
          ? { jerseyNumber: Number(membershipForm.jerseyNumber) }
          : {}),
        participationType: membershipForm.participationType,
        playerAssociationId: membershipForm.playerAssociationId,
        relationship: membershipForm.relationship,
        seasonStartingYear: data.season.startingYear,
        status: membershipForm.status,
        teamName: data.teamName,
      });
      membershipMessage = `Membership saved for the ${data.season.startingYear}–${data.season.endingYear} season.`;
      await invalidateAll();
    } catch (error) {
      membershipError = error instanceof Error ? error.message : 'Membership could not be saved.';
    } finally {
      membershipSaving = false;
    }
  };
</script>

<svelte:head>
  <title>Team · Team</title>
  <meta name="description" content="Players, memberships, and attendance for the team" />
</svelte:head>

<section class="intro">
  <p class="eyebrow">{data.teamName} · {data.season.startingYear}–{data.season.endingYear}</p>
  <h1>Team</h1>
  <p class="lede">Keep the current roster up to date and record attendance in one place.</p>
</section>

<section class="panel" aria-labelledby="players-heading">
  <div class="panel-heading">
    <div>
      <p class="eyebrow">{currentTeamPlayers.length} team memberships</p>
      <h2 id="players-heading">Players</h2>
    </div>
    <a class="text-link" href="#attendance">Record attendance <span aria-hidden="true">↓</span></a>
  </div>

  {#if data.players.length === 0}
    <div class="empty-state">
      <h3>No players registered yet</h3>
      <p>Register the players below, then add their current-season membership.</p>
    </div>
  {:else}
    <div class="player-list">
      {#each data.players as player (player.associationId)}
        <article class="player-card">
          <div>
            <p class="player-name">{player.name}</p>
            <p class="player-meta">
              {player.normalAgeGroup} · Born {player.birthDate} · Association ID {player.associationId}
            </p>
          </div>
          {#if player.membership}
            <div class="membership-details">
              <strong>{player.membership.teamName}</strong>
              <span>
                {player.membership.status === 'active' ? 'Active' : 'Inactive'} ·
                {player.membership.participationType === 'trains_only'
                  ? 'Trains only'
                  : 'Trains and plays'}
              </span>
              <span>
                {player.membership.relationship === 'primary' ? 'Primary' : 'Secondary'}
                {#if player.membership.jerseyNumber}
                  · #{player.membership.jerseyNumber}{/if}
              </span>
            </div>
          {:else}
            <span class="unassigned">No {data.teamName} membership</span>
          {/if}
        </article>
      {/each}
    </div>
  {/if}
</section>

<div class="two-column">
  <section class="panel" aria-labelledby="register-player-heading">
    <div class="panel-heading">
      <div>
        <p class="eyebrow">Player details</p>
        <h2 id="register-player-heading">Register a player</h2>
      </div>
    </div>

    <form class="form-grid" on:submit|preventDefault={registerPlayer}>
      <label>
        Name
        <input bind:value={playerForm.name} required autocomplete="name" />
      </label>
      <label>
        Birthday
        <input bind:value={playerForm.birthDate} required type="date" />
      </label>
      <label>
        Association ID
        <input bind:value={playerForm.associationId} required autocomplete="off" />
      </label>
      <button disabled={playerSaving} type="submit">
        {playerSaving ? 'Saving…' : 'Register player'}
      </button>
    </form>
    {#if playerMessage}<p class="form-message" role="status">{playerMessage}</p>{/if}
    {#if playerError}<p class="form-error" role="alert">{playerError}</p>{/if}
  </section>

  <section class="panel" aria-labelledby="membership-heading">
    <div class="panel-heading">
      <div>
        <p class="eyebrow">{data.teamName} · {data.season.startingYear}–{data.season.endingYear}</p>
        <h2 id="membership-heading">Team membership</h2>
      </div>
    </div>

    {#if data.players.length === 0}
      <div class="empty-state compact">
        <h3>Register a player first</h3>
        <p>The new player will appear here for membership setup.</p>
      </div>
    {:else}
      <form class="form-grid" on:submit|preventDefault={saveMembership}>
        <label>
          Player
          <select bind:value={membershipForm.playerAssociationId} required>
            <option disabled value="">Choose a player</option>
            {#each data.players as player (player.associationId)}
              <option value={player.associationId}>{player.name}</option>
            {/each}
          </select>
        </label>
        <label>
          Participation
          <select bind:value={membershipForm.participationType}>
            <option value="trains_and_plays">Trains and plays</option>
            <option value="trains_only">Trains only</option>
          </select>
        </label>
        <label>
          Relationship
          <select bind:value={membershipForm.relationship}>
            <option value="primary">Primary</option>
            <option value="secondary">Secondary</option>
          </select>
        </label>
        <label>
          Status
          <select bind:value={membershipForm.status}>
            <option value="active">Active</option>
            <option value="inactive">Inactive</option>
          </select>
        </label>
        <label>
          Jersey number <span class="optional">optional</span>
          <input bind:value={membershipForm.jerseyNumber} min="1" type="number" />
        </label>
        <button disabled={membershipSaving} type="submit">
          {membershipSaving ? 'Saving…' : 'Save membership'}
        </button>
      </form>
      {#if membershipMessage}<p class="form-message" role="status">{membershipMessage}</p>{/if}
      {#if membershipError}<p class="form-error" role="alert">{membershipError}</p>{/if}
    {/if}
  </section>
</div>

<AttendancePanel
  events={data.events}
  players={data.players}
  selectedEventId={data.selectedEventId}
  teamName={data.teamName}
/>

<style>
  .player-list {
    display: grid;
    gap: 0.65rem;
  }

  .player-card {
    align-items: center;
    background: #faf7f0;
    border: 1px solid #ebe4d8;
    border-radius: 0.9rem;
    display: flex;
    gap: 1rem;
    justify-content: space-between;
    padding: 1rem;
  }

  .player-name {
    font-size: 1.05rem;
    font-weight: 800;
    margin: 0;
  }

  .player-meta,
  .membership-details,
  .unassigned {
    color: var(--muted);
    font-size: 0.84rem;
    margin: 0.3rem 0 0;
  }

  .membership-details {
    display: grid;
    gap: 0.2rem;
    margin: 0;
    text-align: right;
  }

  .membership-details strong {
    color: var(--ink);
  }

  .two-column {
    display: grid;
    gap: 1rem;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    margin-top: 1rem;
  }

  .form-grid {
    display: grid;
    gap: 0.9rem;
  }

  .form-grid label {
    color: var(--muted);
    display: grid;
    font-size: 0.78rem;
    font-weight: 800;
    gap: 0.35rem;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  input,
  select {
    background: #fffdf8;
    border: 1px solid var(--line);
    border-radius: 0.55rem;
    box-sizing: border-box;
    color: var(--ink);
    font: inherit;
    font-size: 0.95rem;
    font-weight: 500;
    letter-spacing: normal;
    min-height: 2.7rem;
    padding: 0.55rem 0.65rem;
    text-transform: none;
    width: 100%;
  }

  input:focus,
  select:focus,
  button:focus-visible {
    outline: 3px solid rgba(213, 99, 62, 0.25);
    outline-offset: 2px;
  }

  button {
    background: var(--accent);
    border: 0;
    border-radius: 0.55rem;
    color: white;
    cursor: pointer;
    font: inherit;
    font-weight: 800;
    min-height: 2.7rem;
    padding: 0.65rem 0.9rem;
  }

  button:hover:not(:disabled) {
    background: var(--accent-dark);
  }

  button:disabled {
    cursor: wait;
    opacity: 0.55;
  }

  .optional {
    font-size: 0.68rem;
    font-weight: 500;
    letter-spacing: 0;
    text-transform: none;
  }

  .form-message,
  .form-error {
    font-size: 0.88rem;
    margin: 1rem 0 0;
  }

  .form-message {
    color: #397044;
  }

  .form-error {
    color: var(--accent-dark);
  }

  @media (max-width: 48rem) {
    .two-column {
      grid-template-columns: 1fr;
    }
  }

  @media (max-width: 36rem) {
    .player-card {
      align-items: stretch;
      flex-direction: column;
    }

    .membership-details {
      text-align: left;
    }
  }
</style>
