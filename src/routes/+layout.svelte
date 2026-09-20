<script>
  import { resolve } from '$app/paths';
</script>

<svelte:head>
  <meta name="theme-color" content="#f4f0e8" />
</svelte:head>

<header class="site-header">
  <a class="brand" href={resolve('/')}>Team <span>U16-1</span></a>
  <nav aria-label="Primary navigation">
    <a href={resolve('/')}>Home</a>
    <a href={resolve('/program')}>Program</a>
    <a href={resolve('/team')}>Team</a>
    <a href={resolve('/duties')}>Duties</a>
    <a href={resolve('/messages')}>Messages</a>
    <a href={resolve('/history')}>History</a>
    <a href={resolve('/settings')}>Settings</a>
  </nav>
  <a class="login-link" href={resolve('/auth/login')}>Coordinator login</a>
</header>

<main class="page-shell">
  <slot />
</main>

<style>
  :global(:root) {
    --accent: #d5633e;
    --accent-dark: #9f3c27;
    --ink: #24221f;
    --line: #d8d0c4;
    --muted: #716a61;
    --paper: #f4f0e8;
    --panel: #fffdf8;
    color: var(--ink);
    font-family: 'Avenir Next', Avenir, 'Segoe UI', sans-serif;
    font-synthesis: none;
  }

  :global(body) {
    background: var(--paper);
    margin: 0;
  }

  :global(a) {
    color: inherit;
  }

  .site-header {
    align-items: center;
    border-bottom: 1px solid var(--line);
    display: flex;
    gap: 2rem;
    margin: 0 auto;
    max-width: 72rem;
    padding: 1.25rem 1.5rem;
  }

  .brand {
    font-size: 1.25rem;
    font-weight: 800;
    letter-spacing: -0.04em;
    text-decoration: none;
    white-space: nowrap;
  }

  .brand span {
    color: var(--accent);
    font-size: 0.75rem;
    letter-spacing: 0.04em;
    margin-left: 0.25rem;
    text-transform: uppercase;
  }

  nav {
    display: flex;
    flex: 1;
    gap: 1.25rem;
  }

  nav a,
  .login-link {
    color: var(--muted);
    font-size: 0.9rem;
    text-decoration: none;
  }

  nav a:hover,
  nav a:focus-visible,
  .login-link:hover,
  .login-link:focus-visible {
    color: var(--accent-dark);
  }

  .login-link {
    border: 1px solid var(--line);
    border-radius: 999px;
    padding: 0.55rem 0.85rem;
    white-space: nowrap;
  }

  .page-shell {
    margin: 0 auto;
    max-width: 72rem;
    padding: 3.5rem 1.5rem 5rem;
  }

  :global(.intro) {
    margin-bottom: 2.5rem;
    max-width: 42rem;
  }

  :global(.eyebrow) {
    color: var(--accent-dark);
    font-size: 0.72rem;
    font-weight: 800;
    letter-spacing: 0.12em;
    margin: 0 0 0.5rem;
    text-transform: uppercase;
  }

  :global(h1) {
    font-size: clamp(2.5rem, 7vw, 5rem);
    letter-spacing: -0.07em;
    line-height: 0.95;
    margin: 0;
  }

  :global(.lede) {
    color: var(--muted);
    font-size: 1.1rem;
    line-height: 1.5;
    margin: 1rem 0 0;
  }

  :global(.panel) {
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 1.25rem;
    padding: clamp(1.25rem, 4vw, 2rem);
  }

  :global(.panel-heading) {
    align-items: end;
    display: flex;
    justify-content: space-between;
    margin-bottom: 1.5rem;
  }

  :global(.panel-heading h2) {
    font-size: clamp(1.5rem, 4vw, 2rem);
    letter-spacing: -0.05em;
    margin: 0;
  }

  :global(.text-link) {
    color: var(--accent-dark);
    font-size: 0.9rem;
    font-weight: 700;
    text-decoration: none;
  }

  :global(.empty-state) {
    border: 1px dashed var(--line);
    border-radius: 0.9rem;
    padding: 2rem;
    text-align: center;
  }

  :global(.empty-state h3) {
    font-size: 1.1rem;
    margin: 0;
  }

  :global(.empty-state p) {
    color: var(--muted);
    margin: 0.5rem 0 0;
  }

  :global(.event-list) {
    display: grid;
    gap: 0.75rem;
  }

  :global(.event-card) {
    align-items: center;
    background: #faf7f0;
    border: 1px solid #ebe4d8;
    border-radius: 0.9rem;
    display: grid;
    gap: 1rem;
    grid-template-columns: minmax(7rem, 0.25fr) 1fr;
    padding: 1rem;
  }

  :global(.event-date) {
    border-right: 1px solid var(--line);
    display: grid;
    gap: 0.25rem;
    padding-right: 1rem;
  }

  :global(.event-date span),
  :global(.event-details p),
  :global(.event-kind) {
    color: var(--muted);
    font-size: 0.84rem;
    margin: 0;
  }

  :global(.event-date strong) {
    font-size: 1.25rem;
  }

  :global(.event-kind) {
    color: var(--accent-dark);
    font-size: 0.7rem;
    font-weight: 800;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  :global(.event-details h3) {
    font-size: 1.1rem;
    letter-spacing: -0.03em;
    margin: 0.2rem 0 0.35rem;
  }

  @media (max-width: 48rem) {
    .site-header {
      align-items: start;
      flex-wrap: wrap;
      gap: 1rem;
    }

    nav {
      flex-basis: 100%;
      order: 3;
      overflow-x: auto;
      padding-bottom: 0.25rem;
    }

    .page-shell {
      padding-top: 2.5rem;
    }
  }

  @media (max-width: 36rem) {
    .site-header,
    .page-shell {
      padding-left: 1rem;
      padding-right: 1rem;
    }

    :global(.event-card) {
      align-items: start;
      grid-template-columns: 1fr;
    }

    :global(.event-date) {
      border-bottom: 1px solid var(--line);
      border-right: 0;
      padding-bottom: 0.75rem;
      padding-right: 0;
      width: 100%;
    }

    :global(.panel-heading) {
      align-items: start;
      flex-direction: column;
      gap: 0.75rem;
    }
  }
</style>
