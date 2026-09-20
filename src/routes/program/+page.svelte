<script>
  import { invalidateAll } from '$app/navigation';

  export let data;

  /** @param {string} date */
  /** @param {string} date */
  const formatDate = (date) =>
    new Intl.DateTimeFormat('en', {
      day: 'numeric',
      month: 'short',
      timeZone: 'UTC',
      weekday: 'short',
      year: 'numeric',
    }).format(new Date(`${date}T00:00:00.000Z`));

  let gameForm = {
    arrivalBufferMinutes: '30',
    awayTeamName: '',
    date: '',
    homeTeamName: 'U16-1',
    locationName: '',
    startTime: '',
    travelMinutes: '0',
  };
  let savingGame = false;
  let gameMessage = '';
  let gameError = '';

  let trainingForm = {
    durationMinutes: '90',
    endDate: '',
    locationName: '',
    startDate: '',
    startTime: '18:00',
    weekday: '2',
  };
  let savingTraining = false;
  let trainingMessage = '';
  let trainingError = '';

  let oneOffTrainingForm = {
    date: '',
    durationMinutes: '90',
    locationName: '',
    startTime: '18:00',
  };
  let savingOneOffTraining = false;
  let oneOffTrainingMessage = '';
  let oneOffTrainingError = '';

  /** @param {string} path @param {unknown} payload @param {string} fallbackMessage */
  const postJson = async (path, payload, fallbackMessage) => {
    const response = await fetch(path, {
      body: JSON.stringify(payload),
      headers: { 'content-type': 'application/json' },
      method: 'POST',
    });
    const body = await response.json().catch(() => ({}));

    if (!response.ok) {
      throw new Error(body.error ?? fallbackMessage);
    }

    return body;
  };

  const saveGame = async () => {
    savingGame = true;
    gameMessage = '';
    gameError = '';

    try {
      const fixture = await postJson(
        '/api/games/fixtures',
        {
          awayTeamName: gameForm.awayTeamName,
          homeTeamName: gameForm.homeTeamName,
        },
        'The game could not be stored.',
      );
      await postJson(
        '/api/game-occurrences',
        {
          arrivalBufferMinutes: Number(gameForm.arrivalBufferMinutes),
          date: gameForm.date,
          fixtureId: fixture.id,
          locationName: gameForm.locationName,
          startTime: gameForm.startTime,
          travelMinutes: Number(gameForm.travelMinutes),
        },
        'The game could not be stored.',
      );
      gameMessage = 'Game added to the program.';
      gameForm = { ...gameForm, awayTeamName: '', date: '', startTime: '' };
      await invalidateAll();
    } catch (error) {
      gameError = error instanceof Error ? error.message : 'The game could not be stored.';
    } finally {
      savingGame = false;
    }
  };

  const saveOneOffTraining = async () => {
    savingOneOffTraining = true;
    oneOffTrainingMessage = '';
    oneOffTrainingError = '';

    try {
      await postJson(
        '/api/training-occurrences',
        {
          date: oneOffTrainingForm.date,
          durationMinutes: Number(oneOffTrainingForm.durationMinutes),
          locationName: oneOffTrainingForm.locationName,
          startTime: oneOffTrainingForm.startTime,
        },
        'The training occurrence could not be stored.',
      );
      oneOffTrainingMessage = 'One-off training added to the program.';
      oneOffTrainingForm = { ...oneOffTrainingForm, date: '' };
      await invalidateAll();
    } catch (error) {
      oneOffTrainingError =
        error instanceof Error ? error.message : 'The training occurrence could not be stored.';
    } finally {
      savingOneOffTraining = false;
    }
  };

  const saveTraining = async () => {
    savingTraining = true;
    trainingMessage = '';
    trainingError = '';

    try {
      await postJson(
        '/api/training-series',
        {
          durationMinutes: Number(trainingForm.durationMinutes),
          endDate: trainingForm.endDate,
          locationName: trainingForm.locationName,
          startDate: trainingForm.startDate,
          startTime: trainingForm.startTime,
          weekday: Number(trainingForm.weekday),
        },
        'The training series could not be stored.',
      );
      trainingMessage = 'Training series added to the program.';
      trainingForm = { ...trainingForm, endDate: '', startDate: '' };
      await invalidateAll();
    } catch (error) {
      trainingError =
        error instanceof Error ? error.message : 'The training series could not be stored.';
    } finally {
      savingTraining = false;
    }
  };
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

<section class="panel manual-game-panel" aria-labelledby="manual-game-heading">
  <div class="panel-heading">
    <div>
      <p class="eyebrow">Coordinator entry</p>
      <h2 id="manual-game-heading">Add a game</h2>
    </div>
  </div>

  <form class="game-form" on:submit|preventDefault={saveGame}>
    <label>
      Home team
      <input bind:value={gameForm.homeTeamName} required />
    </label>
    <label>
      Away team
      <input bind:value={gameForm.awayTeamName} required />
    </label>
    <label>
      Date
      <input bind:value={gameForm.date} required type="date" />
    </label>
    <label>
      Start time
      <input bind:value={gameForm.startTime} pattern="\d{2}:\d{2}" required type="time" />
    </label>
    <label>
      Location
      <input bind:value={gameForm.locationName} required />
    </label>
    <label>
      Travel minutes
      <input bind:value={gameForm.travelMinutes} min="0" required type="number" />
    </label>
    <label>
      Arrival buffer
      <input bind:value={gameForm.arrivalBufferMinutes} min="0" required type="number" />
    </label>
    <button disabled={savingGame} type="submit">{savingGame ? 'Saving…' : 'Add game'}</button>
  </form>
  <p class="form-hint">
    The location must already exist in Settings. Departure is calculated from travel time and
    arrival buffer.
  </p>
  {#if gameMessage}<p class="form-message" role="status">{gameMessage}</p>{/if}
  {#if gameError}<p class="form-error" role="alert">{gameError}</p>{/if}
</section>

<section class="panel manual-game-panel" aria-labelledby="training-series-heading">
  <div class="panel-heading">
    <div>
      <p class="eyebrow">Recurring event</p>
      <h2 id="training-series-heading">Add a training series</h2>
    </div>
  </div>

  <form class="game-form" on:submit|preventDefault={saveTraining}>
    <label>
      Weekday
      <select bind:value={trainingForm.weekday}>
        <option value="1">Monday</option>
        <option value="2">Tuesday</option>
        <option value="3">Wednesday</option>
        <option value="4">Thursday</option>
        <option value="5">Friday</option>
        <option value="6">Saturday</option>
        <option value="7">Sunday</option>
      </select>
    </label>
    <label>
      Start date
      <input bind:value={trainingForm.startDate} required type="date" />
    </label>
    <label>
      End date
      <input bind:value={trainingForm.endDate} required type="date" />
    </label>
    <label>
      Start time
      <input bind:value={trainingForm.startTime} pattern="\d{2}:\d{2}" required type="time" />
    </label>
    <label>
      Duration minutes
      <input bind:value={trainingForm.durationMinutes} min="1" required type="number" />
    </label>
    <label>
      Location
      <input bind:value={trainingForm.locationName} required />
    </label>
    <button disabled={savingTraining} type="submit">
      {savingTraining ? 'Saving…' : 'Add training series'}
    </button>
  </form>
  <p class="form-hint">The location must already exist in Settings.</p>
  {#if trainingMessage}<p class="form-message" role="status">{trainingMessage}</p>{/if}
  {#if trainingError}<p class="form-error" role="alert">{trainingError}</p>{/if}
</section>

<section class="panel manual-game-panel" aria-labelledby="one-off-training-heading">
  <div class="panel-heading">
    <div>
      <p class="eyebrow">Single occurrence</p>
      <h2 id="one-off-training-heading">Add one-off training</h2>
    </div>
  </div>

  <form class="game-form" on:submit|preventDefault={saveOneOffTraining}>
    <label>
      Date
      <input bind:value={oneOffTrainingForm.date} required type="date" />
    </label>
    <label>
      Start time
      <input bind:value={oneOffTrainingForm.startTime} pattern="\d{2}:\d{2}" required type="time" />
    </label>
    <label>
      Duration minutes
      <input bind:value={oneOffTrainingForm.durationMinutes} min="1" required type="number" />
    </label>
    <label>
      Location
      <input bind:value={oneOffTrainingForm.locationName} required />
    </label>
    <button disabled={savingOneOffTraining} type="submit">
      {savingOneOffTraining ? 'Saving…' : 'Add one-off training'}
    </button>
  </form>
  <p class="form-hint">The location must already exist in Settings.</p>
  {#if oneOffTrainingMessage}<p class="form-message" role="status">{oneOffTrainingMessage}</p>{/if}
  {#if oneOffTrainingError}<p class="form-error" role="alert">{oneOffTrainingError}</p>{/if}
</section>

<style>
  .manual-game-panel {
    margin-top: 1rem;
  }

  .game-form {
    display: grid;
    gap: 0.8rem;
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }

  .game-form label {
    color: var(--muted);
    display: grid;
    font-size: 0.78rem;
    font-weight: 800;
    gap: 0.35rem;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .game-form input,
  .game-form select {
    background: #fffdf8;
    border: 1px solid var(--line);
    border-radius: 0.55rem;
    box-sizing: border-box;
    color: var(--ink);
    font: inherit;
    font-size: 0.95rem;
    font-weight: 500;
    min-height: 2.7rem;
    padding: 0.55rem 0.65rem;
    width: 100%;
  }

  .game-form select {
    appearance: none;
  }

  .game-form button {
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

  .game-form button:hover:not(:disabled) {
    background: var(--accent-dark);
  }

  .game-form button:disabled {
    cursor: wait;
    opacity: 0.55;
  }

  .form-hint,
  .form-message,
  .form-error {
    font-size: 0.88rem;
    margin: 1rem 0 0;
  }

  .form-hint {
    color: var(--muted);
  }

  .form-message {
    color: #397044;
  }

  .form-error {
    color: var(--accent-dark);
  }

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

  @media (max-width: 48rem) {
    .game-form {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }

  @media (max-width: 36rem) {
    .game-form {
      grid-template-columns: 1fr;
    }

    .program-row {
      grid-template-columns: 1fr;
      gap: 0.5rem;
    }
  }
</style>
