<script>
  /** @typedef {import('./$types').PageData} PageData */
  /** @type {PageData} */
  export let data;

  /** @param {string} associationId */
  const playerName = (associationId) =>
    data.players.find((player) => player.associationId === associationId)?.name ?? associationId;

  /** @param {{ percentage?: number }} metric */
  const percentage = (metric) => (metric.percentage === undefined ? '—' : `${metric.percentage}%`);

  /** @param {string | undefined} date */
  const formatDate = (date) =>
    date
      ? new Intl.DateTimeFormat('en', {
          day: 'numeric',
          month: 'short',
          timeZone: 'UTC',
          year: 'numeric',
        }).format(new Date(`${date}T00:00:00.000Z`))
      : '—';

  /** @param {Date | string} timestamp */
  const formatTimestamp = (timestamp) =>
    new Intl.DateTimeFormat('en', {
      dateStyle: 'medium',
      timeStyle: 'short',
    }).format(new Date(timestamp));
</script>

<svelte:head>
  <title>History · Team</title>
  <meta name="description" content="Participation and duty history" />
</svelte:head>

<section class="intro">
  <p class="eyebrow">Current season · recorded activity</p>
  <h1>History</h1>
  <p class="lede">Participation percentages and completed duties for the active team roster.</p>
</section>

<section class="panel" aria-labelledby="summary-heading">
  <div class="panel-heading">
    <div>
      <p class="eyebrow">Team balance</p>
      <h2 id="summary-heading">Player summary</h2>
    </div>
  </div>

  {#if data.players.length === 0}
    <div class="empty-state">
      <h3>No history recorded yet</h3>
      <p>Store attendance or complete a duty to start building the current-season summary.</p>
    </div>
  {:else}
    <div class="table-wrap">
      <table>
        <thead>
          <tr>
            <th>Player</th>
            <th>Games</th>
            <th>Training</th>
            <th>Duties completed</th>
          </tr>
        </thead>
        <tbody>
          {#each data.players as player (player.associationId)}
            <tr>
              <th scope="row">{player.name}</th>
              <td
                >{percentage(player.games)}
                <span>{player.games.present}/{player.games.recorded}</span></td
              >
              <td
                >{percentage(player.trainings)}
                <span>{player.trainings.present}/{player.trainings.recorded}</span></td
              >
              <td>{player.dutiesCompleted}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
    <p class="hint">Percentages use recorded participation as the denominator.</p>
  {/if}
</section>

<section class="panel" aria-labelledby="entries-heading">
  <div class="panel-heading">
    <div>
      <p class="eyebrow">Attendance log</p>
      <h2 id="entries-heading">Participation entries</h2>
    </div>
  </div>

  {#if data.entries.length === 0}
    <div class="empty-state compact">
      <h3>No participation entries</h3>
      <p>Attendance records will appear here after they are stored from Team.</p>
    </div>
  {:else}
    <div class="entry-list">
      {#each data.entries as entry (`${entry.eventId}-${entry.playerAssociationId}`)}
        <article class="entry-row">
          <div>
            <strong>{formatDate(entry.date)}</strong>
            <span>{entry.occurrenceType === 'game' ? 'Game' : 'Training'}</span>
          </div>
          <div>
            <strong>{playerName(entry.playerAssociationId)}</strong>
            <span>{entry.eventLabel}</span>
          </div>
          <strong class:absent={entry.status === 'absent'}>{entry.status}</strong>
        </article>
      {/each}
    </div>
  {/if}
</section>

<section class="panel" aria-labelledby="audit-heading">
  <div class="panel-heading">
    <div>
      <p class="eyebrow">Traceability</p>
      <h2 id="audit-heading">Recent audit entries</h2>
    </div>
  </div>

  {#if data.auditEntries.length === 0}
    <div class="empty-state compact">
      <h3>No audit entries</h3>
      <p>Import and coordinator changes will be recorded here.</p>
    </div>
  {:else}
    <div class="entry-list">
      {#each data.auditEntries as entry (entry.id)}
        <article class="entry-row audit-row">
          <div>
            <strong>{entry.action}</strong>
            <span>{formatTimestamp(entry.occurredAt)}</span>
          </div>
          <div>
            <strong>{entry.entityType}</strong>
            <span>{entry.entityId}</span>
          </div>
          <span class="audit-details">{JSON.stringify(entry.metadata)}</span>
        </article>
      {/each}
    </div>
  {/if}
</section>

<style>
  .table-wrap {
    overflow-x: auto;
  }

  table {
    border-collapse: collapse;
    min-width: 38rem;
    width: 100%;
  }

  th,
  td {
    border-bottom: 1px solid var(--line);
    padding: 0.8rem;
    text-align: left;
  }

  thead th {
    color: var(--muted);
    font-size: 0.72rem;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }

  tbody th {
    font-size: 0.95rem;
  }

  td span,
  .hint {
    color: var(--muted);
    font-size: 0.82rem;
  }

  .hint {
    margin: 1rem 0 0;
  }

  .entry-list {
    display: grid;
    gap: 0.75rem;
  }

  .entry-row {
    align-items: center;
    border-top: 1px solid var(--line);
    display: grid;
    gap: 1rem;
    grid-template-columns: 8rem 1fr auto;
    padding-top: 0.75rem;
  }

  .entry-row:first-child {
    border-top: 0;
    padding-top: 0;
  }

  .entry-row div {
    display: grid;
    gap: 0.2rem;
  }

  .entry-row span {
    color: var(--muted);
    font-size: 0.84rem;
  }

  .entry-row > strong:last-child {
    color: #397044;
    font-size: 0.82rem;
    text-transform: capitalize;
  }

  .entry-row > strong.absent {
    color: var(--accent-dark);
  }

  .audit-details {
    color: var(--muted);
    font-size: 0.76rem;
    max-width: 18rem;
    overflow-wrap: anywhere;
  }

  .compact {
    padding: 1.25rem;
  }

  @media (max-width: 36rem) {
    .entry-row {
      grid-template-columns: 1fr auto;
    }

    .entry-row div:nth-child(2) {
      grid-column: 1 / -1;
      grid-row: 2;
    }
  }
</style>
