<script>
  import { invalidateAll } from '$app/navigation';
  import { onMount } from 'svelte';

  /** @typedef {import('$lib/application/participation/participation-repository').ParticipationRecord} ParticipationRecord */
  /** @typedef {import('$lib/application/program/read-program').ProgramEvent} ProgramEvent */
  /** @typedef {import('./$types').PageData} PageData */
  /** @typedef {'illness' | 'injury' | 'other'} AbsenceReason */
  /** @typedef {Record<string, boolean>} AttendanceState */
  /** @typedef {Record<string, AbsenceReason>} AbsenceReasonState */

  /** @type {PageData} */
  export let data;

  const absenceReasons = [
    { value: 'illness', label: 'Illness' },
    { value: 'injury', label: 'Injury' },
    { value: 'other', label: 'Other' },
  ];

  /** @param {string} date */
  const formatDate = (date) =>
    new Intl.DateTimeFormat('en', {
      day: 'numeric',
      month: 'short',
      timeZone: 'UTC',
      weekday: 'short',
      year: 'numeric',
    }).format(new Date(`${date}T00:00:00.000Z`));

  /** @param {ProgramEvent} event */
  const isAttendanceEvent = (event) =>
    event.type === 'training' ||
    (event.status === 'scheduled' &&
      (event.homeTeamName === data.teamName || event.awayTeamName === data.teamName));
  const initialEvent = data.events.find(isAttendanceEvent);

  let selectedEventId = initialEvent?.id ?? '';
  /** @type {AttendanceState} */
  let attendance = {};
  /** @type {AbsenceReasonState} */
  let recordedAbsenceReasons = {};
  let attendanceLoading = false;
  let attendanceSaving = false;
  let attendanceMessage = '';
  let attendanceError = '';

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

  $: attendanceEvents = data.events.filter(isAttendanceEvent);
  $: selectedEvent = data.events.find((event) => event.id === selectedEventId);
  $: currentTeamPlayers = data.players.filter(
    (player) => player.membership?.teamName === data.teamName,
  );
  $: eligiblePlayers = selectedEvent
    ? currentTeamPlayers.filter(
        (player) =>
          player.membership?.status === 'active' &&
          (selectedEvent.type === 'training' ||
            player.membership.participationType === 'trains_and_plays'),
      )
    : [];

  /** @param {ProgramEvent} event */
  const eligiblePlayersFor = (event) =>
    currentTeamPlayers.filter(
      (player) =>
        player.membership?.status === 'active' &&
        (event.type === 'training' || player.membership.participationType === 'trains_and_plays'),
    );

  /** @param {ProgramEvent} event */
  const occurrenceIdFor = (event) =>
    event.type === 'training' ? (event.occurrenceId ?? event.id) : event.id;

  /** @param {ProgramEvent} event */
  const eventTitle = (event) =>
    event.type === 'training' ? 'Team training' : `${event.homeTeamName} vs ${event.awayTeamName}`;

  /** @param {ProgramEvent} event */
  const eventSummary = (event) =>
    event.type === 'training'
      ? `${event.locationName} · ${event.durationMinutes} minutes`
      : `${event.locationName} · Leave by ${event.suggestedDepartureTime}`;

  /** @param {ProgramEvent | undefined} event */
  const loadAttendance = async (event) => {
    attendanceMessage = '';
    attendanceError = '';

    if (!event) {
      attendance = {};
      recordedAbsenceReasons = {};
      return;
    }

    attendanceLoading = true;

    try {
      const response = await fetch(
        `/api/participation?occurrenceType=${event.type}&occurrenceId=${occurrenceIdFor(event)}`,
      );

      if (!response.ok) {
        throw new Error('Attendance could not be loaded.');
      }

      /** @type {ParticipationRecord[]} */
      const records = await response.json();
      const recordsByPlayer = new Map(
        records.map((record) => [record.playerAssociationId, record]),
      );
      /** @type {AttendanceState} */
      const nextAttendance = {};
      /** @type {AbsenceReasonState} */
      const nextAbsenceReasons = {};

      for (const player of eligiblePlayersFor(event)) {
        const record = recordsByPlayer.get(player.associationId);
        nextAttendance[player.associationId] = record?.status !== 'absent';

        if (record?.status === 'absent') {
          nextAbsenceReasons[player.associationId] = record.absenceReason ?? 'other';
        }
      }

      attendance = nextAttendance;
      recordedAbsenceReasons = nextAbsenceReasons;
    } catch (error) {
      attendanceError = error instanceof Error ? error.message : 'Attendance could not be loaded.';
    } finally {
      attendanceLoading = false;
    }
  };

  const handleEventChange = async () => {
    await loadAttendance(selectedEvent);
  };

  /** @param {boolean} present */
  const setAllAttendance = (present) => {
    attendance = Object.fromEntries(
      eligiblePlayers.map((player) => [player.associationId, present]),
    );

    if (!present) {
      recordedAbsenceReasons = Object.fromEntries(
        eligiblePlayers.map((player) => [player.associationId, 'other']),
      );
    }
  };

  /** @param {string} associationId @param {Event} event */
  const setPlayerAttendance = (associationId, event) => {
    const input = /** @type {HTMLInputElement} */ (event.currentTarget);
    attendance = { ...attendance, [associationId]: input.checked };

    if (!input.checked && !recordedAbsenceReasons[associationId]) {
      recordedAbsenceReasons = { ...recordedAbsenceReasons, [associationId]: 'other' };
    }
  };

  /** @param {string} associationId @param {Event} event */
  const setAbsenceReason = (associationId, event) => {
    const select = /** @type {HTMLSelectElement} */ (event.currentTarget);
    const reason = /** @type {AbsenceReason} */ (select.value);
    recordedAbsenceReasons = {
      ...recordedAbsenceReasons,
      [associationId]: reason,
    };
  };

  const storeAttendance = async () => {
    if (!selectedEvent) return;

    attendanceSaving = true;
    attendanceMessage = '';
    attendanceError = '';

    try {
      const response = await fetch('/api/participation', {
        body: JSON.stringify({
          absences: eligiblePlayers
            .filter((player) => attendance[player.associationId] === false)
            .map((player) => ({
              playerAssociationId: player.associationId,
              reason: recordedAbsenceReasons[player.associationId] ?? 'other',
            })),
          eligiblePlayerAssociationIds: eligiblePlayers.map((player) => player.associationId),
          occurrenceId: occurrenceIdFor(selectedEvent),
          occurrenceType: selectedEvent.type,
        }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      });

      if (!response.ok) {
        const body = await response.json().catch(() => ({}));
        throw new Error(body.error ?? 'Attendance could not be stored.');
      }

      attendanceMessage = `Attendance stored for ${eventTitle(selectedEvent)}.`;
      await loadAttendance(selectedEvent);
    } catch (error) {
      attendanceError = error instanceof Error ? error.message : 'Attendance could not be stored.';
    } finally {
      attendanceSaving = false;
    }
  };

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

  onMount(() => {
    if (selectedEvent) {
      void loadAttendance(selectedEvent);
    }
  });
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

<section class="panel attendance-panel" id="attendance" aria-labelledby="attendance-heading">
  <div class="panel-heading">
    <div>
      <p class="eyebrow">Bulk workflow · {eligiblePlayers.length} eligible players</p>
      <h2 id="attendance-heading">Attendance</h2>
    </div>
  </div>

  {#if attendanceEvents.length === 0}
    <div class="empty-state">
      <h3>No attendance events yet</h3>
      <p>Add a training occurrence or scheduled game in Program before recording participation.</p>
    </div>
  {:else}
    <label class="event-picker">
      Event
      <select bind:value={selectedEventId} on:change={handleEventChange}>
        {#each attendanceEvents as event (event.id)}
          <option value={event.id}>{formatDate(event.date)} · {eventTitle(event)}</option>
        {/each}
      </select>
    </label>

    {#if selectedEvent}
      <div class="selected-event">
        <div>
          <p class="event-kind">{selectedEvent.type === 'training' ? 'Training' : 'Game'}</p>
          <h3>{eventTitle(selectedEvent)}</h3>
          <p>{selectedEvent.startTime} · {eventSummary(selectedEvent)}</p>
        </div>
        <div class="attendance-actions">
          <button
            class="secondary-button"
            disabled={attendanceLoading}
            type="button"
            on:click={() => setAllAttendance(true)}
          >
            Select all present
          </button>
          <button
            class="secondary-button"
            disabled={attendanceLoading}
            type="button"
            on:click={() => setAllAttendance(false)}
          >
            Clear selection
          </button>
        </div>
      </div>

      {#if attendanceLoading}
        <p class="loading-message" role="status">Loading recorded attendance…</p>
      {:else if eligiblePlayers.length === 0}
        <div class="empty-state compact">
          <h3>No eligible players</h3>
          <p>
            Active {data.teamName} memberships appear here. Training-only players are excluded from games.
          </p>
        </div>
      {:else}
        <div class="attendance-list">
          {#each eligiblePlayers as player (player.associationId)}
            <div class:absent={attendance[player.associationId] === false} class="attendance-row">
              <label class="attendance-player">
                <input
                  checked={attendance[player.associationId] !== false}
                  type="checkbox"
                  on:change={(event) => setPlayerAttendance(player.associationId, event)}
                />
                <span>
                  <strong>{player.name}</strong>
                  <small
                    >{player.normalAgeGroup}{#if player.membership?.jerseyNumber}
                      · #{player.membership.jerseyNumber}{/if}</small
                  >
                </span>
              </label>
              {#if attendance[player.associationId] === false}
                <label class="absence-reason">
                  <span>Absence reason</span>
                  <select
                    value={recordedAbsenceReasons[player.associationId] ?? 'other'}
                    on:change={(event) => setAbsenceReason(player.associationId, event)}
                  >
                    {#each absenceReasons as reason (reason.value)}
                      <option value={reason.value}>{reason.label}</option>
                    {/each}
                  </select>
                </label>
              {:else}
                <span class="present-label">Present</span>
              {/if}
            </div>
          {/each}
        </div>
        <button
          class="primary-button store-button"
          disabled={attendanceLoading || attendanceSaving}
          type="button"
          on:click={storeAttendance}
        >
          {attendanceSaving ? 'Storing…' : 'Store attendance'}
        </button>
      {/if}
    {/if}

    {#if attendanceMessage}<p class="form-message" role="status">{attendanceMessage}</p>{/if}
    {#if attendanceError}<p class="form-error" role="alert">{attendanceError}</p>{/if}
  {/if}
</section>

<style>
  .player-list,
  .attendance-list {
    display: grid;
    gap: 0.65rem;
  }

  .player-card,
  .attendance-row {
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
  .unassigned,
  .loading-message {
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

  .form-grid label,
  .event-picker,
  .absence-reason {
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

  .attendance-panel {
    margin-top: 1rem;
  }

  .event-picker {
    max-width: 34rem;
  }

  .selected-event {
    align-items: end;
    border-bottom: 1px solid var(--line);
    display: flex;
    gap: 1rem;
    justify-content: space-between;
    margin: 1.5rem 0 1rem;
    padding-bottom: 1rem;
  }

  .selected-event h3 {
    font-size: 1.2rem;
    letter-spacing: -0.03em;
    margin: 0.2rem 0;
  }

  .selected-event p:last-child {
    color: var(--muted);
    margin: 0;
  }

  .event-kind {
    color: var(--accent-dark);
    font-size: 0.7rem;
    font-weight: 800;
    letter-spacing: 0.08em;
    margin: 0;
    text-transform: uppercase;
  }

  .attendance-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    justify-content: end;
  }

  .secondary-button {
    background: transparent;
    border: 1px solid var(--line);
    color: var(--accent-dark);
    font-size: 0.8rem;
    min-height: 2.35rem;
  }

  .secondary-button:hover:not(:disabled) {
    background: #f4eee4;
  }

  .attendance-row {
    align-items: start;
  }

  .attendance-row.absent {
    border-color: #e2b8aa;
  }

  .attendance-player {
    align-items: center;
    display: flex;
    flex: 1;
    gap: 0.75rem;
    min-height: 2.7rem;
  }

  .attendance-player input {
    accent-color: var(--accent);
    min-height: 1.2rem;
    width: 1.2rem;
  }

  .attendance-player span {
    display: grid;
    gap: 0.15rem;
  }

  .attendance-player small {
    color: var(--muted);
  }

  .absence-reason {
    min-width: 9rem;
  }

  .present-label {
    color: #397044;
    font-size: 0.84rem;
    font-weight: 700;
    padding: 0.75rem 0;
  }

  .store-button {
    margin-top: 1rem;
    width: 100%;
  }

  .primary-button {
    background: var(--accent);
  }

  .compact {
    padding: 1.25rem;
  }

  @media (max-width: 48rem) {
    .two-column {
      grid-template-columns: 1fr;
    }
  }

  @media (max-width: 36rem) {
    .player-card,
    .attendance-row,
    .selected-event {
      align-items: stretch;
      flex-direction: column;
    }

    .membership-details {
      text-align: left;
    }

    .attendance-actions {
      justify-content: start;
    }

    .absence-reason {
      width: 100%;
    }
  }
</style>
