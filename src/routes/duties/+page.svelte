<script>
  /** @typedef {import('$lib/application/duties/duty-repository').DutyView} DutyView */
  /** @typedef {import('$lib/application/duties/duty-repository').DutySlot} DutySlot */
  /** @typedef {import('$lib/application/duties/duty-repository').DutySlotStatus} DutySlotStatus */
  /** @typedef {import('$lib/application/duties/duty-repository').DutyType} DutyType */
  /** @typedef {import('$lib/application/program/read-program').ProgramGameEvent} ProgramGameEvent */
  /** @typedef {import('./$types').PageData} PageData */

  /** @type {PageData} */
  export let data;

  const statusOptions = ['open', 'assigned', 'completed', 'incomplete', 'cancelled'];
  const signupStatuses = [
    { value: 'volunteer', label: 'Volunteer' },
    { value: 'waitlisted', label: 'Waitlisted' },
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

  /** @param {DutyType} type */
  const dutyLabel = (type) => ({ driving: 'Driving', jury: 'Jury', referee: 'Referee' })[type];

  /** @param {ProgramGameEvent} event */
  const eventTitle = (event) => `${event.homeTeamName} vs ${event.awayTeamName}`;

  const initialEvent = data.events[0];
  let selectedEventId = initialEvent?.id ?? '';
  let dutyViews = data.duties;
  let configuration = initialEvent
    ? { ...data.duties[initialEvent.id].requirements }
    : { drivingSlots: 0, jurySlots: 0, refereeSlots: 0 };
  let signupPlayerAssociationId = data.players[0]?.associationId ?? '';
  let signupType = 'referee';
  let signupStatus = 'volunteer';
  let savingConfiguration = false;
  let savingSignup = false;
  let message = '';
  let error = '';

  $: selectedEvent = data.events.find((event) => event.id === selectedEventId);
  $: selectedDuties = dutyViews[selectedEventId];
  $: eligiblePlayers = data.players.filter(
    (player) =>
      player.membership?.teamName === data.teamName &&
      player.membership.status === 'active' &&
      player.membership.participationType === 'trains_and_plays',
  );

  /** @param {string} id */
  const playerName = (id) => data.players.find((player) => player.associationId === id)?.name ?? id;

  /** @param {Event} event */
  const handleGameChange = (event) => {
    const select = /** @type {HTMLSelectElement} */ (event.currentTarget);
    selectedEventId = select.value;
    const duties = dutyViews[selectedEventId];
    configuration = duties
      ? { ...duties.requirements }
      : { drivingSlots: 0, jurySlots: 0, refereeSlots: 0 };
    message = '';
    error = '';
  };

  /** @param {string} path @param {unknown} payload @returns {Promise<unknown>} */
  const postJson = async (path, payload) => {
    const response = await fetch(path, {
      body: JSON.stringify(payload),
      headers: { 'content-type': 'application/json' },
      method: 'POST',
    });
    const body = await response.json().catch(() => ({}));

    if (!response.ok) {
      const errorBody = /** @type {{ error?: string }} */ (body);
      throw new Error(errorBody.error ?? 'The change could not be stored.');
    }

    return body;
  };

  const storeConfiguration = async () => {
    if (!selectedEvent) return;

    savingConfiguration = true;
    message = '';
    error = '';

    try {
      const view = /** @type {DutyView} */ (
        await postJson('/api/duties', {
          ...configuration,
          occurrenceId: selectedEvent.id,
        })
      );
      dutyViews = { ...dutyViews, [selectedEvent.id]: view };
      configuration = { ...view.requirements };
      message = 'Duty requirements stored.';
    } catch (caught) {
      error = caught instanceof Error ? caught.message : 'Duty requirements could not be stored.';
    } finally {
      savingConfiguration = false;
    }
  };

  const storeSignup = async () => {
    if (!selectedEvent || !signupPlayerAssociationId) return;

    savingSignup = true;
    message = '';
    error = '';

    try {
      const view = /** @type {DutyView} */ (
        await postJson('/api/duties/signups', {
          dutyType: signupType,
          occurrenceId: selectedEvent.id,
          playerAssociationId: signupPlayerAssociationId,
          status: signupStatus,
        })
      );
      dutyViews = { ...dutyViews, [selectedEvent.id]: view };
      message = 'Signup stored.';
    } catch (caught) {
      error = caught instanceof Error ? caught.message : 'Signup could not be stored.';
    } finally {
      savingSignup = false;
    }
  };

  /** @param {string} slotId @param {Event} event */
  const assignSlot = async (slotId, event) => {
    const select = /** @type {HTMLSelectElement} */ (event.currentTarget);
    if (!select.value) return;

    error = '';

    try {
      const view = /** @type {DutyView} */ (
        await postJson('/api/duties/assignments', {
          playerAssociationId: select.value,
          slotId,
        })
      );
      dutyViews = { ...dutyViews, [selectedEventId]: view };
      message = 'Duty assignment stored.';
    } catch (caught) {
      error = caught instanceof Error ? caught.message : 'Duty assignment could not be stored.';
    }
  };

  /** @param {string} slotId @param {Event} event */
  const updateStatus = async (slotId, event) => {
    const select = /** @type {HTMLSelectElement} */ (event.currentTarget);
    error = '';

    try {
      const view = /** @type {DutyView} */ (
        await postJson(`/api/duties/slots/${slotId}/status`, { status: select.value })
      );
      dutyViews = { ...dutyViews, [selectedEventId]: view };
      message = 'Duty status corrected.';
    } catch (caught) {
      error = caught instanceof Error ? caught.message : 'Duty status could not be corrected.';
    }
  };
</script>

<svelte:head>
  <title>Duties · Team</title>
  <meta name="description" content="Game duties and transport coordination" />
</svelte:head>

<section class="intro">
  <p class="eyebrow">{data.teamName} · Game coordination</p>
  <h1>Duties</h1>
  <p class="lede">
    Configure referee, jury, and driving slots, then turn chat signups into assignments.
  </p>
</section>

<section class="panel" aria-labelledby="game-heading">
  <div class="panel-heading">
    <div>
      <p class="eyebrow">{data.events.length} own games</p>
      <h2 id="game-heading">Game duties and transport</h2>
    </div>
  </div>

  {#if data.events.length === 0}
    <div class="empty-state">
      <h3>No own games configured yet</h3>
      <p>Games involving {data.teamName} will appear here when they are added to the Program.</p>
    </div>
  {:else if selectedEvent && selectedDuties}
    <label class="event-picker">
      Game
      <select value={selectedEventId} on:change={handleGameChange}>
        {#each data.events as event (event.id)}
          <option value={event.id}>{formatDate(event.date)} · {eventTitle(event)}</option>
        {/each}
      </select>
    </label>

    <div class="game-summary">
      <div>
        <p class="event-kind">
          {selectedEvent.status === 'scheduled' ? 'Scheduled game' : 'Cancelled game'}
        </p>
        <h3>{eventTitle(selectedEvent)}</h3>
        <p>{selectedEvent.startTime} · {selectedEvent.locationName}</p>
      </div>
      <div class="departure-card">
        <span>Suggested departure</span>
        <strong>{selectedEvent.suggestedDepartureTime}</strong>
        <small>{selectedEvent.travelMinutes} minutes travel + arrival buffer</small>
      </div>
    </div>

    <div class="two-column">
      <section class="subpanel" aria-labelledby="requirements-heading">
        <div class="subpanel-heading">
          <div>
            <p class="eyebrow">Capacity</p>
            <h3 id="requirements-heading">Duty requirements</h3>
          </div>
        </div>
        <div class="form-grid requirements-grid">
          <label>
            Referee slots
            <input bind:value={configuration.refereeSlots} max="2" min="0" type="number" />
          </label>
          <label>
            Jury slots
            <input bind:value={configuration.jurySlots} max="2" min="0" type="number" />
          </label>
          <label>
            Driving slots
            <input bind:value={configuration.drivingSlots} max="2" min="0" type="number" />
          </label>
          <button disabled={savingConfiguration} type="button" on:click={storeConfiguration}>
            {savingConfiguration ? 'Saving…' : 'Store requirements'}
          </button>
        </div>
        <p class="hint">
          Driving slots apply to away games. Referee and jury slots can be configured independently.
        </p>
      </section>

      <section class="subpanel" aria-labelledby="signup-heading">
        <div class="subpanel-heading">
          <div>
            <p class="eyebrow">From chat</p>
            <h3 id="signup-heading">Record a signup</h3>
          </div>
        </div>
        {#if eligiblePlayers.length === 0}
          <p class="hint">Active players who train and play will be available for duty signups.</p>
        {:else}
          <div class="form-grid">
            <label>
              Player
              <select bind:value={signupPlayerAssociationId}>
                {#each eligiblePlayers as player (player.associationId)}
                  <option value={player.associationId}>{player.name}</option>
                {/each}
              </select>
            </label>
            <label>
              Duty type
              <select bind:value={signupType}>
                <option value="referee">Referee</option>
                <option value="jury">Jury</option>
                <option value="driving">Driving</option>
              </select>
            </label>
            <label>
              Signup status
              <select bind:value={signupStatus}>
                {#each signupStatuses as status (status.value)}
                  <option value={status.value}>{status.label}</option>
                {/each}
              </select>
            </label>
            <button disabled={savingSignup} type="button" on:click={storeSignup}>
              {savingSignup ? 'Saving…' : 'Store signup'}
            </button>
          </div>
        {/if}
      </section>
    </div>

    <div class="duty-columns">
      <section class="subpanel" aria-labelledby="slots-heading">
        <div class="subpanel-heading">
          <div>
            <p class="eyebrow">Confirm and correct</p>
            <h3 id="slots-heading">Duty slots</h3>
          </div>
        </div>
        {#if selectedDuties.slots.length === 0}
          <div class="empty-state compact">
            <h3>No slots configured</h3>
            <p>Set the referee, jury, or driving requirements above.</p>
          </div>
        {:else}
          <div class="slot-list">
            {#each selectedDuties.slots as slot (slot.id)}
              <div class="slot-row">
                <div class="slot-title">
                  <strong>{dutyLabel(slot.dutyType)} {slot.slotNumber}</strong>
                  <span class:status-complete={slot.status === 'completed'}>{slot.status}</span>
                </div>
                <label>
                  Assigned player
                  <select
                    value={slot.assignedPlayerAssociationId ?? ''}
                    disabled={slot.status === 'cancelled' || slot.status === 'completed'}
                    on:change={(event) => assignSlot(slot.id, event)}
                  >
                    <option value="">Unassigned</option>
                    {#each eligiblePlayers as player (player.associationId)}
                      <option value={player.associationId}>{player.name}</option>
                    {/each}
                  </select>
                </label>
                <label>
                  Status
                  <select value={slot.status} on:change={(event) => updateStatus(slot.id, event)}>
                    {#each statusOptions as status (status)}
                      <option value={status}>{status}</option>
                    {/each}
                  </select>
                </label>
              </div>
            {/each}
          </div>
        {/if}
      </section>

      <section class="subpanel" aria-labelledby="signups-heading">
        <div class="subpanel-heading">
          <div>
            <p class="eyebrow">Retained history</p>
            <h3 id="signups-heading">Signups</h3>
          </div>
        </div>
        {#if selectedDuties.signups.length === 0}
          <div class="empty-state compact">
            <h3>No signups recorded</h3>
            <p>Keep every volunteer here, including people who are not selected.</p>
          </div>
        {:else}
          <div class="signup-list">
            {#each selectedDuties.signups as signup (signup.playerAssociationId + signup.dutyType)}
              <div class="signup-row">
                <span
                  ><strong>{playerName(signup.playerAssociationId)}</strong> · {dutyLabel(
                    signup.dutyType,
                  )}</span
                >
                <span class="signup-status">{signup.status}</span>
              </div>
            {/each}
          </div>
        {/if}
      </section>
    </div>

    <section class="subpanel history-panel" aria-labelledby="assignment-history-heading">
      <div class="subpanel-heading">
        <div>
          <p class="eyebrow">Reassignment trail</p>
          <h3 id="assignment-history-heading">Assignment history</h3>
        </div>
      </div>
      {#if selectedDuties.assignmentHistory.length === 0}
        <p class="hint">Assignments will be retained here when slots are confirmed or changed.</p>
      {:else}
        <div class="signup-list">
          {#each selectedDuties.assignmentHistory as history, index (`${history.slotId}-${history.playerAssociationId}-${history.status}-${index}`)}
            <div class="signup-row">
              <span
                ><strong>{playerName(history.playerAssociationId)}</strong> · {dutyLabel(
                  history.dutyType,
                )}</span
              >
              <span class="signup-status">{history.status}</span>
            </div>
          {/each}
        </div>
      {/if}
    </section>

    <section class="subpanel fairness-panel" aria-labelledby="fairness-heading">
      <div class="subpanel-heading">
        <div>
          <p class="eyebrow">Completed assignments only</p>
          <h3 id="fairness-heading">Duty balance</h3>
        </div>
      </div>
      {#if selectedDuties.fairness.length === 0}
        <p class="hint">No completed duties have been recorded yet.</p>
      {:else}
        <div class="fairness-list">
          {#each selectedDuties.fairness as player (player.playerAssociationId)}
            <div class="fairness-row">
              <span>{playerName(player.playerAssociationId)}</span>
              <strong>{player.completedCount}</strong>
            </div>
          {/each}
        </div>
      {/if}
    </section>

    {#if message}<p class="form-message" role="status">{message}</p>{/if}
    {#if error}<p class="form-error" role="alert">{error}</p>{/if}
  {/if}
</section>

<style>
  .event-picker,
  .form-grid label,
  .slot-row label {
    color: var(--muted);
    display: grid;
    font-size: 0.78rem;
    font-weight: 800;
    gap: 0.35rem;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .event-picker {
    max-width: 36rem;
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

  input:focus,
  select:focus,
  button:focus-visible {
    outline: 3px solid rgba(213, 99, 62, 0.25);
    outline-offset: 2px;
  }

  .game-summary {
    align-items: end;
    border-bottom: 1px solid var(--line);
    display: flex;
    gap: 1rem;
    justify-content: space-between;
    margin: 1.5rem 0 1rem;
    padding-bottom: 1rem;
  }

  .game-summary h3 {
    font-size: 1.35rem;
    letter-spacing: -0.04em;
    margin: 0.2rem 0;
  }

  .game-summary p:last-child {
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

  .departure-card {
    background: #faf7f0;
    border: 1px solid #ebe4d8;
    border-radius: 0.9rem;
    display: grid;
    gap: 0.2rem;
    min-width: 11rem;
    padding: 0.8rem 1rem;
  }

  .departure-card span,
  .departure-card small {
    color: var(--muted);
    font-size: 0.78rem;
  }

  .departure-card strong {
    font-size: 1.35rem;
  }

  .two-column,
  .duty-columns {
    display: grid;
    gap: 1rem;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    margin-top: 1rem;
  }

  .subpanel {
    background: #faf7f0;
    border: 1px solid #ebe4d8;
    border-radius: 0.9rem;
    padding: 1rem;
  }

  .subpanel-heading {
    align-items: end;
    display: flex;
    justify-content: space-between;
    margin-bottom: 1rem;
  }

  .subpanel-heading h3 {
    font-size: 1.15rem;
    letter-spacing: -0.03em;
    margin: 0.15rem 0 0;
  }

  .form-grid {
    display: grid;
    gap: 0.8rem;
  }

  .requirements-grid {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }

  .requirements-grid button {
    grid-column: 1 / -1;
  }

  .hint {
    color: var(--muted);
    font-size: 0.84rem;
    line-height: 1.45;
    margin: 0.9rem 0 0;
  }

  .slot-list,
  .signup-list,
  .fairness-list {
    display: grid;
    gap: 0.55rem;
  }

  .slot-row {
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 0.7rem;
    display: grid;
    gap: 0.65rem;
    grid-template-columns: minmax(7rem, 0.7fr) 1fr 8rem;
    padding: 0.75rem;
  }

  .slot-title {
    display: grid;
    gap: 0.2rem;
    align-content: center;
  }

  .slot-title span,
  .signup-status {
    color: var(--muted);
    font-size: 0.76rem;
    text-transform: capitalize;
  }

  .status-complete {
    color: #397044 !important;
  }

  .signup-row,
  .fairness-row {
    align-items: center;
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 0.7rem;
    display: flex;
    gap: 1rem;
    justify-content: space-between;
    padding: 0.75rem;
  }

  .signup-status {
    text-transform: capitalize;
  }

  .history-panel,
  .fairness-panel {
    margin-top: 1rem;
  }

  .fairness-list {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }

  .fairness-row strong {
    color: var(--accent-dark);
    font-size: 1.2rem;
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

  .compact {
    padding: 1rem;
  }

  @media (max-width: 56rem) {
    .requirements-grid {
      grid-template-columns: 1fr;
    }

    .requirements-grid button {
      grid-column: auto;
    }

    .slot-row {
      grid-template-columns: 1fr 1fr;
    }

    .slot-title {
      grid-column: 1 / -1;
    }
  }

  @media (max-width: 48rem) {
    .two-column,
    .duty-columns {
      grid-template-columns: 1fr;
    }

    .fairness-list {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }

  @media (max-width: 36rem) {
    .game-summary {
      align-items: stretch;
      flex-direction: column;
    }

    .departure-card {
      min-width: 0;
    }

    .slot-row {
      grid-template-columns: 1fr;
    }

    .slot-title {
      grid-column: auto;
    }

    .fairness-list {
      grid-template-columns: 1fr;
    }
  }
</style>
