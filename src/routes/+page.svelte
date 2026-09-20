<script>
  import { resolve } from '$app/paths';

  export let data;

  /** @param {string} date */
  const formatDate = (date) =>
    new Intl.DateTimeFormat('en', {
      day: 'numeric',
      month: 'short',
      timeZone: 'UTC',
      weekday: 'short',
    }).format(new Date(`${date}T00:00:00.000Z`));
  const today = new Intl.DateTimeFormat('en-CA', { timeZone: 'UTC' }).format(new Date());
  const scheduledEvents = data.events.filter((event) => event.status === 'scheduled');
  const previousEvents = scheduledEvents.filter((event) => event.date < today).slice(-2);
  const todayEvents = scheduledEvents.filter((event) => event.date === today);
  const nextEvents = scheduledEvents.filter((event) => event.date > today).slice(0, 3);
  const visibleEvents = [...previousEvents, ...todayEvents, ...nextEvents];
  const nextEventId = todayEvents.length === 0 ? nextEvents[0]?.id : undefined;
  /** @param {import('$lib/application/program/read-program').ProgramEvent} event */
  const isAttendanceEvent = (event) =>
    event.type === 'training' ||
    event.homeTeamName === data.teamName ||
    event.awayTeamName === data.teamName;
  /** @param {import('$lib/application/program/read-program').ProgramEvent} event */
  const eventHref = (event) =>
    isAttendanceEvent(event)
      ? resolve('/events/[eventId]', { eventId: event.id })
      : `${resolve('/events')}#event-${encodeURIComponent(event.id)}`;
</script>

<svelte:head>
  <title>Home · Team</title>
  <meta name="description" content="Coordinator overview for the U16-1 team" />
</svelte:head>

<section class="intro">
  <p class="eyebrow">Coordinator workspace</p>
  <h1>Home</h1>
  <p class="lede">A quick view of the team’s upcoming events.</p>
</section>

<section class="panel" aria-labelledby="upcoming-heading">
  <div class="panel-heading">
    <div>
      <p class="eyebrow">{visibleEvents.length} events</p>
      <h2 id="upcoming-heading">Events</h2>
    </div>
    <a class="text-link" href={resolve('/events')}>View events <span aria-hidden="true">→</span></a>
  </div>

  {#if visibleEvents.length === 0}
    <div class="empty-state">
      <h3>No scheduled events yet</h3>
      <p>Add games or recurring training sessions to see them here.</p>
    </div>
  {:else}
    <div class="event-list">
      {#each visibleEvents as event, index (event.id)}
        <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -->
        <a class="event-card-link" href={eventHref(event)}>
          <article
            class:past-event={index < previousEvents.length}
            class:today-event={event.date === today}
            class:next-event={event.id === nextEventId}
            class="event-card"
          >
            <div class="event-date">
              {#if event.date === today}
                <span class="event-badge">Today</span>
              {:else if event.id === nextEventId}
                <span class="event-badge">Next</span>
              {/if}
              <span>{formatDate(event.date)}</span>
              <strong>{event.startTime}</strong>
            </div>
            <div class="event-details">
              {#if event.type === 'game'}
                <p class="event-kind">
                  Game · {event.status}{#if event.seasonHalf}
                    · {event.seasonHalf}{/if}
                </p>
                <h3>
                  {event.homeTeamName} <span aria-hidden="true">vs</span>
                  {event.awayTeamName}
                </h3>
                <p>{event.locationName} · Leave by {event.suggestedDepartureTime}</p>
              {:else}
                <p class="event-kind">Training</p>
                <h3>Team training</h3>
                <p>{event.locationName} · {event.durationMinutes} minutes</p>
              {/if}
            </div>
          </article>
        </a>
      {/each}
    </div>
  {/if}
</section>

<style>
  :global(.event-card-link) {
    color: inherit;
    display: block;
    text-decoration: none;
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

  :global(.past-event) {
    opacity: 0.7;
  }

  :global(.today-event) {
    background: #fff4ef;
    border-color: var(--accent);
    box-shadow: 0 0 0 2px rgba(213, 99, 62, 0.12);
    opacity: 1;
  }

  :global(.next-event) {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px rgba(213, 99, 62, 0.08);
    opacity: 1;
  }

  .event-badge {
    color: var(--accent-dark);
    font-size: 0.68rem;
    font-weight: 800;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }
</style>
