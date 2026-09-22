<script>
  import { invalidateAll } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { normalizeTeamName } from '$lib/domain/team-name';
  import { deriveSeasonHalf } from '$lib/domain/season-half';

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
    date: '',
    isHome: 'home',
    locationName: '',
    opponentName: '',
    startTime: '',
    travelMinutes: '0',
  };
  let savingGame = false;
  let showGameModal = false;
  let showTrainingModal = false;
  let showOneOffTrainingModal = false;
  let gameMessage = '';
  let gameError = '';
  let lifecycleAction = '';
  let lifecycleMessage = '';
  let lifecycleError = '';
  let rescheduleEventId = '';
  let rescheduleForm = {
    arrivalBufferMinutes: '30',
    date: '',
    locationName: '',
    startTime: '',
    travelMinutes: '0',
  };
  let trainingLifecycleAction = '';
  let trainingLifecycleMessage = '';
  let trainingLifecycleError = '';
  let trainingRescheduleEventId = '';
  let trainingRescheduleForm = {
    date: '',
    durationMinutes: '90',
    locationName: '',
    startTime: '',
  };

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

  const updateGameTravelTime = () => {
    const location = data.locations.find((candidate) => candidate.name === gameForm.locationName);

    if (location) {
      gameForm = { ...gameForm, travelMinutes: String(location.travelMinutes) };
    }
  };

  const updateRescheduleTravelTime = () => {
    const location = data.locations.find(
      (candidate) => candidate.name === rescheduleForm.locationName,
    );

    if (location) {
      rescheduleForm = { ...rescheduleForm, travelMinutes: String(location.travelMinutes) };
    }
  };

  const saveGame = async () => {
    savingGame = true;
    gameMessage = '';
    gameError = '';

    try {
      const knownLocation = data.locations.find(
        (location) => location.name === gameForm.locationName,
      );

      if (!knownLocation) {
        await postJson(
          '/api/locations',
          {
            name: gameForm.locationName,
            travelMinutes: Number(gameForm.travelMinutes),
          },
          'The venue could not be stored.',
        );
      }

      const isHome = gameForm.isHome === 'home';
      const opponentName = normalizeTeamName(gameForm.opponentName);
      const fixture = await postJson(
        '/api/games/fixtures',
        {
          awayTeamName: isHome ? opponentName : data.teamName,
          homeTeamName: isHome ? data.teamName : opponentName,
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
      gameMessage = 'Game added to the events.';
      gameForm = { ...gameForm, date: '', opponentName: '', startTime: '' };
      showGameModal = false;
      await invalidateAll();
    } catch (error) {
      gameError = error instanceof Error ? error.message : 'The game could not be stored.';
    } finally {
      savingGame = false;
    }
  };

  /** @param {string} eventId */
  const cancelGame = async (eventId) => {
    lifecycleAction = eventId;
    lifecycleMessage = '';
    lifecycleError = '';

    try {
      await postJson(
        `/api/game-occurrences/${eventId}/cancel`,
        {},
        'The game could not be cancelled.',
      );
      lifecycleMessage = 'Game cancelled.';
      await invalidateAll();
    } catch (error) {
      lifecycleError = error instanceof Error ? error.message : 'The game could not be cancelled.';
    } finally {
      lifecycleAction = '';
    }
  };

  /**
   * @param {{ id: string; arrivalBufferMinutes: number; date: string; locationName: string; startTime: string; travelMinutes: number }} event
   */
  const openReschedule = (event) => {
    rescheduleEventId = event.id;
    rescheduleForm = {
      arrivalBufferMinutes: String(event.arrivalBufferMinutes),
      date: event.date,
      locationName: event.locationName,
      startTime: event.startTime,
      travelMinutes: String(event.travelMinutes),
    };
    lifecycleMessage = '';
    lifecycleError = '';
  };

  const rescheduleGame = async () => {
    lifecycleAction = rescheduleEventId;
    lifecycleMessage = '';
    lifecycleError = '';

    try {
      await postJson(
        `/api/game-occurrences/${rescheduleEventId}/reschedule`,
        {
          arrivalBufferMinutes: Number(rescheduleForm.arrivalBufferMinutes),
          date: rescheduleForm.date,
          locationName: rescheduleForm.locationName,
          startTime: rescheduleForm.startTime,
          travelMinutes: Number(rescheduleForm.travelMinutes),
        },
        'The game could not be rescheduled.',
      );
      rescheduleEventId = '';
      lifecycleMessage = 'Game rescheduled.';
      await invalidateAll();
    } catch (error) {
      lifecycleError =
        error instanceof Error ? error.message : 'The game could not be rescheduled.';
    } finally {
      lifecycleAction = '';
    }
  };

  /** @param {string | undefined} occurrenceId */
  const cancelTraining = async (occurrenceId) => {
    if (!occurrenceId) {
      return;
    }

    trainingLifecycleAction = occurrenceId;
    trainingLifecycleMessage = '';
    trainingLifecycleError = '';

    try {
      await postJson(
        `/api/training-occurrences/${occurrenceId}/cancel`,
        {},
        'The training occurrence could not be cancelled.',
      );
      trainingLifecycleMessage = 'Training occurrence cancelled.';
      await invalidateAll();
    } catch (error) {
      trainingLifecycleError =
        error instanceof Error ? error.message : 'The training occurrence could not be cancelled.';
    } finally {
      trainingLifecycleAction = '';
    }
  };

  /**
   * @param {{ occurrenceId?: string; date: string; durationMinutes: number; locationName: string; startTime: string }} event
   */
  const openTrainingReschedule = (event) => {
    if (!event.occurrenceId) {
      return;
    }

    trainingRescheduleEventId = event.occurrenceId;
    trainingRescheduleForm = {
      date: event.date,
      durationMinutes: String(event.durationMinutes),
      locationName: event.locationName,
      startTime: event.startTime,
    };
    trainingLifecycleMessage = '';
    trainingLifecycleError = '';
  };

  const rescheduleTraining = async () => {
    trainingLifecycleAction = trainingRescheduleEventId;
    trainingLifecycleMessage = '';
    trainingLifecycleError = '';

    try {
      await postJson(
        `/api/training-occurrences/${trainingRescheduleEventId}/reschedule`,
        {
          date: trainingRescheduleForm.date,
          durationMinutes: Number(trainingRescheduleForm.durationMinutes),
          locationName: trainingRescheduleForm.locationName,
          startTime: trainingRescheduleForm.startTime,
        },
        'The training occurrence could not be rescheduled.',
      );
      trainingRescheduleEventId = '';
      trainingLifecycleMessage = 'Training occurrence rescheduled.';
      await invalidateAll();
    } catch (error) {
      trainingLifecycleError =
        error instanceof Error
          ? error.message
          : 'The training occurrence could not be rescheduled.';
    } finally {
      trainingLifecycleAction = '';
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
      oneOffTrainingMessage = 'One-off training added to the events.';
      oneOffTrainingForm = { ...oneOffTrainingForm, date: '' };
      showOneOffTrainingModal = false;
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
      trainingMessage = 'Training series added to the events.';
      trainingForm = { ...trainingForm, endDate: '', startDate: '' };
      showTrainingModal = false;
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
  <title>Events · Team</title>
  <meta name="description" content="Games, training, and duties for the team" />
</svelte:head>

<section class="intro">
  <p class="eyebrow">Shared timeline</p>
  <h1>Events</h1>
  <p class="lede">Games and training occurrences in chronological order.</p>
</section>

<section class="panel" aria-labelledby="events-heading">
  <div class="panel-heading">
    <div>
      <p class="eyebrow">{data.events.length} events</p>
      <h2 id="events-heading">All events</h2>
    </div>
    <div class="action-row">
      <button type="button" on:click={() => (showGameModal = true)}>Add game</button>
      <button type="button" on:click={() => (showTrainingModal = true)}>Add training series</button>
      <button type="button" on:click={() => (showOneOffTrainingModal = true)}>
        Add training
      </button>
    </div>
  </div>

  {#if data.events.length === 0}
    <div class="empty-state">
      <h3>No events configured yet</h3>
      <p>Add a game or training event to start the timeline.</p>
    </div>
  {:else}
    <div class="program-list">
      {#each data.events as event (event.id)}
        <article class="program-row" id={`event-${event.id}`}>
          <div class="program-date">
            <strong>{formatDate(event.date)}</strong>
            <span>{event.startTime}</span>
          </div>
          <div class="program-event">
            {#if event.type === 'game'}
              <p class="event-kind">
                Game · {event.status}{#if deriveSeasonHalf(event.date, data.season.startingYear)}
                  · {deriveSeasonHalf(event.date, data.season.startingYear)}{/if}
              </p>
              <h2>{event.homeTeamName} <span aria-hidden="true">vs</span> {event.awayTeamName}</h2>
              <p>{event.locationName} · Suggested departure {event.suggestedDepartureTime}</p>
              {#if event.status === 'scheduled'}
                <div class="event-actions">
                  <button
                    disabled={lifecycleAction === event.id}
                    on:click={() => cancelGame(event.id)}
                    type="button"
                  >
                    {lifecycleAction === event.id ? 'Saving…' : 'Cancel game'}
                  </button>
                  <button on:click={() => openReschedule(event)} type="button">Reschedule</button>
                </div>
                {#if rescheduleEventId === event.id}
                  <form class="game-form lifecycle-form" on:submit|preventDefault={rescheduleGame}>
                    <label>
                      Date
                      <input bind:value={rescheduleForm.date} required type="date" />
                    </label>
                    <label>
                      Start time
                      <input bind:value={rescheduleForm.startTime} required type="time" />
                    </label>
                    <label>
                      Location
                      <select
                        bind:value={rescheduleForm.locationName}
                        on:change={updateRescheduleTravelTime}
                        required
                      >
                        <option disabled value="">Choose a location</option>
                        {#each data.locations as location (location.name)}
                          <option value={location.name}>{location.name}</option>
                        {/each}
                      </select>
                    </label>
                    <label>
                      Travel minutes
                      <input
                        bind:value={rescheduleForm.travelMinutes}
                        min="0"
                        required
                        type="number"
                      />
                    </label>
                    <label>
                      Arrival buffer
                      <input
                        bind:value={rescheduleForm.arrivalBufferMinutes}
                        min="0"
                        required
                        type="number"
                      />
                    </label>
                    <button disabled={lifecycleAction === event.id} type="submit">
                      {lifecycleAction === event.id ? 'Saving…' : 'Save reschedule'}
                    </button>
                  </form>
                {/if}
              {/if}
            {:else}
              <p class="event-kind">Training · {event.status}</p>
              <h2>Team training</h2>
              <p>{event.locationName} · {event.durationMinutes} minutes</p>
              {#if event.status === 'scheduled' && event.occurrenceId}
                <div class="event-actions">
                  <button
                    disabled={trainingLifecycleAction === event.occurrenceId}
                    on:click={() => cancelTraining(event.occurrenceId)}
                    type="button"
                  >
                    {trainingLifecycleAction === event.occurrenceId ? 'Saving…' : 'Cancel training'}
                  </button>
                  <button on:click={() => openTrainingReschedule(event)} type="button">
                    Reschedule
                  </button>
                </div>
                {#if trainingRescheduleEventId === event.occurrenceId}
                  <form
                    class="game-form lifecycle-form"
                    on:submit|preventDefault={rescheduleTraining}
                  >
                    <label>
                      Date
                      <input bind:value={trainingRescheduleForm.date} required type="date" />
                    </label>
                    <label>
                      Start time
                      <input bind:value={trainingRescheduleForm.startTime} required type="time" />
                    </label>
                    <label>
                      Duration minutes
                      <input
                        bind:value={trainingRescheduleForm.durationMinutes}
                        min="1"
                        required
                        type="number"
                      />
                    </label>
                    <label>
                      Location
                      <select bind:value={trainingRescheduleForm.locationName} required>
                        <option disabled value="">Choose a location</option>
                        {#each data.locations as location (location.name)}
                          <option value={location.name}>{location.name}</option>
                        {/each}
                      </select>
                    </label>
                    <button disabled={trainingLifecycleAction === event.occurrenceId} type="submit">
                      {trainingLifecycleAction === event.occurrenceId
                        ? 'Saving…'
                        : 'Save reschedule'}
                    </button>
                  </form>
                {/if}
              {/if}
            {/if}
          </div>
        </article>
      {/each}
    </div>
    {#if trainingLifecycleMessage}<p class="form-message" role="status">
        {trainingLifecycleMessage}
      </p>{/if}
    {#if trainingLifecycleError}<p class="form-error" role="alert">{trainingLifecycleError}</p>{/if}
  {/if}
</section>

{#if showGameModal}
  <div class="modal-backdrop" role="presentation">
    <div class="panel modal" aria-labelledby="manual-game-heading" role="dialog" aria-modal="true">
      <div class="modal-header">
        <div>
          <p class="eyebrow">New event</p>
          <h2 id="manual-game-heading">Add a game</h2>
        </div>
        <button class="close-button" type="button" on:click={() => (showGameModal = false)}>
          Close
        </button>
      </div>

      <form class="game-form" on:submit|preventDefault={saveGame}>
        <label>
          Venue
          <select bind:value={gameForm.isHome}>
            <option value="home">Home game</option>
            <option value="away">Away game</option>
          </select>
        </label>
        <label>
          Opponent
          <input
            bind:value={gameForm.opponentName}
            placeholder="e.g. Haarlem Ballers U15-4"
            required
          />
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
          Venue
          <input
            bind:value={gameForm.locationName}
            list="event-locations"
            on:change={updateGameTravelTime}
            placeholder="Select or add a venue"
            required
          />
          <datalist id="event-locations">
            {#each data.locations as location (location.name)}
              <option value={location.name}>{location.travelMinutes} minutes travel</option>
            {/each}
          </datalist>
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
        {#if data.locations.length === 0}
          Add a location in <a href={resolve('/admin')}>Admin</a> before adding events.
        {:else}
          Departure is calculated from travel time and arrival buffer.
        {/if}
      </p>
      {#if gameMessage}<p class="form-message" role="status">{gameMessage}</p>{/if}
      {#if gameError}<p class="form-error" role="alert">{gameError}</p>{/if}
      {#if lifecycleMessage}<p class="form-message" role="status">{lifecycleMessage}</p>{/if}
      {#if lifecycleError}<p class="form-error" role="alert">{lifecycleError}</p>{/if}
    </div>
  </div>
{/if}

{#if showTrainingModal}
  <div class="modal-backdrop" role="presentation">
    <div
      class="panel modal"
      aria-labelledby="training-series-heading"
      role="dialog"
      aria-modal="true"
    >
      <div class="modal-header">
        <div>
          <p class="eyebrow">Recurring event</p>
          <h2 id="training-series-heading">Add a training series</h2>
        </div>
        <button class="close-button" type="button" on:click={() => (showTrainingModal = false)}>
          Close
        </button>
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
          <select bind:value={trainingForm.locationName} required>
            <option disabled value="">Choose a location</option>
            {#each data.locations as location (location.name)}
              <option value={location.name}>{location.name}</option>
            {/each}
          </select>
        </label>
        <button disabled={savingTraining} type="submit">
          {savingTraining ? 'Saving…' : 'Add training series'}
        </button>
      </form>
      <p class="form-hint">
        {#if data.locations.length === 0}
          Add a location in <a href={resolve('/admin')}>Admin</a> before adding training.
        {:else}
          Locations are managed in <a href={resolve('/admin')}>Admin</a>.
        {/if}
      </p>
      {#if trainingMessage}<p class="form-message" role="status">{trainingMessage}</p>{/if}
      {#if trainingError}<p class="form-error" role="alert">{trainingError}</p>{/if}
    </div>
  </div>
{/if}

{#if showOneOffTrainingModal}
  <div class="modal-backdrop" role="presentation">
    <div
      class="panel modal"
      aria-labelledby="one-off-training-heading"
      role="dialog"
      aria-modal="true"
    >
      <div class="modal-header">
        <div>
          <p class="eyebrow">Single occurrence</p>
          <h2 id="one-off-training-heading">Add one-off training</h2>
        </div>
        <button
          class="close-button"
          type="button"
          on:click={() => (showOneOffTrainingModal = false)}
        >
          Close
        </button>
      </div>

      <form class="game-form" on:submit|preventDefault={saveOneOffTraining}>
        <label>
          Date
          <input bind:value={oneOffTrainingForm.date} required type="date" />
        </label>
        <label>
          Start time
          <input
            bind:value={oneOffTrainingForm.startTime}
            pattern="\d{2}:\d{2}"
            required
            type="time"
          />
        </label>
        <label>
          Duration minutes
          <input bind:value={oneOffTrainingForm.durationMinutes} min="1" required type="number" />
        </label>
        <label>
          Location
          <select bind:value={oneOffTrainingForm.locationName} required>
            <option disabled value="">Choose a location</option>
            {#each data.locations as location (location.name)}
              <option value={location.name}>{location.name}</option>
            {/each}
          </select>
        </label>
        <button disabled={savingOneOffTraining} type="submit">
          {savingOneOffTraining ? 'Saving…' : 'Add one-off training'}
        </button>
      </form>
      <p class="form-hint">
        {#if data.locations.length === 0}
          Add a location in <a href={resolve('/admin')}>Admin</a> before adding training.
        {:else}
          Locations are managed in <a href={resolve('/admin')}>Admin</a>.
        {/if}
      </p>
      {#if oneOffTrainingMessage}<p class="form-message" role="status">
          {oneOffTrainingMessage}
        </p>{/if}
      {#if oneOffTrainingError}<p class="form-error" role="alert">{oneOffTrainingError}</p>{/if}
    </div>
  </div>
{/if}

<style>
  .action-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .action-row button,
  .close-button {
    background: var(--accent);
    border: 0;
    border-radius: 0.55rem;
    color: white;
    cursor: pointer;
    font: inherit;
    font-size: 0.82rem;
    font-weight: 800;
    min-height: 2.4rem;
    padding: 0.55rem 0.75rem;
  }

  .action-row button:hover,
  .close-button:hover {
    background: var(--accent-dark);
  }

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
    max-width: 48rem;
    width: 100%;
  }

  .modal-header {
    align-items: start;
    display: flex;
    gap: 1rem;
    justify-content: space-between;
    margin-bottom: 1.5rem;
  }

  .modal-header h2 {
    font-size: clamp(1.5rem, 4vw, 2rem);
    letter-spacing: -0.05em;
    margin: 0;
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

  .event-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin-top: 0.75rem;
  }

  .event-actions button {
    background: transparent;
    border: 1px solid var(--line);
    border-radius: 0.45rem;
    color: var(--ink);
    cursor: pointer;
    font: inherit;
    font-size: 0.82rem;
    padding: 0.45rem 0.65rem;
  }

  .event-actions button:hover:not(:disabled) {
    border-color: var(--accent);
    color: var(--accent-dark);
  }

  .event-actions button:disabled {
    cursor: wait;
    opacity: 0.55;
  }

  .lifecycle-form {
    margin-top: 0.75rem;
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
