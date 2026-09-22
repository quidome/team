<script>
  import { invalidateAll } from '$app/navigation';

  export let data;

  /** @type {Record<'games' | 'locations' | 'players', Array<{ key: string; label: string; required: boolean }>>} */
  const fieldsByKind = {
    games: [
      { key: 'homeTeamName', label: 'Home team', required: true },
      { key: 'awayTeamName', label: 'Away team', required: true },
      { key: 'date', label: 'Date', required: true },
      { key: 'startTime', label: 'Start time', required: true },
      { key: 'locationName', label: 'Location', required: true },
      { key: 'travelMinutes', label: 'Travel minutes', required: false },
      { key: 'arrivalBufferMinutes', label: 'Arrival buffer', required: false },
      { key: 'jurySlots', label: 'Jury slots', required: false },
      { key: 'refereeSlots', label: 'Referee slots', required: false },
    ],
    locations: [
      { key: 'name', label: 'Name', required: true },
      { key: 'address', label: 'Address', required: false },
      { key: 'travelMinutes', label: 'Travel minutes', required: false },
    ],
    players: [
      { key: 'firstName', label: 'First name', required: true },
      { key: 'lastName', label: 'Last name', required: false },
      { key: 'birthDate', label: 'Birthdate', required: false },
      { key: 'associationId', label: 'Association ID', required: false },
      { key: 'jerseyNumber', label: 'Jersey number', required: false },
      { key: 'participationType', label: 'Participation type', required: false },
    ],
  };

  /** @type {Record<'games' | 'locations' | 'players', Array<{ key: string; label: string }>>} */
  const previewColumnsByKind = {
    games: [
      { key: 'homeTeamName', label: 'Home' },
      { key: 'awayTeamName', label: 'Away' },
      { key: 'date', label: 'Date' },
      { key: 'startTime', label: 'Time' },
      { key: 'locationName', label: 'Location' },
    ],
    locations: [
      { key: 'name', label: 'Name' },
      { key: 'address', label: 'Address' },
      { key: 'travelMinutes', label: 'Travel minutes' },
    ],
    players: [
      { key: 'firstName', label: 'First name' },
      { key: 'lastName', label: 'Last name' },
      { key: 'birthDate', label: 'Birthdate' },
      { key: 'associationId', label: 'Association ID' },
    ],
  };

  /** @type {Record<'games' | 'locations' | 'players', string>} */
  const importKindLabels = { games: 'games', locations: 'locations', players: 'roster' };

  /** @type {'games' | 'locations' | 'players'} */
  let importKind = 'games';
  $: fields = fieldsByKind[importKind];
  $: previewColumns = previewColumnsByKind[importKind];

  let fileName = '';
  let content = '';
  let encoding = 'text';
  /** @type {string[]} */
  let headers = [];
  /** @type {string[]} */
  let worksheets = [];
  let sheetName = '';
  /** @type {Record<string, string>} */
  let mapping = {};
  /**
   * @type {
   *   | import('$lib/application/imports/game-import').GameImportPreview
   *   | import('$lib/application/imports/player-import').PlayerImportPreview
   *   | import('$lib/application/imports/location-import').LocationImportPreview
   *   | undefined
   * }
   */
  let preview = undefined;
  let loading = false;
  let importing = false;
  let error = '';
  let importMessage = '';
  /** @type {Array<{sourceRow: number, existingOccurrenceId: string, fields: Array<{field: string, existingValue: string | number, importedValue: string | number}>}>} */
  let conflicts = [];
  /** @type {Record<string, Record<string, string>>} */
  let conflictChoices = {};
  /** @type {Array<{sourceRow: number, candidates: Array<{id: string, firstName: string, lastName?: string, birthDate?: string, associationId?: string}>}>} */
  let playerDuplicates = [];
  /** @type {Record<string, { choice: string; matchedPlayerId: string }>} */
  let duplicateChoices = {};
  let settingsForm = {
    primaryTeamName: data.coordinatorSettings?.primaryTeamName ?? '',
    seasonStartingYear: data.coordinatorSettings?.seasonStartingYear
      ? String(data.coordinatorSettings.seasonStartingYear)
      : '',
  };
  let savingSettings = false;
  let settingsMessage = '';
  let settingsError = '';
  let seasonForm = { startingYear: '' };
  let savingSeason = false;
  let seasonMessage = '';
  let seasonError = '';
  let teamForm = { name: '' };
  let savingTeam = false;
  let teamMessage = '';
  let teamError = '';
  let locationForm = {
    address: '',
    name: '',
    travelMinutes: '0',
  };
  let savingLocation = false;
  let locationMessage = '';
  let locationError = '';
  let adminMessage = '';
  let adminError = '';
  /** @type {{ type: 'season' | 'team' | 'location'; label: string; currentName?: string; currentStartingYear?: number } | undefined} */
  let editTarget;
  let editForm = { address: '', name: '', startingYear: '', travelMinutes: '0' };
  let editSaving = false;

  /** @param {string} path @param {string} label */
  /** @param {'season' | 'team' | 'location'} type @param {any} item */
  const openEdit = (type, item) => {
    editTarget = {
      currentName: type === 'season' ? undefined : item.name,
      currentStartingYear: type === 'season' ? item.startingYear : undefined,
      label:
        type === 'season'
          ? `${item.startingYear}–${item.endingYear} season`
          : `${item.name} ${type}`,
      type,
    };
    editForm = {
      address: type === 'location' ? (item.address ?? '') : '',
      name: type === 'season' ? '' : item.name,
      startingYear: type === 'season' ? String(item.startingYear) : '',
      travelMinutes: type === 'location' ? String(item.travelMinutes) : '0',
    };
    adminError = '';
    adminMessage = '';
  };

  const saveEdit = async () => {
    if (!editTarget) return;

    editSaving = true;
    adminError = '';
    adminMessage = '';

    try {
      const payload =
        editTarget.type === 'season'
          ? {
              currentStartingYear: editTarget.currentStartingYear,
              startingYear: Number(editForm.startingYear),
            }
          : editTarget.type === 'team'
            ? { currentName: editTarget.currentName, name: editForm.name }
            : {
                ...(editForm.address.trim() ? { address: editForm.address.trim() } : {}),
                currentName: editTarget.currentName,
                name: editForm.name,
                travelMinutes: Number(editForm.travelMinutes),
              };
      const response = await fetch(`/api/${editTarget.type}s`, {
        body: JSON.stringify(payload),
        headers: { 'content-type': 'application/json' },
        method: 'PUT',
      });
      const body = await response.json().catch(() => ({}));

      if (!response.ok) {
        throw new Error(body.error ?? `${editTarget.label} could not be updated.`);
      }

      adminMessage = `${editTarget.label} updated.`;
      editTarget = undefined;
      await invalidateAll();
    } catch (caught) {
      adminError = caught instanceof Error ? caught.message : 'The item could not be updated.';
    } finally {
      editSaving = false;
    }
  };

  /** @param {string} path @param {string} label */
  const deleteItem = async (path, label) => {
    if (!window.confirm(`Delete ${label}?`)) return;

    adminMessage = '';
    adminError = '';

    try {
      const response = await fetch(path, { method: 'DELETE' });
      const body = await response.json().catch(() => ({}));

      if (!response.ok) {
        throw new Error(body.error ?? `${label} could not be deleted.`);
      }

      adminMessage = `${label} deleted.`;
      await invalidateAll();
    } catch (caught) {
      adminError = caught instanceof Error ? caught.message : `${label} could not be deleted.`;
    }
  };

  const saveLocation = async () => {
    savingLocation = true;
    locationMessage = '';
    locationError = '';

    try {
      const response = await fetch('/api/locations', {
        body: JSON.stringify({
          ...(locationForm.address.trim() ? { address: locationForm.address.trim() } : {}),
          name: locationForm.name,
          travelMinutes: Number(locationForm.travelMinutes),
        }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      });
      const body = await response.json().catch(() => ({}));

      if (!response.ok) {
        throw new Error(body.error ?? 'The location could not be stored.');
      }

      locationForm = { address: '', name: '', travelMinutes: '0' };
      locationMessage = `${body.name} is available for games and training.`;
      await invalidateAll();
    } catch (caught) {
      locationError =
        caught instanceof Error ? caught.message : 'The location could not be stored.';
    } finally {
      savingLocation = false;
    }
  };

  const saveSettings = async () => {
    savingSettings = true;
    settingsMessage = '';
    settingsError = '';

    try {
      const response = await fetch('/api/settings', {
        body: JSON.stringify({
          primaryTeamName: settingsForm.primaryTeamName,
          seasonStartingYear: Number(settingsForm.seasonStartingYear),
        }),
        headers: { 'content-type': 'application/json' },
        method: 'PUT',
      });
      const body = await response.json().catch(() => ({}));

      if (!response.ok) {
        throw new Error(body.error ?? 'The coordinator settings could not be saved.');
      }

      settingsMessage = `${body.primaryTeamName} · ${body.seasonStartingYear}–${body.seasonStartingYear + 1} is the active workspace context.`;
      await invalidateAll();
    } catch (caught) {
      settingsError =
        caught instanceof Error ? caught.message : 'The coordinator settings could not be saved.';
    } finally {
      savingSettings = false;
    }
  };

  const saveSeason = async () => {
    savingSeason = true;
    seasonMessage = '';
    seasonError = '';

    try {
      const response = await fetch('/api/seasons', {
        body: JSON.stringify({ startingYear: Number(seasonForm.startingYear) }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      });
      const body = await response.json().catch(() => ({}));

      if (!response.ok) {
        throw new Error(body.error ?? 'The season could not be stored.');
      }

      seasonForm = { startingYear: '' };
      seasonMessage = `${body.startingYear}–${body.endingYear} is available as a team context.`;
      await invalidateAll();
    } catch (caught) {
      seasonError = caught instanceof Error ? caught.message : 'The season could not be stored.';
    } finally {
      savingSeason = false;
    }
  };

  const saveTeam = async () => {
    savingTeam = true;
    teamMessage = '';
    teamError = '';

    try {
      const response = await fetch('/api/teams', {
        body: JSON.stringify({ name: teamForm.name }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      });
      const body = await response.json().catch(() => ({}));

      if (!response.ok) {
        throw new Error(body.error ?? 'The team could not be stored.');
      }

      teamForm = { name: '' };
      teamMessage = `${body.name} is available as a team context.`;
      await invalidateAll();
    } catch (caught) {
      teamError = caught instanceof Error ? caught.message : 'The team could not be stored.';
    } finally {
      savingTeam = false;
    }
  };

  /** @param {string} value */
  const readHeaders = (value) => {
    const firstLine = value.split(/\r?\n/, 1)[0] ?? '';
    const found = [];
    let current = '';
    let quoted = false;

    for (let index = 0; index < firstLine.length; index += 1) {
      const character = firstLine[index];
      const next = firstLine[index + 1];

      if (character === '"') {
        if (quoted && next === '"') {
          current += '"';
          index += 1;
        } else {
          quoted = !quoted;
        }
      } else if (character === ',' && !quoted) {
        found.push(current.trim());
        current = '';
      } else {
        current += character;
      }
    }

    found.push(current.trim());
    return found.filter(Boolean);
  };

  /** @type {Record<'games' | 'locations' | 'players', Record<string, string[]>>} */
  const aliasesByKind = {
    games: {
      awayTeamName: ['away', 'away team', 'visitor'],
      arrivalBufferMinutes: ['arrival buffer', 'buffer'],
      date: ['date', 'datum'],
      homeTeamName: ['home', 'home team'],
      jurySlots: ['jury', 'jury duty'],
      locationName: ['location', 'venue', 'plaats'],
      refereeSlots: ['referee', 'ref'],
      startTime: ['start time', 'time', 'tijd'],
      travelMinutes: ['travel', 'travel minutes', 'reistijd'],
    },
    locations: {
      address: ['address', 'adres'],
      name: ['name', 'location', 'venue', 'plaats'],
      travelMinutes: ['travel', 'travel minutes', 'reistijd'],
    },
    players: {
      associationId: ['association id', 'association', 'bondsnummer', 'lidnummer'],
      birthDate: ['birthdate', 'birthday', 'born', 'geboortedatum'],
      firstName: ['first name', 'firstname', 'voornaam'],
      jerseyNumber: ['jersey', 'jersey number', 'number', 'rugnummer'],
      lastName: ['last name', 'lastname', 'surname', 'achternaam'],
      participationType: ['participation', 'participation type', 'type'],
    },
  };

  /** @param {string[]} availableHeaders */
  const guessMapping = (availableHeaders) => {
    /** @type {Record<string, string>} */
    const next = {};
    const aliases = aliasesByKind[importKind];

    for (const field of fields) {
      const match = availableHeaders.find((header) =>
        aliases[field.key].includes(header.toLowerCase()),
      );

      if (match) next[field.key] = match;
    }

    return next;
  };

  const resetImportState = () => {
    fileName = '';
    content = '';
    encoding = 'text';
    headers = [];
    worksheets = [];
    sheetName = '';
    mapping = {};
    preview = undefined;
    loading = false;
    importing = false;
    error = '';
    importMessage = '';
    conflicts = [];
    conflictChoices = {};
    playerDuplicates = [];
    duplicateChoices = {};
  };

  /** @param {Event} event */
  const handleImportKindChange = (event) => {
    const select = /** @type {HTMLSelectElement} */ (event.currentTarget);
    importKind = /** @type {'games' | 'locations' | 'players'} */ (select.value);
    resetImportState();
  };

  const loadHeaders = async () => {
    const response = await fetch(`/api/imports/${importKind}/preview`, {
      body: JSON.stringify({
        content,
        encoding,
        fileName,
        mapping: {},
        ...(sheetName ? { sheetName } : {}),
      }),
      headers: { 'content-type': 'application/json' },
      method: 'POST',
    });
    const body = await response.json();

    if (!response.ok) {
      throw new Error(body.error ?? 'The import file could not be read.');
    }

    headers = body.headers;
    worksheets = body.worksheets ?? [];
    if (worksheets.length > 0 && !worksheets.includes(sheetName)) {
      sheetName = worksheets[0];
    }
    mapping = guessMapping(headers);
  };

  const handleWorksheetChange = async () => {
    headers = [];
    mapping = {};
    preview = undefined;
    error = '';

    try {
      await loadHeaders();
    } catch (caught) {
      error = caught instanceof Error ? caught.message : 'The worksheet could not be read.';
    }
  };

  /** @param {Event} event */
  const handleFile = async (event) => {
    const input = /** @type {HTMLInputElement} */ (event.currentTarget);
    const file = input.files?.[0];

    if (!file) return;

    fileName = file.name;
    worksheets = [];
    sheetName = '';
    error = '';
    if (/\.csv$/i.test(file.name)) {
      encoding = 'text';
      content = await file.text();
    } else {
      encoding = 'base64';
      const bytes = new Uint8Array(await file.arrayBuffer());
      let binary = '';

      for (const byte of bytes) {
        binary += String.fromCharCode(byte);
      }

      content = btoa(binary);
    }
    headers = encoding === 'text' ? readHeaders(content) : [];
    mapping = encoding === 'text' ? guessMapping(headers) : {};
    if (encoding === 'base64') {
      try {
        await loadHeaders();
      } catch (caught) {
        error = caught instanceof Error ? caught.message : 'The import file could not be read.';
      }
    }
    preview = undefined;
    importMessage = '';
    conflicts = [];
    conflictChoices = {};
    playerDuplicates = [];
    duplicateChoices = {};
  };

  const previewImport = async () => {
    loading = true;
    error = '';

    try {
      const response = await fetch(`/api/imports/${importKind}/preview`, {
        body: JSON.stringify({
          content,
          encoding,
          fileName,
          mapping,
          ...(sheetName ? { sheetName } : {}),
        }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      });
      const body = await response.json();

      if (!response.ok) {
        throw new Error(body.error ?? 'The import preview could not be generated.');
      }

      preview = body;
    } catch (caught) {
      error =
        caught instanceof Error ? caught.message : 'The import preview could not be generated.';
    } finally {
      loading = false;
    }
  };

  $: allConflictChoicesSelected = conflicts.every((conflict) =>
    conflict.fields.every((field) => Boolean(conflictChoices[conflict.sourceRow]?.[field.field])),
  );

  $: allPlayerDuplicatesResolved = playerDuplicates.every((duplicate) => {
    const choice = duplicateChoices[duplicate.sourceRow];

    if (!choice?.choice) return false;
    if (choice.choice === 'overwrite' && duplicate.candidates.length > 1) {
      return Boolean(choice.matchedPlayerId);
    }

    return true;
  });

  /** @param {string} key */
  const conflictFieldLabel = (key) =>
    fieldsByKind.games.find((field) => field.key === key)?.label ?? key;

  /** @param {number} sourceRow */
  const conflictGameSummary = (sourceRow) => {
    const records = /** @type {any[]} */ (preview?.records ?? []);
    const record = records.find((candidate) => candidate.sourceRow === sourceRow);

    return record
      ? `${record.homeTeamName} vs ${record.awayTeamName} · ${record.date} · ${record.startTime}`
      : undefined;
  };

  /** @param {any} body */
  const describeImportResult = (body) => {
    if (importKind === 'games') {
      /** @type {Array<{merged: boolean}>} */
      const importedGames = body.imported;
      const mergedCount = importedGames.filter((game) => game.merged).length;
      return `${body.imported.length} games imported; ${body.duplicates.length} duplicates skipped${mergedCount ? `; ${mergedCount} conflicts merged.` : '.'}`;
    }

    if (importKind === 'players') {
      /** @type {Array<{updated: boolean}>} */
      const importedPlayers = body.imported;
      const updatedCount = importedPlayers.filter((player) => player.updated).length;
      const skippedCount = body.skipped.length;
      return `${body.imported.length} players imported (${updatedCount} updated)${skippedCount ? `; ${skippedCount} skipped` : ''}${body.failed.length ? `; ${body.failed.length} failed.` : '.'}`;
    }

    return `${body.imported.length} locations added, ${body.updated.length} updated, ${body.unchanged.length} unchanged${body.failed.length ? `; ${body.failed.length} failed.` : '.'}`;
  };

  const runImport = async () => {
    importing = true;
    error = '';
    importMessage = '';

    try {
      const response = await fetch(`/api/imports/${importKind}`, {
        body: JSON.stringify({
          content,
          encoding,
          fileName,
          mapping,
          ...(sheetName ? { sheetName } : {}),
          ...(importKind === 'games'
            ? {
                resolutions: conflicts.map((conflict) => ({
                  fields: conflictChoices[conflict.sourceRow],
                  sourceRow: conflict.sourceRow,
                })),
              }
            : {}),
          ...(importKind === 'players'
            ? {
                resolutions: playerDuplicates.map((duplicate) => ({
                  choice: duplicateChoices[duplicate.sourceRow]?.choice,
                  ...(duplicateChoices[duplicate.sourceRow]?.matchedPlayerId
                    ? { matchedPlayerId: duplicateChoices[duplicate.sourceRow].matchedPlayerId }
                    : {}),
                  sourceRow: duplicate.sourceRow,
                })),
              }
            : {}),
          sourceName: fileName,
        }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      });
      const body = await response.json();

      if (importKind === 'games' && response.status === 409 && body.error === 'import_conflicts') {
        conflicts = body.conflicts;
        conflictChoices = Object.fromEntries(
          conflicts.map((conflict) => [
            conflict.sourceRow,
            Object.fromEntries(conflict.fields.map((field) => [field.field, ''])),
          ]),
        );
        return;
      }

      if (
        importKind === 'players' &&
        response.status === 409 &&
        body.error === 'import_duplicates'
      ) {
        playerDuplicates = body.duplicates;
        duplicateChoices = Object.fromEntries(
          playerDuplicates.map((duplicate) => [
            duplicate.sourceRow,
            {
              choice: '',
              matchedPlayerId: duplicate.candidates.length === 1 ? duplicate.candidates[0].id : '',
            },
          ]),
        );
        return;
      }

      if (!response.ok) {
        throw new Error(body.error ?? `The ${importKindLabels[importKind]} could not be imported.`);
      }

      conflicts = [];
      conflictChoices = {};
      playerDuplicates = [];
      duplicateChoices = {};
      importMessage = describeImportResult(body);
    } catch (caught) {
      error =
        caught instanceof Error
          ? caught.message
          : `The ${importKindLabels[importKind]} could not be imported.`;
    } finally {
      importing = false;
    }
  };
</script>

<svelte:head>
  <title>Admin · Team</title>
  <meta name="description" content="Seasons, teams, locations, and imports" />
</svelte:head>

<section class="intro">
  <p class="eyebrow">Shared configuration</p>
  <h1>Admin</h1>
  <p class="lede">Set up seasons, teams, locations, and imports for the shared team workspace.</p>
  {#if adminMessage}<p class="form-message" role="status">{adminMessage}</p>{/if}
  {#if adminError}<p class="form-error" role="alert">{adminError}</p>{/if}
</section>

{#if editTarget}
  <div class="modal-backdrop" role="presentation">
    <div class="panel modal" role="dialog" aria-modal="true" aria-labelledby="edit-heading">
      <div class="modal-header">
        <div>
          <p class="eyebrow">Edit configuration</p>
          <h2 id="edit-heading">{editTarget.label}</h2>
        </div>
        <button class="close-button" type="button" on:click={() => (editTarget = undefined)}>
          Close
        </button>
      </div>
      <form class="configuration-form edit-form" on:submit|preventDefault={saveEdit}>
        {#if editTarget.type === 'season'}
          <label>
            Starting year
            <input bind:value={editForm.startingYear} min="2000" required type="number" />
          </label>
        {:else}
          <label>
            Name
            <input bind:value={editForm.name} required />
          </label>
          {#if editTarget.type === 'location'}
            <label>
              Address <span class="optional">optional</span>
              <input bind:value={editForm.address} autocomplete="street-address" />
            </label>
            <label>
              Travel minutes
              <input bind:value={editForm.travelMinutes} min="0" required type="number" />
            </label>
          {/if}
        {/if}
        <button disabled={editSaving} type="submit">
          {editSaving ? 'Saving…' : 'Save changes'}
        </button>
      </form>
    </div>
  </div>
{/if}

<section class="panel" aria-labelledby="settings-heading">
  <div class="panel-heading">
    <div>
      <p class="eyebrow">Active workspace context</p>
      <h2 id="settings-heading">Coordinator settings</h2>
    </div>
  </div>

  <form class="configuration-form" on:submit|preventDefault={saveSettings}>
    <label>
      Primary team
      <select bind:value={settingsForm.primaryTeamName} required>
        <option value="" disabled>Choose a team</option>
        {#each data.teams as team (team.name)}
          <option value={team.name}>{team.name}</option>
        {/each}
      </select>
    </label>
    <label>
      Current season
      <select bind:value={settingsForm.seasonStartingYear} required>
        <option value="" disabled>Choose a season</option>
        {#each data.seasons as season (season.startingYear)}
          <option value={String(season.startingYear)}
            >{season.startingYear}–{season.endingYear}</option
          >
        {/each}
      </select>
    </label>
    <button disabled={savingSettings} type="submit">
      {savingSettings ? 'Saving…' : 'Save settings'}
    </button>
  </form>
  <p class="form-hint">
    Sets the team and season shown across Home, Events, Roster, Messages, and History.
  </p>
  {#if settingsMessage}<p class="form-message" role="status">{settingsMessage}</p>{/if}
  {#if settingsError}<p class="form-error" role="alert">{settingsError}</p>{/if}
</section>

<section class="panel" aria-labelledby="seasons-heading">
  <div class="panel-heading">
    <div>
      <p class="eyebrow">Competition periods</p>
      <h2 id="seasons-heading">Seasons</h2>
    </div>
  </div>

  <form class="configuration-form" on:submit|preventDefault={saveSeason}>
    <label>
      Starting year
      <input bind:value={seasonForm.startingYear} min="2000" required type="number" />
    </label>
    <button disabled={savingSeason} type="submit">
      {savingSeason ? 'Saving…' : 'Add season'}
    </button>
  </form>
  <p class="form-hint">A season starting in 2026 is shown as 2026–2027.</p>
  {#if seasonMessage}<p class="form-message" role="status">{seasonMessage}</p>{/if}
  {#if seasonError}<p class="form-error" role="alert">{seasonError}</p>{/if}

  {#if data.seasons.length === 0}
    <div class="empty-state compact">
      <h3>No seasons configured yet</h3>
      <p>Add a season before creating a team context.</p>
    </div>
  {:else}
    <div class="configuration-list" aria-label="Configured seasons">
      {#each data.seasons as season (season.startingYear)}
        <div class="configuration-row">
          <div>
            <strong>{season.startingYear}–{season.endingYear}</strong>
            <span>Available for team contexts</span>
          </div>
          <div class="row-actions">
            <button class="edit-button" type="button" on:click={() => openEdit('season', season)}>
              Edit
            </button>
            <button
              class="delete-button"
              type="button"
              on:click={() =>
                deleteItem(
                  `/api/seasons?startingYear=${season.startingYear}`,
                  `${season.startingYear}–${season.endingYear} season`,
                )}
            >
              Delete
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</section>

<section class="panel configuration-panel" aria-labelledby="teams-heading">
  <div class="panel-heading">
    <div>
      <p class="eyebrow">Roster groups</p>
      <h2 id="teams-heading">Teams</h2>
    </div>
  </div>

  <form class="configuration-form" on:submit|preventDefault={saveTeam}>
    <label>
      Team name
      <input bind:value={teamForm.name} placeholder="e.g. U16-1" required />
    </label>
    <button disabled={savingTeam} type="submit">
      {savingTeam ? 'Saving…' : 'Add team'}
    </button>
  </form>
  <p class="form-hint">Teams can be used in more than one season.</p>
  {#if teamMessage}<p class="form-message" role="status">{teamMessage}</p>{/if}
  {#if teamError}<p class="form-error" role="alert">{teamError}</p>{/if}

  {#if data.teams.length === 0}
    <div class="empty-state compact">
      <h3>No teams configured yet</h3>
      <p>Add a team before creating a team context.</p>
    </div>
  {:else}
    <div class="configuration-list" aria-label="Configured teams">
      {#each data.teams as team (team.name)}
        <div class="configuration-row">
          <div>
            <strong>{team.name}</strong>
            <span>Available across seasons</span>
          </div>
          <div class="row-actions">
            <button class="edit-button" type="button" on:click={() => openEdit('team', team)}>
              Edit
            </button>
            <button
              class="delete-button"
              type="button"
              on:click={() =>
                deleteItem(`/api/teams?name=${encodeURIComponent(team.name)}`, `${team.name} team`)}
            >
              Delete
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</section>

<section class="panel configuration-panel" aria-labelledby="locations-heading">
  <div class="panel-heading">
    <div>
      <p class="eyebrow">Reusable schedule data</p>
      <h2 id="locations-heading">Locations</h2>
    </div>
  </div>

  <form class="location-form" on:submit|preventDefault={saveLocation}>
    <label>
      Name
      <input bind:value={locationForm.name} placeholder="e.g. Home court" required />
    </label>
    <label>
      Address <span class="optional">optional</span>
      <input
        bind:value={locationForm.address}
        autocomplete="street-address"
        placeholder="e.g. Kamillehof 24, 3991GZ, Huizen"
      />
    </label>
    <label>
      Travel minutes
      <input bind:value={locationForm.travelMinutes} min="0" required type="number" />
    </label>
    <button disabled={savingLocation} type="submit">
      {savingLocation ? 'Saving…' : 'Add location'}
    </button>
  </form>
  <p class="form-hint">
    Add each venue here before using it for a game or training. Travel time is used for suggested
    game departure times.
  </p>
  {#if locationMessage}<p class="form-message" role="status">{locationMessage}</p>{/if}
  {#if locationError}<p class="form-error" role="alert">{locationError}</p>{/if}

  {#if data.locations.length === 0}
    <div class="empty-state compact">
      <h3>No locations configured yet</h3>
      <p>Add the first location to make it available in Events.</p>
    </div>
  {:else}
    <div class="location-list" aria-label="Configured locations">
      {#each data.locations as location (location.name)}
        <div class="location-row">
          <div>
            <strong>{location.name}</strong>
            {#if location.address}<span>{location.address}</span>{/if}
            <span>{location.travelMinutes} minutes travel</span>
          </div>
          <div class="row-actions">
            <button
              class="edit-button"
              type="button"
              on:click={() => openEdit('location', location)}
            >
              Edit
            </button>
            <button
              class="delete-button"
              type="button"
              on:click={() =>
                deleteItem(
                  `/api/locations?name=${encodeURIComponent(location.name)}`,
                  `${location.name} location`,
                )}
            >
              Delete
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</section>

<section class="panel import-panel" aria-labelledby="import-heading">
  <div class="panel-heading">
    <div>
      <p class="eyebrow">Spreadsheet mapping · preview · import</p>
      <h2 id="import-heading">Import data</h2>
    </div>
  </div>

  <label class="primary-team">
    What are you importing?
    <select bind:value={importKind} on:change={handleImportKindChange}>
      <option value="games">Games</option>
      <option value="players">Roster</option>
      <option value="locations">Locations</option>
    </select>
  </label>

  <label class="file-picker">
    Import file
    <input accept=".csv,.xls,.xlsx,.ods,text/csv" type="file" on:change={handleFile} />
  </label>

  {#if fileName}
    <p class="file-name">Selected: <strong>{fileName}</strong></p>
    {#if worksheets.length > 1}
      <label class="primary-team">
        Worksheet
        <select bind:value={sheetName} on:change={handleWorksheetChange}>
          {#each worksheets as worksheet (worksheet)}
            <option value={worksheet}>{worksheet}</option>
          {/each}
        </select>
      </label>
      <p class="form-hint">Choose which worksheet contains the schedule data.</p>
    {/if}
    <div class="mapping-grid">
      {#each fields as field (field.key)}
        <label>
          {field.label}{#if field.required}
            <span class="required">required</span>{/if}
          <select bind:value={mapping[field.key]}>
            <option value="">Not mapped</option>
            {#each headers as header (header)}
              <option value={header}>{header}</option>
            {/each}
          </select>
        </label>
      {/each}
    </div>
    <div class="action-row">
      <button disabled={loading} type="button" on:click={previewImport}>
        {loading ? 'Previewing…' : `Preview mapped ${importKindLabels[importKind]}`}
      </button>
      {#if preview && preview.issues.length === 0 && preview.records.length > 0}
        <button
          class="secondary-button"
          disabled={importing || !allConflictChoicesSelected || !allPlayerDuplicatesResolved}
          type="button"
          on:click={runImport}
        >
          {importing ? 'Importing…' : `Import valid ${importKindLabels[importKind]}`}
        </button>
      {/if}
    </div>
  {:else}
    <div class="empty-state compact">
      <h3>Select an import file</h3>
      <p>CSV, XLS, XLSX, and ODS files are mapped and validated before import.</p>
    </div>
  {/if}

  {#if error}<p class="form-error" role="alert">{error}</p>{/if}
  {#if importMessage}<p class="form-message" role="status">{importMessage}</p>{/if}

  {#if importKind === 'games' && conflicts.length > 0}
    <section class="conflict-list" aria-labelledby="conflicts-heading">
      <h3 id="conflicts-heading">Resolve existing game differences</h3>
      <p class="form-hint">Choose which value should be kept for each conflicting field.</p>
      {#each conflicts as conflict (conflict.sourceRow)}
        <fieldset>
          <legend>
            Source row {conflict.sourceRow}{#if conflictGameSummary(conflict.sourceRow)}
              · {conflictGameSummary(conflict.sourceRow)}{/if}
          </legend>
          {#each conflict.fields as field (field.field)}
            <label>
              {conflictFieldLabel(field.field)}
              <select bind:value={conflictChoices[conflict.sourceRow][field.field]}>
                <option value="">Choose a value</option>
                <option value="existing">Existing: {field.existingValue}</option>
                <option value="imported">Imported: {field.importedValue}</option>
              </select>
            </label>
          {/each}
        </fieldset>
      {/each}
    </section>
  {/if}

  {#if importKind === 'players' && playerDuplicates.length > 0}
    <section class="conflict-list" aria-labelledby="duplicates-heading">
      <h3 id="duplicates-heading">Resolve possible duplicate players</h3>
      <p class="form-hint">
        These rows have no association ID and share a name with a player already on this roster.
        Choose whether to overwrite that player, add a new player anyway, or skip the row.
      </p>
      {#each playerDuplicates as duplicate (duplicate.sourceRow)}
        <fieldset>
          <legend>Source row {duplicate.sourceRow}</legend>
          <p class="form-hint">
            Matches: {duplicate.candidates
              .map(
                (candidate) =>
                  `${candidate.firstName}${candidate.lastName ? ` ${candidate.lastName}` : ''}${candidate.birthDate ? ` (born ${candidate.birthDate})` : ''}`,
              )
              .join(', ')}
          </p>
          <label>
            Resolution
            <select bind:value={duplicateChoices[duplicate.sourceRow].choice}>
              <option value="">Choose a resolution</option>
              <option value="overwrite">Overwrite the matched player</option>
              <option value="add">Add as a new player</option>
              <option value="skip">Skip this row</option>
            </select>
          </label>
          {#if duplicateChoices[duplicate.sourceRow].choice === 'overwrite' && duplicate.candidates.length > 1}
            <label>
              Player to overwrite
              <select bind:value={duplicateChoices[duplicate.sourceRow].matchedPlayerId}>
                <option value="">Choose a player</option>
                {#each duplicate.candidates as candidate (candidate.id)}
                  <option value={candidate.id}
                    >{candidate.firstName}{candidate.lastName
                      ? ` ${candidate.lastName}`
                      : ''}{candidate.birthDate ? ` (born ${candidate.birthDate})` : ''}</option
                  >
                {/each}
              </select>
            </label>
          {/if}
        </fieldset>
      {/each}
    </section>
  {/if}

  {#if preview}
    <div class="preview-summary" role="status">
      <strong>{preview.validRowCount} valid {importKindLabels[importKind]}</strong>
      <span>{preview.issues.length} validation issues</span>
    </div>
    {#if preview.issues.length > 0}
      <div class="issue-list">
        {#each preview.issues as issue, index (`${issue.row}-${issue.field ?? 'row'}-${index}`)}
          <p>
            Row {issue.row}{#if issue.field}
              · {issue.field}{/if}: {issue.message}
          </p>
        {/each}
      </div>
    {/if}
    {#if 'notices' in preview && preview.notices.length > 0}
      <div class="notice-list">
        {#each preview.notices as notice, index (`${notice.row}-${notice.field ?? 'row'}-${index}`)}
          <p>Row {notice.row}: {notice.message}</p>
        {/each}
      </div>
    {/if}
    {#if preview.records.length > 0}
      <div class="preview-table-wrap">
        <table>
          <thead>
            <tr>
              <th>Row</th>
              {#each previewColumns as column (column.key)}
                <th>{column.label}</th>
              {/each}
            </tr>
          </thead>
          <tbody>
            {#each preview.records as record (record.sourceRow)}
              <tr>
                <td>{record.sourceRow}</td>
                {#each previewColumns as column (column.key)}
                  <td>{record[column.key] ?? ''}</td>
                {/each}
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
    <p class="form-hint">
      Importing and field-level reconciliation will follow this preview slice.
    </p>
  {/if}
</section>

<style>
  .import-panel,
  .configuration-panel {
    margin-top: 1rem;
  }

  .configuration-form {
    align-items: end;
    display: grid;
    gap: 0.8rem;
    grid-template-columns: 1fr auto;
  }

  .configuration-form label {
    color: var(--muted);
    display: grid;
    font-size: 0.78rem;
    font-weight: 800;
    gap: 0.35rem;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .optional {
    font-size: 0.68rem;
    font-weight: 500;
    letter-spacing: 0;
    text-transform: none;
  }

  .configuration-form input {
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

  .configuration-form button {
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

  .configuration-form button:hover:not(:disabled) {
    background: var(--accent-dark);
  }

  .configuration-form button:disabled {
    cursor: wait;
    opacity: 0.55;
  }

  .configuration-list {
    border-top: 1px solid var(--line);
    display: grid;
    gap: 0.65rem;
    margin-top: 1.5rem;
    padding-top: 1rem;
  }

  .configuration-row {
    align-items: baseline;
    background: #faf7f0;
    border: 1px solid #ebe4d8;
    border-radius: 0.7rem;
    display: flex;
    gap: 1rem;
    justify-content: space-between;
    padding: 0.8rem 1rem;
  }

  .configuration-row > div,
  .location-row > div {
    display: grid;
    gap: 0.2rem;
  }

  .configuration-row span {
    color: var(--muted);
    font-size: 0.84rem;
  }

  .row-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.45rem;
  }

  .edit-button,
  .delete-button {
    background: transparent;
    border: 1px solid var(--line);
    color: var(--ink);
    font-size: 0.78rem;
    min-height: 2.2rem;
    padding: 0.45rem 0.65rem;
  }

  .edit-button:hover {
    background: #fffdf8;
    border-color: var(--accent);
  }

  .delete-button {
    border-color: #e2b8aa;
    color: var(--accent-dark);
  }

  .delete-button:hover {
    background: #fff4ef;
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
    max-width: 32rem;
    width: 100%;
  }

  .modal-header {
    align-items: start;
    display: flex;
    gap: 1rem;
    justify-content: space-between;
    margin-bottom: 1.25rem;
  }

  .modal-header h2 {
    font-size: clamp(1.5rem, 4vw, 2rem);
    letter-spacing: -0.05em;
    margin: 0;
  }

  .close-button {
    background: transparent;
    border: 1px solid var(--line);
    color: var(--ink);
    cursor: pointer;
    font: inherit;
    font-size: 0.82rem;
    font-weight: 800;
    min-height: 2.2rem;
    padding: 0.45rem 0.65rem;
  }

  .edit-form {
    grid-template-columns: 1fr;
  }

  .location-form {
    align-items: end;
    display: grid;
    gap: 0.8rem;
    grid-template-columns: 1fr 12rem auto;
  }

  .location-form label {
    color: var(--muted);
    display: grid;
    font-size: 0.78rem;
    font-weight: 800;
    gap: 0.35rem;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .location-form input {
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

  .location-list {
    border-top: 1px solid var(--line);
    display: grid;
    gap: 0.65rem;
    margin-top: 1.5rem;
    padding-top: 1rem;
  }

  .location-row {
    align-items: baseline;
    background: #faf7f0;
    border: 1px solid #ebe4d8;
    border-radius: 0.7rem;
    display: flex;
    gap: 1rem;
    justify-content: space-between;
    padding: 0.8rem 1rem;
  }

  .location-row span {
    color: var(--muted);
    font-size: 0.84rem;
  }

  .file-picker,
  .primary-team,
  .mapping-grid label {
    color: var(--muted);
    display: grid;
    font-size: 0.78rem;
    font-weight: 800;
    gap: 0.35rem;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  input[type='file'],
  select {
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
    text-transform: none;
    width: 100%;
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

  button:hover:not(:disabled) {
    background: var(--accent-dark);
  }

  button:disabled {
    cursor: wait;
    opacity: 0.55;
  }

  .file-name,
  .form-hint,
  .form-message,
  .form-error {
    font-size: 0.88rem;
    margin: 1rem 0 0;
  }

  .file-name,
  .form-hint {
    color: var(--muted);
  }

  .form-message {
    color: #397044;
  }

  .form-error {
    color: var(--accent-dark);
  }

  .action-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.7rem;
  }

  .secondary-button {
    background: transparent;
    border: 1px solid var(--accent);
    color: var(--accent-dark);
  }

  .secondary-button:hover:not(:disabled) {
    background: #fff4ef;
  }

  .mapping-grid {
    display: grid;
    gap: 0.8rem;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    margin: 1rem 0;
  }

  .required {
    color: var(--accent-dark);
    font-size: 0.68rem;
    letter-spacing: 0;
    text-transform: lowercase;
  }

  .conflict-list {
    border-top: 1px solid var(--line);
    margin-top: 1.5rem;
    padding-top: 1rem;
  }

  .conflict-list h3 {
    margin: 0;
  }

  .conflict-list fieldset {
    border: 1px solid #e2b8aa;
    border-radius: 0.7rem;
    display: grid;
    gap: 0.8rem;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    margin: 1rem 0 0;
    padding: 1rem;
  }

  .conflict-list legend,
  .conflict-list label {
    color: var(--muted);
    font-size: 0.78rem;
    font-weight: 800;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .conflict-list label {
    display: grid;
    gap: 0.35rem;
  }

  .preview-summary {
    align-items: baseline;
    border-top: 1px solid var(--line);
    display: flex;
    gap: 1rem;
    margin-top: 1.5rem;
    padding-top: 1rem;
  }

  .preview-summary span {
    color: var(--muted);
    font-size: 0.88rem;
  }

  .issue-list {
    background: #fff4ef;
    border: 1px solid #e2b8aa;
    border-radius: 0.7rem;
    margin-top: 0.8rem;
    padding: 0.7rem 1rem;
  }

  .issue-list p {
    color: var(--accent-dark);
    font-size: 0.84rem;
    margin: 0.3rem 0;
  }

  .notice-list {
    background: #faf7f0;
    border: 1px solid #ebe4d8;
    border-radius: 0.7rem;
    margin-top: 0.8rem;
    padding: 0.7rem 1rem;
  }

  .notice-list p {
    color: var(--muted);
    font-size: 0.84rem;
    margin: 0.3rem 0;
  }

  .preview-table-wrap {
    margin-top: 1rem;
    overflow-x: auto;
  }

  table {
    border-collapse: collapse;
    min-width: 42rem;
    width: 100%;
  }

  th,
  td {
    border-bottom: 1px solid var(--line);
    padding: 0.7rem;
    text-align: left;
  }

  th {
    color: var(--muted);
    font-size: 0.72rem;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }

  td {
    font-size: 0.88rem;
  }

  .compact {
    padding: 1.25rem;
  }

  @media (max-width: 48rem) {
    .configuration-form {
      grid-template-columns: 1fr;
    }

    .configuration-form button {
      justify-self: start;
    }

    .location-form {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }

    .location-form button {
      grid-column: 1 / -1;
    }

    .mapping-grid,
    .conflict-list fieldset {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }

  @media (max-width: 36rem) {
    .configuration-row {
      align-items: start;
      flex-direction: column;
      gap: 0.25rem;
    }

    .location-form {
      grid-template-columns: 1fr;
    }

    .location-form button {
      grid-column: auto;
    }

    .location-row {
      align-items: start;
      flex-direction: column;
      gap: 0.25rem;
    }

    .mapping-grid,
    .conflict-list fieldset {
      grid-template-columns: 1fr;
    }
  }
</style>
