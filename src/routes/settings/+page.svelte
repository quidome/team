<script>
  /** @typedef {import('$lib/application/imports/game-import').GameImportPreview} GameImportPreview */

  const fields = [
    { key: 'homeTeamName', label: 'Home team', required: true },
    { key: 'awayTeamName', label: 'Away team', required: true },
    { key: 'date', label: 'Date', required: true },
    { key: 'startTime', label: 'Start time', required: true },
    { key: 'locationName', label: 'Location', required: true },
    { key: 'travelMinutes', label: 'Travel minutes', required: false },
    { key: 'arrivalBufferMinutes', label: 'Arrival buffer', required: false },
  ];

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
  /** @type {GameImportPreview | undefined} */
  let preview = undefined;
  let loading = false;
  let importing = false;
  let error = '';
  let importMessage = '';
  let primaryTeamName = 'U16-1';
  /** @type {Array<{sourceRow: number, existingOccurrenceId: string, fields: Array<{field: string, existingValue: string | number, importedValue: string | number}>}>} */
  let conflicts = [];
  /** @type {Record<string, Record<string, string>>} */
  let conflictChoices = {};

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

  /** @param {string[]} availableHeaders */
  const guessMapping = (availableHeaders) => {
    /** @type {Record<string, string>} */
    const next = {};
    /** @type {Record<string, string[]>} */
    const aliases = {
      awayTeamName: ['away', 'away team', 'visitor'],
      arrivalBufferMinutes: ['arrival buffer', 'buffer'],
      date: ['date', 'datum'],
      homeTeamName: ['home', 'home team'],
      locationName: ['location', 'venue', 'plaats'],
      startTime: ['start time', 'time', 'tijd'],
      travelMinutes: ['travel', 'travel minutes', 'reistijd'],
    };

    for (const field of fields) {
      const match = availableHeaders.find((header) =>
        aliases[field.key].includes(header.toLowerCase()),
      );

      if (match) next[field.key] = match;
    }

    return next;
  };

  const loadHeaders = async () => {
    const response = await fetch('/api/imports/games/preview', {
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
  };

  const previewImport = async () => {
    loading = true;
    error = '';

    try {
      const response = await fetch('/api/imports/games/preview', {
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

  const allConflictChoicesSelected = () =>
    conflicts.every((conflict) =>
      conflict.fields.every((field) => Boolean(conflictChoices[conflict.sourceRow]?.[field.field])),
    );

  const importValidGames = async () => {
    importing = true;
    error = '';
    importMessage = '';

    try {
      const response = await fetch('/api/imports/games', {
        body: JSON.stringify({
          content,
          encoding,
          fileName,
          mapping,
          ...(sheetName ? { sheetName } : {}),
          primaryTeamName,
          resolutions: conflicts.map((conflict) => ({
            fields: conflictChoices[conflict.sourceRow],
            sourceRow: conflict.sourceRow,
          })),
          sourceName: fileName,
        }),
        headers: { 'content-type': 'application/json' },
        method: 'POST',
      });
      const body = await response.json();

      if (response.status === 409 && body.error === 'import_conflicts') {
        conflicts = body.conflicts;
        conflictChoices = Object.fromEntries(
          conflicts.map((conflict) => [
            conflict.sourceRow,
            Object.fromEntries(conflict.fields.map((field) => [field.field, ''])),
          ]),
        );
        return;
      }

      if (!response.ok) {
        throw new Error(body.error ?? 'The games could not be imported.');
      }

      conflicts = [];
      conflictChoices = {};
      /** @type {Array<{merged: boolean}>} */
      const importedGames = body.imported;
      const mergedCount = importedGames.filter((game) => game.merged).length;
      importMessage = `${body.imported.length} games imported; ${body.duplicates.length} duplicates skipped${mergedCount ? `; ${mergedCount} conflicts merged.` : '.'}`;
    } catch (caught) {
      error = caught instanceof Error ? caught.message : 'The games could not be imported.';
    } finally {
      importing = false;
    }
  };
</script>

<svelte:head>
  <title>Settings · Team</title>
  <meta name="description" content="Season, team, locations, and imports" />
</svelte:head>

<section class="intro">
  <p class="eyebrow">Configuration and imports</p>
  <h1>Settings</h1>
  <p class="lede">Preview schedule data before it is allowed to change the shared program.</p>
</section>

<section class="panel" aria-labelledby="import-heading">
  <div class="panel-heading">
    <div>
      <p class="eyebrow">Spreadsheet mapping · preview · import</p>
      <h2 id="import-heading">Import program data</h2>
    </div>
  </div>

  <label class="file-picker">
    Schedule file
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
    <label class="primary-team">
      Primary team
      <input bind:value={primaryTeamName} required />
    </label>
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
        {loading ? 'Previewing…' : 'Preview mapped games'}
      </button>
      {#if preview && preview.issues.length === 0 && preview.records.length > 0}
        <button
          class="secondary-button"
          disabled={importing || !allConflictChoicesSelected()}
          type="button"
          on:click={importValidGames}
        >
          {importing ? 'Importing…' : 'Import valid games'}
        </button>
      {/if}
    </div>
  {:else}
    <div class="empty-state compact">
      <h3>Select a schedule export</h3>
      <p>CSV, XLS, XLSX, and ODS files are mapped and validated before import.</p>
    </div>
  {/if}

  {#if error}<p class="form-error" role="alert">{error}</p>{/if}
  {#if importMessage}<p class="form-message" role="status">{importMessage}</p>{/if}

  {#if conflicts.length > 0}
    <section class="conflict-list" aria-labelledby="conflicts-heading">
      <h3 id="conflicts-heading">Resolve existing game differences</h3>
      <p class="form-hint">Choose which value should be kept for each conflicting field.</p>
      {#each conflicts as conflict (conflict.sourceRow)}
        <fieldset>
          <legend>Source row {conflict.sourceRow}</legend>
          {#each conflict.fields as field (field.field)}
            <label>
              {field.field}
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

  {#if preview}
    <div class="preview-summary" role="status">
      <strong>{preview.validRowCount} valid games</strong>
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
    {#if preview.records.length > 0}
      <div class="preview-table-wrap">
        <table>
          <thead>
            <tr>
              <th>Row</th>
              <th>Home</th>
              <th>Away</th>
              <th>Date</th>
              <th>Time</th>
              <th>Location</th>
            </tr>
          </thead>
          <tbody>
            {#each preview.records as record (record.sourceRow)}
              <tr>
                <td>{record.sourceRow}</td>
                <td>{record.homeTeamName}</td>
                <td>{record.awayTeamName}</td>
                <td>{record.date}</td>
                <td>{record.startTime}</td>
                <td>{record.locationName}</td>
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
    .mapping-grid,
    .conflict-list fieldset {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }

  @media (max-width: 36rem) {
    .mapping-grid,
    .conflict-list fieldset {
      grid-template-columns: 1fr;
    }
  }
</style>
