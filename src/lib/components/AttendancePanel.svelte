<script>
  import { invalidateAll } from '$app/navigation';
  import { onMount } from 'svelte';
  import { isAttendanceEligible } from '$lib/application/program/read-program';

  /** @typedef {import('$lib/application/participation/participation-repository').ParticipationRecord} ParticipationRecord */
  /** @typedef {import('$lib/application/program/read-program').ProgramEvent} ProgramEvent */
  /** @typedef {'illness' | 'injury' | 'other'} AbsenceReason */
  /** @typedef {Record<string, boolean>} AttendanceState */
  /** @typedef {Record<string, AbsenceReason>} AbsenceReasonState */

  /** @type {ProgramEvent[]} */
  export let events = [];
  /** @type {import('$lib/application/team/read-team').TeamPlayer[]} */
  export let players = [];
  /** @type {string} */
  export let teamName = '';
  /** @type {string | undefined} */
  export let selectedEventId = undefined;
  /** @type {boolean} */
  export let showEventPicker = true;
  /** @type {boolean} */
  export let showEventSummary = true;

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
    event.type === 'training' || isAttendanceEligible(event, teamName);
  const initialEvent =
    events.find((event) => event.id === selectedEventId && isAttendanceEvent(event)) ??
    events.find(isAttendanceEvent);

  let activeEventId = initialEvent?.id ?? '';
  /** @type {AttendanceState} */
  let attendance = {};
  /** @type {AbsenceReasonState} */
  let recordedAbsenceReasons = {};
  let attendanceLoading = false;
  let attendanceSaving = false;
  let attendanceMessage = '';
  let attendanceError = '';

  $: attendanceEvents = events.filter(isAttendanceEvent);
  $: selectedEvent = events.find((event) => event.id === activeEventId);
  $: currentTeamPlayers = players.filter((player) => player.membership?.teamName === teamName);
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
      const recordsByPlayer = new Map(records.map((record) => [record.playerId, record]));
      /** @type {AttendanceState} */
      const nextAttendance = {};
      /** @type {AbsenceReasonState} */
      const nextAbsenceReasons = {};

      for (const player of eligiblePlayersFor(event)) {
        const record = recordsByPlayer.get(player.id);
        nextAttendance[player.id] = record?.status !== 'absent';

        if (record?.status === 'absent') {
          nextAbsenceReasons[player.id] = record.absenceReason ?? 'other';
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
    attendance = Object.fromEntries(eligiblePlayers.map((player) => [player.id, present]));

    if (!present) {
      recordedAbsenceReasons = Object.fromEntries(
        eligiblePlayers.map((player) => [player.id, 'other']),
      );
    }
  };

  /** @param {string} playerId */
  const toggleTrainingAttendance = (playerId) => {
    const present = attendance[playerId] !== false;
    attendance = { ...attendance, [playerId]: !present };

    if (present) {
      recordedAbsenceReasons = { ...recordedAbsenceReasons, [playerId]: 'other' };
    }
  };

  /** @param {string} playerId @param {Event} event */
  const setPlayerAttendance = (playerId, event) => {
    const input = /** @type {HTMLInputElement} */ (event.currentTarget);
    attendance = { ...attendance, [playerId]: input.checked };

    if (!input.checked && !recordedAbsenceReasons[playerId]) {
      recordedAbsenceReasons = { ...recordedAbsenceReasons, [playerId]: 'other' };
    }
  };

  /** @param {string} playerId @param {Event} event */
  const setAbsenceReason = (playerId, event) => {
    const select = /** @type {HTMLSelectElement} */ (event.currentTarget);
    const reason = /** @type {AbsenceReason} */ (select.value);
    recordedAbsenceReasons = {
      ...recordedAbsenceReasons,
      [playerId]: reason,
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
            .filter((player) => attendance[player.id] === false)
            .map((player) => ({
              playerId: player.id,
              reason: recordedAbsenceReasons[player.id] ?? 'other',
            })),
          eligiblePlayerIds: eligiblePlayers.map((player) => player.id),
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
      await invalidateAll();
    } catch (error) {
      attendanceError = error instanceof Error ? error.message : 'Attendance could not be stored.';
    } finally {
      attendanceSaving = false;
    }
  };

  onMount(() => {
    if (selectedEvent) {
      void loadAttendance(selectedEvent);
    }
  });
</script>

<section class="panel attendance-panel" id="attendance" aria-labelledby="attendance-heading">
  <div class="panel-heading">
    <div>
      <p class="eyebrow">
        {#if selectedEvent?.type === 'training'}
          Tap players to toggle attendance
        {:else}
          Bulk workflow · {eligiblePlayers.length} eligible players
        {/if}
      </p>
      <h2 id="attendance-heading">Attendance</h2>
    </div>
  </div>

  {#if attendanceEvents.length === 0}
    <div class="empty-state">
      <h3>No attendance events yet</h3>
      <p>Add a training occurrence or scheduled game in Program before recording participation.</p>
    </div>
  {:else}
    {#if showEventPicker}
      <label class="event-picker">
        Event
        <select bind:value={activeEventId} on:change={handleEventChange}>
          {#each attendanceEvents as event (event.id)}
            <option value={event.id}>{formatDate(event.date)} · {eventTitle(event)}</option>
          {/each}
        </select>
      </label>
    {/if}

    {#if selectedEvent}
      <div class:summary-hidden={!showEventSummary} class="selected-event">
        {#if showEventSummary}
          <div>
            <p class="event-kind">{selectedEvent.type === 'training' ? 'Training' : 'Game'}</p>
            <h3>{eventTitle(selectedEvent)}</h3>
            <p>{selectedEvent.startTime} · {eventSummary(selectedEvent)}</p>
          </div>
        {/if}
        <div class="attendance-actions">
          <button
            class="secondary-button"
            disabled={attendanceLoading || attendanceSaving}
            type="button"
            on:click={() => setAllAttendance(true)}
          >
            Select all present
          </button>
          <button
            class="secondary-button"
            disabled={attendanceLoading || attendanceSaving}
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
            Active {teamName} memberships appear here. Training-only players are excluded from games.
          </p>
        </div>
      {:else if selectedEvent.type === 'training'}
        <div class="attendance-grid" aria-label="Training attendance">
          {#each eligiblePlayers as player (player.id)}
            <button
              aria-label={`${player.firstName}: ${attendance[player.id] !== false ? 'present' : 'absent'}`}
              aria-pressed={attendance[player.id] !== false}
              class:present={attendance[player.id] !== false}
              class:absent={attendance[player.id] === false}
              class="attendance-tile"
              disabled={attendanceSaving}
              type="button"
              on:click={() => toggleTrainingAttendance(player.id)}
            >
              <strong>{player.firstName}</strong>
              <small
                >{player.normalAgeGroup}{#if player.membership?.jerseyNumber}
                  · #{player.membership.jerseyNumber}{/if}</small
              >
              <span class="attendance-tile-status">
                {attendance[player.id] !== false ? 'Present' : 'Absent'}
              </span>
            </button>
          {/each}
        </div>
      {:else}
        <div class="attendance-list">
          {#each eligiblePlayers as player (player.id)}
            <div class:absent={attendance[player.id] === false} class="attendance-row">
              <label class="attendance-player">
                <input
                  checked={attendance[player.id] !== false}
                  type="checkbox"
                  on:change={(event) => setPlayerAttendance(player.id, event)}
                />
                <span>
                  <strong>{player.firstName}</strong>
                  <small
                    >{player.normalAgeGroup}{#if player.membership?.jerseyNumber}
                      · #{player.membership.jerseyNumber}{/if}</small
                  ></span
                >
              </label>
              {#if attendance[player.id] === false}
                <label class="absence-reason">
                  <span>Absence reason</span>
                  <select
                    value={recordedAbsenceReasons[player.id] ?? 'other'}
                    on:change={(event) => setAbsenceReason(player.id, event)}
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
      {/if}

      <button
        class="primary-button store-button"
        disabled={attendanceLoading || attendanceSaving}
        type="button"
        on:click={storeAttendance}
      >
        {attendanceSaving ? 'Storing…' : 'Store attendance'}
      </button>
    {/if}

    {#if attendanceMessage}<p class="form-message" role="status">{attendanceMessage}</p>{/if}
    {#if attendanceError}<p class="form-error" role="alert">{attendanceError}</p>{/if}
  {/if}
</section>

<style>
  .attendance-list,
  .attendance-grid {
    display: grid;
    gap: 0.65rem;
  }

  .attendance-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
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
    color: var(--muted);
    display: grid;
    font-size: 0.78rem;
    font-weight: 800;
    gap: 0.35rem;
    letter-spacing: 0.04em;
    max-width: 34rem;
    text-transform: uppercase;
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

  .selected-event.summary-hidden {
    align-items: start;
    border-bottom: 0;
    justify-content: start;
    margin: 1rem 0;
    padding-bottom: 0;
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

  .attendance-tile {
    align-items: start;
    background: #fffdf8;
    border: 2px solid var(--line);
    color: var(--ink);
    display: grid;
    gap: 0.25rem;
    min-height: 6rem;
    padding: 0.9rem;
    text-align: left;
  }

  .attendance-tile:hover:not(:disabled) {
    background: #f4eee4;
  }

  .attendance-tile.present {
    background: #eef6e9;
    border-color: #91ae83;
  }

  .attendance-tile.present:hover:not(:disabled) {
    background: #e4f0dd;
  }

  .attendance-tile.absent {
    background: #fff4ef;
    border-color: #e2b8aa;
  }

  .attendance-tile strong {
    font-size: 0.95rem;
  }

  .attendance-tile small {
    color: var(--muted);
  }

  .attendance-tile-status {
    color: var(--muted);
    font-size: 0.72rem;
    font-weight: 800;
    letter-spacing: 0.08em;
    margin-top: 0.35rem;
    text-transform: uppercase;
  }

  .attendance-tile.present .attendance-tile-status {
    color: #397044;
  }

  .attendance-tile.absent .attendance-tile-status {
    color: var(--accent-dark);
  }

  .attendance-row {
    align-items: start;
    background: #faf7f0;
    border: 1px solid #ebe4d8;
    border-radius: 0.9rem;
    display: flex;
    gap: 1rem;
    justify-content: space-between;
    padding: 1rem;
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
    color: var(--muted);
    display: grid;
    font-size: 0.78rem;
    font-weight: 800;
    gap: 0.35rem;
    letter-spacing: 0.04em;
    min-width: 9rem;
    text-transform: uppercase;
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

  @media (max-width: 36rem) {
    .selected-event {
      align-items: start;
      flex-direction: column;
    }

    .attendance-actions {
      justify-content: start;
    }

    .attendance-row {
      align-items: stretch;
      flex-direction: column;
    }

    .absence-reason {
      width: 100%;
    }
  }
</style>
