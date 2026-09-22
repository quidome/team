<script>
  import { invalidateAll } from '$app/navigation';
  import AttendancePanel from '$lib/components/AttendancePanel.svelte';

  /** @typedef {import('./$types').PageData} PageData */

  /** @type {PageData} */
  export let data;

  let playerForm = {
    associationId: '',
    birthDate: '',
    firstName: '',
    lastName: '',
  };
  let membershipForm = {
    jerseyNumber: '',
    participationType: 'trains_and_plays',
    playerId: data.players[0]?.id ?? '',
    relationship: 'primary',
    status: 'active',
  };
  let playerEditForm = {
    associationId: '',
    birthDate: '',
    firstName: '',
    lastName: '',
  };
  let playerSaving = false;
  let membershipSaving = false;
  let playerEditSaving = false;
  let showPlayerModal = false;
  let showMembershipModal = false;
  let selectedPlayerId = '';
  let playerMessage = '';
  let playerError = '';
  let membershipError = '';
  let playerEditError = '';

  $: currentTeamPlayers = data.players.filter(
    (player) => player.membership?.teamName === data.teamName,
  );
  $: selectedPlayer = data.players.find((player) => player.id === selectedPlayerId);

  /** @param {string} path @param {unknown} payload @param {string} [method] */
  const postJson = async (path, payload, method = 'POST') => {
    const response = await fetch(path, {
      body: JSON.stringify(payload),
      headers: { 'content-type': 'application/json' },
      method,
    });
    /** @type {any} */
    const body = await response.json().catch(() => ({}));

    if (!response.ok) {
      throw new Error(body.error ?? 'The change could not be stored.');
    }

    return body;
  };

  /** @param {import('$lib/application/team/read-team').TeamPlayer} player */
  const displayName = (player) =>
    player.lastName ? `${player.firstName} ${player.lastName}` : player.firstName;

  /** @param {import('$lib/application/team/read-team').TeamPlayer} player */
  const openMembershipEditor = (player) => {
    selectedPlayerId = player.id;
    membershipForm = {
      jerseyNumber: player.membership?.jerseyNumber?.toString() ?? '',
      participationType: player.membership?.participationType ?? 'trains_and_plays',
      playerId: player.id,
      relationship: player.membership?.relationship ?? 'primary',
      status: player.membership?.status ?? 'active',
    };
    playerEditForm = {
      associationId: player.associationId ?? '',
      birthDate: player.birthDate ?? '',
      firstName: player.firstName,
      lastName: player.lastName ?? '',
    };
    membershipError = '';
    playerEditError = '';
    showMembershipModal = true;
  };

  const deletePlayer = async () => {
    if (!selectedPlayer || !window.confirm(`Delete ${displayName(selectedPlayer)}?`)) return;

    try {
      const response = await fetch(`/api/players?id=${encodeURIComponent(selectedPlayer.id)}`, {
        method: 'DELETE',
      });

      if (!response.ok) {
        const body = await response.json().catch(() => ({}));
        throw new Error(body.error ?? 'Player could not be deleted.');
      }

      showMembershipModal = false;
      await invalidateAll();
    } catch (error) {
      membershipError = error instanceof Error ? error.message : 'Player could not be deleted.';
    }
  };

  const registerPlayer = async () => {
    playerSaving = true;
    playerMessage = '';
    playerError = '';

    try {
      const createdPlayer = await postJson('/api/players', {
        firstName: playerForm.firstName.trim(),
        ...(playerForm.lastName.trim() ? { lastName: playerForm.lastName.trim() } : {}),
        ...(playerForm.birthDate ? { birthDate: playerForm.birthDate } : {}),
        ...(playerForm.associationId.trim()
          ? { associationId: playerForm.associationId.trim() }
          : {}),
      });
      await postJson('/api/memberships', {
        participationType: 'trains_and_plays',
        playerId: createdPlayer.id,
        relationship: 'primary',
        seasonStartingYear: data.season.startingYear,
        status: 'active',
        teamName: data.teamName,
      });
      playerForm = { associationId: '', birthDate: '', firstName: '', lastName: '' };
      playerMessage = `Player added to ${data.season.startingYear}–${data.season.endingYear} · ${data.teamName}.`;
      showPlayerModal = false;
      await invalidateAll();
    } catch (error) {
      playerError = error instanceof Error ? error.message : 'Player could not be registered.';
    } finally {
      playerSaving = false;
    }
  };

  const saveMembership = async () => {
    membershipSaving = true;
    membershipError = '';

    try {
      await postJson('/api/memberships', {
        ...(membershipForm.jerseyNumber
          ? { jerseyNumber: Number(membershipForm.jerseyNumber) }
          : {}),
        participationType: membershipForm.participationType,
        playerId: membershipForm.playerId,
        relationship: membershipForm.relationship,
        seasonStartingYear: data.season.startingYear,
        status: membershipForm.status,
        teamName: data.teamName,
      });
      showMembershipModal = false;
      await invalidateAll();
    } catch (error) {
      membershipError = error instanceof Error ? error.message : 'Membership could not be saved.';
    } finally {
      membershipSaving = false;
    }
  };

  const savePlayerDetails = async () => {
    if (!selectedPlayer) return;

    playerEditSaving = true;
    playerEditError = '';

    try {
      await postJson(
        '/api/players',
        {
          id: selectedPlayer.id,
          firstName: playerEditForm.firstName.trim(),
          ...(playerEditForm.lastName.trim() ? { lastName: playerEditForm.lastName.trim() } : {}),
          ...(playerEditForm.birthDate ? { birthDate: playerEditForm.birthDate } : {}),
          ...(playerEditForm.associationId.trim()
            ? { associationId: playerEditForm.associationId.trim() }
            : {}),
        },
        'PUT',
      );
      await invalidateAll();
    } catch (error) {
      playerEditError =
        error instanceof Error ? error.message : 'Player details could not be saved.';
    } finally {
      playerEditSaving = false;
    }
  };
</script>

<svelte:head>
  <title>Roster · Team</title>
  <meta name="description" content="Players, memberships, and attendance for the team" />
</svelte:head>

<section class="intro">
  <p class="eyebrow">{data.teamName} · {data.season.startingYear}–{data.season.endingYear}</p>
  <h1>Roster</h1>
  <p class="lede">Keep the current roster up to date and record attendance in one place.</p>
</section>

<section class="panel" aria-labelledby="players-heading">
  <div class="panel-heading">
    <div>
      <p class="eyebrow">{currentTeamPlayers.length} team memberships</p>
      <h2 id="players-heading">Players</h2>
    </div>
    <div class="panel-actions">
      <button type="button" on:click={() => (showPlayerModal = true)}>Add player</button>
      <a class="text-link" href="#attendance">Record attendance <span aria-hidden="true">↓</span></a
      >
    </div>
  </div>

  {#if data.players.length === 0}
    <div class="empty-state">
      <h3>No players registered yet</h3>
      <p>Add a player below and they will be added to this team-season automatically.</p>
    </div>
  {:else}
    <div class="player-list">
      {#each data.players as player (player.id)}
        <button class="player-card" type="button" on:click={() => openMembershipEditor(player)}>
          <div>
            <p class="player-name">{player.firstName}</p>
            <p class="player-meta">
              {#if player.normalAgeGroup}{player.normalAgeGroup}{:else}Age group unknown{/if}
              {#if player.birthDate}· Born {player.birthDate}{/if}
              {#if player.associationId}· Association ID {player.associationId}{/if}
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
                {player.membership.relationship === 'primary'
                  ? 'Primary'
                  : 'Secondary'}{#if player.membership.jerseyNumber}&nbsp;·&nbsp;#{player.membership
                    .jerseyNumber}{/if}
              </span>
            </div>
          {:else}
            <span class="unassigned">No {data.teamName} membership</span>
          {/if}
        </button>
      {/each}
    </div>
  {/if}
  {#if playerMessage}<p class="form-message" role="status">{playerMessage}</p>{/if}
</section>

{#if showPlayerModal}
  <div class="modal-backdrop" role="presentation">
    <div
      class="panel modal"
      role="dialog"
      aria-modal="true"
      aria-labelledby="register-player-heading"
    >
      <div class="modal-header">
        <div>
          <p class="eyebrow">Player details</p>
          <h2 id="register-player-heading">Add a player</h2>
        </div>
        <button class="close-button" type="button" on:click={() => (showPlayerModal = false)}>
          Close
        </button>
      </div>
      <form class="form-grid" on:submit|preventDefault={registerPlayer}>
        <label>
          First name
          <input bind:value={playerForm.firstName} required autocomplete="given-name" />
        </label>
        <label>
          Last name <span class="optional">optional</span>
          <input bind:value={playerForm.lastName} autocomplete="family-name" />
        </label>
        <label>
          Birthday <span class="optional">optional</span>
          <input bind:value={playerForm.birthDate} type="date" />
        </label>
        <label>
          Association ID <span class="optional">optional</span>
          <input bind:value={playerForm.associationId} autocomplete="off" />
        </label>
        <button disabled={playerSaving} type="submit">
          {playerSaving ? 'Saving…' : 'Add player'}
        </button>
      </form>
      {#if playerError}<p class="form-error" role="alert">{playerError}</p>{/if}
    </div>
  </div>
{/if}

{#if showMembershipModal && selectedPlayer}
  <div class="modal-backdrop" role="presentation">
    <div class="panel modal" role="dialog" aria-modal="true" aria-labelledby="membership-heading">
      <div class="modal-header">
        <div>
          <p class="eyebrow">
            {data.teamName} · {data.season.startingYear}–{data.season.endingYear}
          </p>
          <h2 id="membership-heading">{displayName(selectedPlayer)}</h2>
        </div>
        <button class="close-button" type="button" on:click={() => (showMembershipModal = false)}>
          Close
        </button>
      </div>
      <p class="modal-context">Player details.</p>
      <form class="form-grid" on:submit|preventDefault={savePlayerDetails}>
        <label>
          First name
          <input bind:value={playerEditForm.firstName} required autocomplete="given-name" />
        </label>
        <label>
          Last name <span class="optional">optional</span>
          <input bind:value={playerEditForm.lastName} autocomplete="family-name" />
        </label>
        <label>
          Birthday <span class="optional">optional</span>
          <input bind:value={playerEditForm.birthDate} type="date" />
        </label>
        <label>
          Association ID <span class="optional">optional</span>
          <input bind:value={playerEditForm.associationId} autocomplete="off" />
        </label>
        <button disabled={playerEditSaving} type="submit">
          {playerEditSaving ? 'Saving…' : 'Save details'}
        </button>
      </form>
      {#if playerEditError}<p class="form-error" role="alert">{playerEditError}</p>{/if}
      <p class="modal-context">
        {selectedPlayer.membership
          ? 'Membership for the active team-season context.'
          : `No ${data.teamName} membership yet — saving will create one with the values below.`}
      </p>
      <form class="form-grid" on:submit|preventDefault={saveMembership}>
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
        <div class="modal-actions">
          <button disabled={membershipSaving} type="submit">
            {membershipSaving ? 'Saving…' : 'Save membership'}
          </button>
          <button class="delete-button" type="button" on:click={deletePlayer}>Delete player</button>
        </div>
      </form>
      {#if membershipError}<p class="form-error" role="alert">{membershipError}</p>{/if}
    </div>
  </div>
{/if}

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
    color: var(--ink);
    display: flex;
    gap: 1rem;
    justify-content: space-between;
    padding: 1rem;
    text-align: left;
    width: 100%;
  }

  .player-card:hover {
    background: #fff4ef;
    border-color: #e2b8aa;
  }

  .panel-actions {
    align-items: center;
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
    justify-content: end;
  }

  .modal-backdrop {
    align-items: start;
    background: rgba(36, 34, 31, 0.42);
    display: flex;
    inset: 0;
    justify-content: center;
    overflow: auto;
    padding: 2rem 1rem;
    position: fixed;
    z-index: 10;
  }

  .modal {
    margin: auto;
    max-width: 32rem;
    width: 100%;
  }

  .modal-header {
    align-items: start;
    display: flex;
    gap: 1rem;
    justify-content: space-between;
    margin-bottom: 1.25rem;
  }

  .modal-header h2 {
    font-size: clamp(1.5rem, 4vw, 2rem);
    letter-spacing: -0.05em;
    margin: 0;
  }

  .close-button {
    background: transparent;
    border: 1px solid var(--line);
    color: var(--ink);
    flex: 0 0 auto;
  }

  .modal-context {
    color: var(--muted);
    font-size: 0.9rem;
    margin: -0.5rem 0 1.25rem;
  }

  .modal-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.65rem;
  }

  .delete-button {
    background: transparent;
    border: 1px solid #e2b8aa;
    color: var(--accent-dark);
  }

  .delete-button:hover {
    background: #fff4ef;
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
