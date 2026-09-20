<script>
  import { generatePollDraft } from '$lib/application/messages/poll-drafts';

  /** @typedef {import('./$types').PageData} PageData */

  /** @type {PageData} */
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

  let selectedEventId = data.events[0]?.id ?? '';
  let departureLocation = '';
  let departureTimeOverride = '';
  let templates = { ...data.templates };
  let draftText = data.events[0]
    ? generatePollDraft(data.events[0], { teamName: data.teamName, templates }).text
    : '';
  let copyMessage = '';

  $: selectedEvent = data.events.find((event) => event.id === selectedEventId);

  const buildDraft = (event = selectedEvent) => {
    if (!event) return '';

    return generatePollDraft(event, {
      departureLocation,
      departureTimeOverride,
      teamName: data.teamName,
      templates,
    }).text;
  };

  const regenerateDraft = () => {
    draftText = buildDraft();
    copyMessage = '';
  };

  /** @param {Event} event */
  const handleGameChange = (event) => {
    const select = /** @type {HTMLSelectElement} */ (event.currentTarget);
    selectedEventId = select.value;
    departureLocation = '';
    departureTimeOverride = '';
    draftText = buildDraft(data.events.find((candidate) => candidate.id === selectedEventId));
    copyMessage = '';
  };

  const copyDraft = async () => {
    try {
      await navigator.clipboard.writeText(draftText);
      copyMessage = 'Draft copied. Paste it into WhatsApp manually.';
    } catch {
      copyMessage = 'Copy was not available. Select the draft and copy it manually.';
    }
  };
</script>

<svelte:head>
  <title>Messages · Team</title>
  <meta name="description" content="Editable Dutch WhatsApp poll drafts" />
</svelte:head>

<section class="intro">
  <p class="eyebrow">Manual copy · Dutch output</p>
  <h1>Messages</h1>
  <p class="lede">
    Prepare editable attendance and transport polls, then copy them into WhatsApp yourself.
  </p>
</section>

<section class="panel" aria-labelledby="draft-heading">
  <div class="panel-heading">
    <div>
      <p class="eyebrow">No sending or duty polls</p>
      <h2 id="draft-heading">Game poll draft</h2>
    </div>
  </div>

  {#if data.events.length === 0}
    <div class="empty-state">
      <h3>No own games configured yet</h3>
      <p>Games involving {data.teamName} will appear here when they are added to the Program.</p>
    </div>
  {:else if selectedEvent}
    <label class="event-picker">
      Game
      <select value={selectedEventId} on:change={handleGameChange}>
        {#each data.events as event (event.id)}
          <option value={event.id}
            >{formatDate(event.date)} · {event.homeTeamName} vs {event.awayTeamName}</option
          >
        {/each}
      </select>
    </label>

    <div class="game-summary">
      <div>
        <p class="event-kind">
          {selectedEvent.homeTeamName === data.teamName ? 'Home-game poll' : 'Away-game poll'}
        </p>
        <h3>{selectedEvent.homeTeamName} vs {selectedEvent.awayTeamName}</h3>
        <p>{selectedEvent.startTime} · {selectedEvent.locationName}</p>
      </div>
      <div class="departure-card">
        <span>Suggested departure</span>
        <strong>{selectedEvent.suggestedDepartureTime}</strong>
      </div>
    </div>

    <div class="override-grid">
      <label>
        Departure location <span class="optional">away games only</span>
        <input bind:value={departureLocation} placeholder="e.g. Clubhouse" />
      </label>
      <label>
        Departure time override <span class="optional">optional</span>
        <input
          bind:value={departureTimeOverride}
          placeholder={selectedEvent.suggestedDepartureTime}
          pattern="\d{2}:\d{2}"
        />
      </label>
      <button class="secondary-button" type="button" on:click={regenerateDraft}
        >Regenerate draft</button
      >
    </div>

    <label class="draft-label">
      Editable Dutch draft
      <textarea bind:value={draftText} rows="11" spellcheck="true"></textarea>
    </label>
    <div class="draft-actions">
      <button type="button" on:click={copyDraft}>Copy draft</button>
      {#if copyMessage}<p role="status">{copyMessage}</p>{/if}
    </div>

    <details class="template-editor">
      <summary>Edit Dutch templates</summary>
      <p class="hint">
        Available placeholders: &#123;&#123;date&#125;&#125;, &#123;&#123;time&#125;&#125;,
        &#123;&#123;team&#125;&#125;, &#123;&#123;opponent&#125;&#125;,
        &#123;&#123;location&#125;&#125;, &#123;&#123;departureTime&#125;&#125;,
        &#123;&#123;departureLocation&#125;&#125;.
      </p>
      <label>
        Home-game template
        <textarea bind:value={templates.homeGame} rows="7"></textarea>
      </label>
      <label>
        Away-game template
        <textarea bind:value={templates.awayGame} rows="9"></textarea>
      </label>
      <button class="secondary-button" type="button" on:click={regenerateDraft}
        >Apply template changes</button
      >
    </details>
  {/if}
</section>

<style>
  .event-picker,
  .override-grid label,
  .draft-label,
  .template-editor label {
    color: var(--muted);
    display: grid;
    font-size: 0.78rem;
    font-weight: 800;
    gap: 0.35rem;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .event-picker {
    max-width: 36rem;
  }

  input,
  select,
  textarea {
    background: #fffdf8;
    border: 1px solid var(--line);
    border-radius: 0.55rem;
    box-sizing: border-box;
    color: var(--ink);
    font: inherit;
    font-size: 0.95rem;
    font-weight: 500;
    letter-spacing: normal;
    padding: 0.65rem;
    text-transform: none;
    width: 100%;
  }

  input,
  select {
    min-height: 2.7rem;
  }

  textarea {
    line-height: 1.5;
    resize: vertical;
  }

  input:focus,
  select:focus,
  textarea:focus,
  button:focus-visible {
    outline: 3px solid rgba(213, 99, 62, 0.25);
    outline-offset: 2px;
  }

  button {
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

  button:hover {
    background: var(--accent-dark);
  }

  .game-summary {
    align-items: end;
    border-bottom: 1px solid var(--line);
    display: flex;
    gap: 1rem;
    justify-content: space-between;
    margin: 1.5rem 0;
    padding-bottom: 1rem;
  }

  .game-summary h3 {
    font-size: 1.35rem;
    letter-spacing: -0.04em;
    margin: 0.2rem 0;
  }

  .game-summary p:last-child {
    color: var(--muted);
    margin: 0;
  }

  .event-kind {
    color: var(--accent-dark);
    font-size: 0.7rem;
    font-weight: 800;
    letter-spacing: 0.08em;
    margin: 0;
    text-transform: uppercase;
  }

  .departure-card {
    background: #faf7f0;
    border: 1px solid #ebe4d8;
    border-radius: 0.9rem;
    display: grid;
    gap: 0.2rem;
    min-width: 11rem;
    padding: 0.8rem 1rem;
  }

  .departure-card span {
    color: var(--muted);
    font-size: 0.78rem;
  }

  .departure-card strong {
    font-size: 1.35rem;
  }

  .override-grid {
    align-items: end;
    display: grid;
    gap: 0.8rem;
    grid-template-columns: 1fr 1fr auto;
    margin-bottom: 1rem;
  }

  .optional {
    font-size: 0.68rem;
    font-weight: 500;
    letter-spacing: 0;
    text-transform: none;
  }

  .secondary-button {
    background: transparent;
    border: 1px solid var(--line);
    color: var(--accent-dark);
  }

  .secondary-button:hover {
    background: #f4eee4;
  }

  .draft-actions {
    align-items: center;
    display: flex;
    gap: 1rem;
    margin-top: 0.8rem;
  }

  .draft-actions p {
    color: #397044;
    font-size: 0.88rem;
    margin: 0;
  }

  .template-editor {
    border-top: 1px solid var(--line);
    display: grid;
    gap: 1rem;
    margin-top: 2rem;
    padding-top: 1rem;
  }

  .template-editor summary {
    color: var(--accent-dark);
    cursor: pointer;
    font-weight: 800;
  }

  .hint {
    color: var(--muted);
    font-size: 0.84rem;
    line-height: 1.45;
    margin: 0;
  }

  @media (max-width: 48rem) {
    .override-grid {
      grid-template-columns: 1fr 1fr;
    }

    .override-grid button {
      grid-column: 1 / -1;
    }
  }

  @media (max-width: 36rem) {
    .game-summary {
      align-items: stretch;
      flex-direction: column;
    }

    .departure-card {
      min-width: 0;
    }

    .override-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
