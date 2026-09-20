<script>
  import AttendancePanel from '$lib/components/AttendancePanel.svelte';

  export let data;

  /** @param {string} date */
  const formatDate = (date) =>
    new Intl.DateTimeFormat('en', {
      day: 'numeric',
      month: 'long',
      timeZone: 'UTC',
      weekday: 'long',
      year: 'numeric',
    }).format(new Date(`${date}T00:00:00.000Z`));

  const eventTitle =
    data.event.type === 'training'
      ? 'Team training'
      : `${data.event.homeTeamName} vs ${data.event.awayTeamName}`;
  const eventSummary =
    data.event.type === 'training'
      ? `${data.event.locationName} · ${data.event.durationMinutes} minutes`
      : `${data.event.locationName} · Leave by ${data.event.suggestedDepartureTime}`;
</script>

<svelte:head>
  <title>{eventTitle} · Team</title>
  <meta name="description" content="Event attendance for the team" />
</svelte:head>

<section class="intro">
  <p class="eyebrow">{data.event.type === 'training' ? 'Training' : 'Game'}</p>
  <h1>{eventTitle}</h1>
  <p class="lede">{formatDate(data.event.date)} · {data.event.startTime} · {eventSummary}</p>
</section>

<AttendancePanel
  events={[data.event]}
  players={data.players}
  selectedEventId={data.event.id}
  showEventPicker={false}
  showEventSummary={false}
  teamName={data.teamName}
/>
