<script>
  import { onMount } from 'svelte';

  /** @typedef {import('$lib/application/duties/duty-repository').DutyView} DutyView */
  /** @typedef {import('$lib/application/duties/duty-repository').DutyType} DutyType */

  /** @type {string} */
  export let occurrenceId;
  /** @type {import('$lib/application/team/read-team').TeamPlayer[]} */
  export let players = [];
  /** @type {string} */
  export let teamName = '';
  /** @type {boolean} */
  export let showDriving = true;
  /** @type {boolean} */
  export let showRefereeJury = true;

  const statusOptions = ['open', 'assigned', 'completed', 'incomplete', 'cancelled'];
  const signupStatuses = [
    { value: 'volunteer', label: 'Volunteer' },
    { value: 'waitlisted', label: 'Waitlisted' },
  ];
  /** @typedef {'drivingSlots' | 'jurySlots' | 'refereeSlots'} RequirementKey */

  /** @type {Record<RequirementKey, string>} */
  const requirementLabels = { drivingSlots: 'Driving', jurySlots: 'Jury', refereeSlots: 'Referee' };
  /** @type {RequirementKey[]} */
  const requirementOrder = ['refereeSlots', 'jurySlots', 'drivingSlots'];

  /** @param {DutyType} type */
  const dutyLabel = (type) => ({ driving: 'Driving', jury: 'Jury', referee: 'Referee' })[type];

  /** @param {string} id */
  const playerName = (id) => players.find((player) => player.id === id)?.firstName ?? id;

  /** @type {DutyView | undefined} */
  let duties = undefined;
  let loading = true;
  let loadError = '';
  let configuration = { drivingSlots: 0, jurySlots: 0, refereeSlots: 0 };
  /** @type {import('$lib/application/participation/participation-repository').ParticipationRecord[]} */
  let participationRecords = [];
  let signupPlayerId = players[0]?.id ?? '';
  let signupType = 'referee';
  let signupStatus = 'volunteer';
  let savingConfiguration = false;
  let savingSignup = false;
  let message = '';
  let error = '';

  $: eligiblePlayers = players.filter(
    (player) =>
      player.membership?.teamName === teamName &&
      player.membership.status === 'active' &&
      player.membership.participationType === 'trains_and_plays',
  );
  $: visibleRequirementKeys = requirementOrder.filter((key) =>
    key === 'drivingSlots' ? showDriving : showRefereeJury,
  );
  $: attendanceRecorded = participationRecords.length > 0;
  $: attendingCount = attendanceRecorded
    ? participationRecords.filter((record) => record.status === 'present').length
    : eligiblePlayers.length;
  $: suggestedDrivingSlots = Math.min(2, Math.ceil(attendingCount / 3));

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

  const load = async () => {
    loading = true;
    loadError = '';

    try {
      const response = await fetch(`/api/duties?occurrenceId=${encodeURIComponent(occurrenceId)}`);

      if (!response.ok) {
        throw new Error('Duty details could not be loaded.');
      }

      const view = /** @type {DutyView} */ (await response.json());
      duties = view;
      configuration = { ...view.requirements };
    } catch (caught) {
      loadError = caught instanceof Error ? caught.message : 'Duty details could not be loaded.';
    } finally {
      loading = false;
    }
  };

  const loadAttendance = async () => {
    try {
      const response = await fetch(
        `/api/participation?occurrenceType=game&occurrenceId=${encodeURIComponent(occurrenceId)}`,
      );

      if (response.ok) {
        participationRecords = await response.json();
      }
    } catch {
      // Non-critical: the driving suggestion just falls back to the full roster.
    }
  };

  onMount(() => {
    void load();
    if (showDriving) {
      void loadAttendance();
    }
  });

  /** @param {{ drivingSlots: number; jurySlots: number; refereeSlots: number }} nextConfiguration */
  const saveRequirements = async (nextConfiguration) => {
    savingConfiguration = true;
    message = '';
    error = '';

    try {
      const view = /** @type {DutyView} */ (
        await postJson('/api/duties', {
          ...nextConfiguration,
          occurrenceId,
        })
      );
      duties = view;
      configuration = { ...view.requirements };
      message = 'Duty requirements stored.';
    } catch (caught) {
      error = caught instanceof Error ? caught.message : 'Duty requirements could not be stored.';
    } finally {
      savingConfiguration = false;
    }
  };

  /** @param {'drivingSlots' | 'jurySlots' | 'refereeSlots'} key @param {1 | -1} delta */
  const adjustRequirement = (key, delta) => {
    const next = Math.min(2, Math.max(0, configuration[key] + delta));

    if (next === configuration[key]) return;

    void saveRequirements({ ...configuration, [key]: next });
  };

  const applySuggestedDriving = () => {
    void saveRequirements({ ...configuration, drivingSlots: suggestedDrivingSlots });
  };

  const storeSignup = async () => {
    if (!signupPlayerId) return;

    savingSignup = true;
    message = '';
    error = '';

    try {
      const view = /** @type {DutyView} */ (
        await postJson('/api/duties/signups', {
          dutyType: signupType,
          occurrenceId,
          playerId: signupPlayerId,
          status: signupStatus,
        })
      );
      duties = view;
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
          playerId: select.value,
          slotId,
        })
      );
      duties = view;
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
      duties = view;
      message = 'Duty status corrected.';
    } catch (caught) {
      error = caught instanceof Error ? caught.message : 'Duty status could not be corrected.';
    }
  };
</script>

{#if loading}
  <p class="loading-message" role="status">Loading duty details…</p>
{:else if loadError}
  <p class="form-error" role="alert">{loadError}</p>
{:else if duties}
  <section class="subpanel" aria-labelledby="requirements-heading">
    <div class="subpanel-heading">
      <div>
        <p class="eyebrow">Capacity</p>
        <h3 id="requirements-heading">Duty requirements</h3>
      </div>
      {#if savingConfiguration}<span class="saving-indicator">Saving…</span>{/if}
    </div>
    <div class="stepper-list">
      {#each visibleRequirementKeys as key (key)}
        <div class="stepper-row">
          <div class="stepper-main">
            <span class="stepper-label">{requirementLabels[key]}</span>
            <div class="stepper-controls">
              <button
                aria-label={`Decrease ${requirementLabels[key].toLowerCase()} slots`}
                disabled={savingConfiguration || configuration[key] <= 0}
                type="button"
                on:click={() => adjustRequirement(key, -1)}
              >
                −
              </button>
              <span class="stepper-count">{configuration[key]}</span>
              <button
                aria-label={`Increase ${requirementLabels[key].toLowerCase()} slots`}
                disabled={savingConfiguration || configuration[key] >= 2}
                type="button"
                on:click={() => adjustRequirement(key, 1)}
              >
                +
              </button>
            </div>
          </div>
          {#if key === 'drivingSlots'}
            <p class="stepper-suggestion">
              {attendingCount}
              {attendanceRecorded ? 'attending' : 'on the roster (no attendance recorded yet)'} · suggest
              {suggestedDrivingSlots}
              {#if suggestedDrivingSlots !== configuration.drivingSlots}
                <button
                  class="suggestion-button"
                  disabled={savingConfiguration}
                  type="button"
                  on:click={applySuggestedDriving}
                >
                  Use {suggestedDrivingSlots}
                </button>
              {/if}
            </p>
          {/if}
        </div>
      {/each}
    </div>
    <p class="hint">
      Referee, jury, and driving can each be configured independently — set whichever applies to
      this game.
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
          <select bind:value={signupPlayerId}>
            {#each eligiblePlayers as player (player.id)}
              <option value={player.id}>{player.firstName}</option>
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

  <section class="subpanel" aria-labelledby="slots-heading">
    <div class="subpanel-heading">
      <div>
        <p class="eyebrow">Confirm and correct</p>
        <h3 id="slots-heading">Duty slots</h3>
      </div>
    </div>
    {#if duties.slots.length === 0}
      <div class="empty-state compact">
        <h3>No slots configured</h3>
        <p>Set the referee, jury, or driving requirements above.</p>
      </div>
    {:else}
      <div class="slot-list">
        {#each duties.slots as slot (slot.id)}
          <div class="slot-row">
            <div class="slot-title">
              <strong>{dutyLabel(slot.dutyType)} {slot.slotNumber}</strong>
              <span class:status-complete={slot.status === 'completed'}>{slot.status}</span>
            </div>
            <label>
              Assigned player
              <select
                value={slot.assignedPlayerId ?? ''}
                disabled={slot.status === 'cancelled' || slot.status === 'completed'}
                on:change={(event) => assignSlot(slot.id, event)}
              >
                <option value="">Unassigned</option>
                {#each eligiblePlayers as player (player.id)}
                  <option value={player.id}>{player.firstName}</option>
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
    {#if duties.signups.length === 0}
      <div class="empty-state compact">
        <h3>No signups recorded</h3>
        <p>Keep every volunteer here, including people who are not selected.</p>
      </div>
    {:else}
      <div class="signup-list">
        {#each duties.signups as signup (signup.playerId + signup.dutyType)}
          <div class="signup-row">
            <span
              ><strong>{playerName(signup.playerId)}</strong> · {dutyLabel(signup.dutyType)}</span
            >
            <span class="signup-status">{signup.status}</span>
          </div>
        {/each}
      </div>
    {/if}
  </section>

  <section class="subpanel history-panel" aria-labelledby="assignment-history-heading">
    <div class="subpanel-heading">
      <div>
        <p class="eyebrow">Reassignment trail</p>
        <h3 id="assignment-history-heading">Assignment history</h3>
      </div>
    </div>
    {#if duties.assignmentHistory.length === 0}
      <p class="hint">Assignments will be retained here when slots are confirmed or changed.</p>
    {:else}
      <div class="signup-list">
        {#each duties.assignmentHistory as history, index (`${history.slotId}-${history.playerId}-${history.status}-${index}`)}
          <div class="signup-row">
            <span
              ><strong>{playerName(history.playerId)}</strong> · {dutyLabel(history.dutyType)}</span
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
    {#if duties.fairness.length === 0}
      <p class="hint">No completed duties have been recorded yet.</p>
    {:else}
      <div class="fairness-list">
        {#each duties.fairness as player (player.playerId)}
          <div class="fairness-row">
            <span>{playerName(player.playerId)}</span>
            <strong>{player.completedCount}</strong>
          </div>
        {/each}
      </div>
    {/if}
  </section>

  {#if message}<p class="form-message" role="status">{message}</p>{/if}
  {#if error}<p class="form-error" role="alert">{error}</p>{/if}
{/if}

<style>
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

  select:focus,
  button:focus-visible {
    outline: 3px solid rgba(213, 99, 62, 0.25);
    outline-offset: 2px;
  }

  .subpanel {
    background: #faf7f0;
    border: 1px solid #ebe4d8;
    border-radius: 0.9rem;
    margin-top: 1rem;
    padding: 1rem;
  }

  .subpanel:first-child {
    margin-top: 0;
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

  .saving-indicator {
    color: var(--muted);
    font-size: 0.78rem;
    font-weight: 800;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .stepper-list {
    display: grid;
    gap: 0.55rem;
  }

  .stepper-row {
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 0.7rem;
    padding: 0.65rem 0.75rem;
  }

  .stepper-main {
    align-items: center;
    display: flex;
    justify-content: space-between;
  }

  .stepper-label {
    font-weight: 700;
  }

  .stepper-controls {
    align-items: center;
    display: flex;
    gap: 0.6rem;
  }

  .stepper-controls button {
    align-items: center;
    border-radius: 50%;
    display: flex;
    font-size: 1.1rem;
    height: 2.4rem;
    justify-content: center;
    min-height: 0;
    padding: 0;
    width: 2.4rem;
  }

  .stepper-count {
    font-size: 1.1rem;
    font-weight: 800;
    min-width: 1.4rem;
    text-align: center;
  }

  .stepper-suggestion {
    align-items: center;
    color: var(--muted);
    display: flex;
    flex-wrap: wrap;
    font-size: 0.8rem;
    gap: 0.5rem;
    margin: 0.5rem 0 0;
  }

  .suggestion-button {
    background: transparent;
    border: 1px solid var(--line);
    color: var(--accent-dark);
    font-size: 0.76rem;
    min-height: 1.9rem;
    padding: 0.3rem 0.65rem;
  }

  .suggestion-button:hover:not(:disabled) {
    background: #f4eee4;
  }

  .hint {
    color: var(--muted);
    font-size: 0.84rem;
    line-height: 1.45;
    margin: 0.9rem 0 0;
  }

  .loading-message {
    color: var(--muted);
    font-size: 0.88rem;
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
    grid-template-columns: repeat(2, minmax(0, 1fr));
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

  @media (max-width: 30rem) {
    .fairness-list {
      grid-template-columns: 1fr;
    }
  }
</style>
