<script>
  import { resolve } from '$app/paths';
  import { invalidateAll } from '$app/navigation';

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
  let newTaskTitle = '';
  let newTaskDueDate = '';
  let taskMessage = '';
  let taskError = '';

  const createTask = async () => {
    taskMessage = '';
    taskError = '';

    try {
      const response = await fetch('/api/tasks', {
        body: JSON.stringify({ dueDate: newTaskDueDate || undefined, title: newTaskTitle }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      });

      if (!response.ok) {
        const body = await response.json().catch(() => ({}));
        throw new Error(body.error ?? 'The reminder could not be stored.');
      }

      newTaskTitle = '';
      newTaskDueDate = '';
      taskMessage = 'Reminder added.';
      await invalidateAll();
    } catch (error) {
      taskError = error instanceof Error ? error.message : 'The reminder could not be stored.';
    }
  };

  /** @param {import('$lib/application/tasks/task-repository').Task} task */
  const completeTask = async (task) => {
    taskMessage = '';
    taskError = '';

    try {
      const response = await fetch(`/api/tasks/${encodeURIComponent(task.id)}/status`, {
        body: JSON.stringify({ status: 'completed' }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      });

      if (!response.ok) {
        throw new Error('The reminder could not be completed.');
      }

      await invalidateAll();
    } catch (error) {
      taskError = error instanceof Error ? error.message : 'The reminder could not be completed.';
    }
  };
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

<section class="panel tasks-panel" aria-labelledby="tasks-heading">
  <div class="panel-heading">
    <div>
      <p class="eyebrow">Coordination queue</p>
      <h2 id="tasks-heading">Tasks and reminders</h2>
    </div>
    <a class="text-link" href={resolve('/messages')}
      >Open messages <span aria-hidden="true">→</span></a
    >
  </div>

  {#if data.tasks.length === 0}
    <div class="empty-state">
      <h3>No tasks yet</h3>
      <p>Generated coordination tasks and manual reminders will appear here.</p>
    </div>
  {:else}
    <div class="task-list">
      {#each data.tasks as task (task.id)}
        <article class:task-completed={task.status === 'completed'} class="task-row">
          <div>
            <p class="task-kind">{task.source === 'generated' ? 'Generated' : 'Reminder'}</p>
            <h3>{task.title}</h3>
            {#if task.description}<p>{task.description}</p>{/if}
            {#if task.dueDate}<small>Due {formatDate(task.dueDate)}</small>{/if}
          </div>
          {#if task.status === 'completed'}
            <span class="task-status">Completed</span>
          {:else if task.source === 'manual'}
            <button class="complete-button" type="button" on:click={() => completeTask(task)}
              >Complete</button
            >
          {:else}
            <a class="text-link" href={resolve('/messages')}>Prepare draft</a>
          {/if}
        </article>
      {/each}
    </div>
  {/if}

  <form class="task-form" on:submit|preventDefault={createTask}>
    <label>
      Add a reminder
      <input bind:value={newTaskTitle} placeholder="e.g. Confirm rides with parents" required />
    </label>
    <label>
      Due date <span class="optional">optional</span>
      <input bind:value={newTaskDueDate} type="date" />
    </label>
    <button type="submit">Add reminder</button>
  </form>
  {#if taskMessage}<p class="form-message" role="status">{taskMessage}</p>{/if}
  {#if taskError}<p class="form-error" role="alert">{taskError}</p>{/if}
</section>

<style>
  .tasks-panel {
    margin-top: 1rem;
  }

  .task-list {
    display: grid;
    gap: 0.65rem;
  }

  .task-row {
    align-items: center;
    background: #faf7f0;
    border: 1px solid #ebe4d8;
    border-radius: 0.9rem;
    display: flex;
    gap: 1rem;
    justify-content: space-between;
    padding: 1rem;
  }

  .task-row h3 {
    font-size: 1rem;
    margin: 0.2rem 0;
  }

  .task-row p,
  .task-row small {
    color: var(--muted);
    font-size: 0.84rem;
    margin: 0.25rem 0 0;
  }

  .task-kind {
    color: var(--accent-dark) !important;
    font-size: 0.7rem !important;
    font-weight: 800;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .task-completed {
    opacity: 0.65;
  }

  .task-status {
    color: #397044;
    font-size: 0.84rem;
    font-weight: 700;
  }

  .complete-button {
    background: transparent;
    border: 1px solid var(--line);
    border-radius: 999px;
    color: var(--accent-dark);
    cursor: pointer;
    font: inherit;
    font-size: 0.82rem;
    font-weight: 700;
    padding: 0.55rem 0.8rem;
  }

  .complete-button:hover {
    background: #f4eee4;
  }

  .task-form {
    align-items: end;
    border-top: 1px solid var(--line);
    display: grid;
    gap: 0.75rem;
    grid-template-columns: 1fr 12rem auto;
    margin-top: 1.25rem;
    padding-top: 1.25rem;
  }

  .task-form label {
    color: var(--muted);
    display: grid;
    font-size: 0.78rem;
    font-weight: 800;
    gap: 0.35rem;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .task-form input {
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

  .task-form button {
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

  .task-form button:hover {
    background: var(--accent-dark);
  }

  .optional {
    font-size: 0.68rem;
    font-weight: 500;
    letter-spacing: 0;
    text-transform: none;
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

  @media (max-width: 48rem) {
    .task-form {
      grid-template-columns: 1fr 1fr;
    }

    .task-form button {
      grid-column: 1 / -1;
    }
  }

  @media (max-width: 36rem) {
    .task-row {
      align-items: start;
      flex-direction: column;
    }

    .task-form {
      grid-template-columns: 1fr;
    }
  }
</style>
