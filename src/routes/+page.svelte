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
  const upcomingEvents = data.events.filter(
    (event) => event.date >= today && (event.type !== 'game' || event.status === 'scheduled'),
  );
</script>

<svelte:head>
  <title>Home · Team</title>
  <meta name="description" content="Coordinator overview for the U16-1 team" />
</svelte:head>

<section class="intro">
  <p class="eyebrow">Coordinator workspace</p>
  <h1>Home</h1>
  <p class="lede">A quick view of the shared team program.</p>
</section>

<section class="panel" aria-labelledby="upcoming-heading">
  <div class="panel-heading">
    <div>
      <p class="eyebrow">Program</p>
      <h2 id="upcoming-heading">Upcoming events</h2>
    </div>
    <a class="text-link" href={resolve('/program')}
      >View program <span aria-hidden="true">→</span></a
    >
  </div>

  {#if upcomingEvents.length === 0}
    <div class="empty-state">
      <h3>No events configured yet</h3>
      <p>Add games or recurring training sessions to see them here.</p>
    </div>
  {:else}
    <div class="event-list">
      {#each upcomingEvents.slice(0, 6) as event (event.id)}
        <article class="event-card">
          <div class="event-date">
            <span>{formatDate(event.date)}</span>
            <strong>{event.startTime}</strong>
          </div>
          <div class="event-details">
            {#if event.type === 'game'}
              <p class="event-kind">Game · {event.status}</p>
              <h3>{event.homeTeamName} <span aria-hidden="true">vs</span> {event.awayTeamName}</h3>
              <p>{event.locationName} · Leave by {event.suggestedDepartureTime}</p>
            {:else}
              <p class="event-kind">Training</p>
              <h3>Team training</h3>
              <p>{event.locationName} · {event.durationMinutes} minutes</p>
            {/if}
          </div>
        </article>
      {/each}
    </div>
  {/if}
</section>
