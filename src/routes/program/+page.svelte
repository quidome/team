<script>
  export let data;

  /** @param {string} date */
  const formatDate = (date) =>
    new Intl.DateTimeFormat('en', {
      day: 'numeric',
      month: 'short',
      timeZone: 'UTC',
      weekday: 'short',
      year: 'numeric',
    }).format(new Date(`${date}T00:00:00.000Z`));
</script>

<svelte:head>
  <title>Program · Team</title>
  <meta name="description" content="Games and training sessions for the team program" />
</svelte:head>

<section class="intro">
  <p class="eyebrow">Shared schedule</p>
  <h1>Program</h1>
  <p class="lede">Games and training occurrences in chronological order.</p>
</section>

<section class="panel" aria-labelledby="program-heading">
  <div class="panel-heading">
    <div>
      <p class="eyebrow">{data.events.length} events</p>
      <h2 id="program-heading">All events</h2>
    </div>
  </div>

  {#if data.events.length === 0}
    <div class="empty-state">
      <h3>No events configured yet</h3>
      <p>Use the program configuration APIs to add the first game or training series.</p>
    </div>
  {:else}
    <div class="program-list">
      {#each data.events as event (event.id)}
        <article class="program-row">
          <div class="program-date">
            <strong>{formatDate(event.date)}</strong>
            <span>{event.startTime}</span>
          </div>
          <div class="program-event">
            {#if event.type === 'game'}
              <p class="event-kind">Game · {event.status}</p>
              <h2>{event.homeTeamName} <span aria-hidden="true">vs</span> {event.awayTeamName}</h2>
              <p>{event.locationName} · Suggested departure {event.suggestedDepartureTime}</p>
            {:else}
              <p class="event-kind">Training</p>
              <h2>Team training</h2>
              <p>{event.locationName} · {event.durationMinutes} minutes</p>
            {/if}
          </div>
        </article>
      {/each}
    </div>
  {/if}
</section>

<style>
  .program-list {
    display: grid;
    gap: 0.75rem;
  }

  .program-row {
    align-items: start;
    border-top: 1px solid var(--line);
    display: grid;
    gap: 1rem;
    grid-template-columns: minmax(9rem, 0.3fr) 1fr;
    padding: 1rem 0;
  }

  .program-row:first-child {
    border-top: 0;
    padding-top: 0;
  }

  .program-date {
    color: var(--muted);
    display: grid;
    gap: 0.25rem;
  }

  .program-date strong {
    color: var(--ink);
  }

  .program-event h2 {
    font-size: 1.1rem;
    margin: 0.2rem 0;
  }

  .program-event p:last-child {
    color: var(--muted);
    margin: 0;
  }

  @media (max-width: 36rem) {
    .program-row {
      grid-template-columns: 1fr;
      gap: 0.5rem;
    }
  }
</style>
