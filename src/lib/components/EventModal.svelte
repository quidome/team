<script>
  import AttendancePanel from './AttendancePanel.svelte';
  import DutyPanel from './DutyPanel.svelte';
  import { isAttendanceEligible } from '$lib/application/program/read-program';
  import { deriveSeasonHalf } from '$lib/domain/season-half';

  /** @typedef {import('$lib/application/program/read-program').ProgramEvent} ProgramEvent */

  /** @type {ProgramEvent} */
  export let event;
  /** @type {import('$lib/application/team/read-team').TeamPlayer[]} */
  export let players = [];
  /** @type {string} */
  export let teamName = '';
  /** @type {number} */
  export let seasonStartingYear;
  /** @type {() => void} */
  export let onClose;

  /** @param {string} date */
  const formatDate = (date) =>
    new Intl.DateTimeFormat('en', {
      day: 'numeric',
      month: 'long',
      timeZone: 'UTC',
      weekday: 'long',
      year: 'numeric',
    }).format(new Date(`${date}T00:00:00.000Z`));

  $: title =
    event.type === 'training' ? 'Team training' : `${event.homeTeamName} vs ${event.awayTeamName}`;
  $: seasonHalf =
    event.type === 'game' ? deriveSeasonHalf(event.date, seasonStartingYear) : undefined;
  $: showAttendance = event.type === 'training' || isAttendanceEligible(event, teamName);
  $: isAwayGame = event.type === 'game' && event.awayTeamName === teamName;
  $: isHomeGame = event.type === 'game' && event.homeTeamName === teamName;
  $: isDutyOnlyGame = event.type === 'game' && !isHomeGame && !isAwayGame;
  $: showDriving = isAwayGame;
  $: showRefereeJury = isDutyOnlyGame;
  $: showDuty = showDriving || showRefereeJury;

  /** @type {import('$lib/application/participation/participation-repository').ParticipationRecord[]} */
  let participationRecords = [];
</script>

<div class="modal-backdrop" role="presentation" on:click|self={onClose}>
  <div class="panel modal" role="dialog" aria-modal="true" aria-labelledby="event-modal-heading">
    <div class="modal-header">
      <div>
        <p class="eyebrow">
          {formatDate(event.date)} · {event.startTime} · {event.locationName}{#if seasonHalf}&nbsp;·&nbsp;{seasonHalf}{/if}
        </p>
        <h2 id="event-modal-heading">{title}</h2>
      </div>
      <button class="close-button" type="button" on:click={onClose}>Close</button>
    </div>

    {#if showAttendance}
      <AttendancePanel
        events={[event]}
        {players}
        {teamName}
        selectedEventId={event.id}
        showEventPicker={false}
        showEventSummary={false}
        onParticipationLoaded={(records) => (participationRecords = records)}
      />
    {/if}
    {#if showDuty}
      <DutyPanel
        occurrenceId={event.id}
        {players}
        {teamName}
        {showDriving}
        {showRefereeJury}
        {participationRecords}
      />
    {/if}
  </div>
</div>

<style>
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
    max-width: 40rem;
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

  button {
    border: 0;
    border-radius: 0.55rem;
    cursor: pointer;
    font: inherit;
    font-weight: 800;
    min-height: 2.7rem;
    padding: 0.65rem 0.9rem;
  }

  button:focus-visible {
    outline: 3px solid rgba(213, 99, 62, 0.25);
    outline-offset: 2px;
  }

  .close-button {
    background: transparent;
    border: 1px solid var(--line);
    color: var(--ink);
    flex: 0 0 auto;
  }
</style>
