<script>
  import EventModal from '$lib/components/EventModal.svelte';

  /** @typedef {import('$lib/application/program/read-program').ProgramGameEvent} ProgramGameEvent */
  /** @typedef {import('./$types').PageData} PageData */

  /** @type {PageData} */
  export let data;

  /** @type {ProgramGameEvent | undefined} */
  let activeEvent = undefined;

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
      <p class="eyebrow">{data.events.length} games</p>
      <h2 id="game-heading">Game duties and transport</h2>
    </div>
  </div>

  {#if data.events.length === 0}
    <div class="empty-state">
      <h3>No games configured yet</h3>
      <p>Games will appear here when they are added to the Program.</p>
    </div>
  {:else}
    <div class="event-list">
      {#each data.events as event (event.id)}
        <button class="event-card-link" type="button" on:click={() => (activeEvent = event)}>
          <article class="event-card">
            <div class="event-date">
              <span>{formatDate(event.date)}</span>
              <strong>{event.startTime}</strong>
            </div>
            <div class="event-details">
              <p class="event-kind">
                {event.status === 'scheduled' ? 'Scheduled game' : 'Cancelled game'}
              </p>
              <h3>{event.homeTeamName} <span aria-hidden="true">vs</span> {event.awayTeamName}</h3>
              <p>{event.locationName} · Leave by {event.suggestedDepartureTime}</p>
            </div>
          </article>
        </button>
      {/each}
    </div>
  {/if}
</section>

{#if activeEvent}
  <EventModal
    event={activeEvent}
    players={data.players}
    teamName={data.teamName}
    seasonStartingYear={data.season.startingYear}
    onClose={() => (activeEvent = undefined)}
  />
{/if}

<style>
  :global(.event-card-link) {
    background: none;
    border: 0;
    color: inherit;
    cursor: pointer;
    display: block;
    font: inherit;
    padding: 0;
    text-align: left;
    text-decoration: none;
    width: 100%;
  }

  :global(.event-card-link:hover .event-card),
  :global(.event-card-link:focus-visible .event-card) {
    border-color: var(--accent);
    transform: translateY(-1px);
  }

  :global(.event-card-link:focus-visible) {
    border-radius: 0.9rem;
    outline: 3px solid rgba(213, 99, 62, 0.25);
    outline-offset: 3px;
  }
</style>
