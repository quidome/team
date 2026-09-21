<script>
  import { page } from '$app/stores';
  import { resolve } from '$app/paths';
  import { defaultTeamSeasonContext, formatTeamSeasonContext } from '$lib/application/team-context';
  import NavIcon from '$lib/components/NavIcon.svelte';

  export let data;

  const activeContextLabel = formatTeamSeasonContext(defaultTeamSeasonContext);

  /** @type {Array<{href: '/' | '/events' | '/team' | '/messages' | '/history' | '/admin', label: string, icon: 'calendar' | 'chat' | 'clipboard' | 'history' | 'home' | 'settings' | 'users', activePaths: string[]}>} */
  const navItems = [
    { activePaths: ['/'], href: '/', icon: 'home', label: 'Home' },
    { activePaths: ['/events'], href: '/events', icon: 'calendar', label: 'Events' },
    { activePaths: ['/team', '/events'], href: '/team', icon: 'users', label: 'Team' },
    { activePaths: ['/messages'], href: '/messages', icon: 'chat', label: 'Messages' },
    { activePaths: ['/history'], href: '/history', icon: 'history', label: 'History' },
    { activePaths: ['/admin'], href: '/admin', icon: 'settings', label: 'Admin' },
  ];

  $: pathname = $page.url.pathname;

  /** @param {{ activePaths: string[] }} item */
  const isActive = (item) =>
    item.activePaths.some(
      (path) => pathname === path || (path !== '/' && pathname.startsWith(`${path}/`)),
    );
</script>

<svelte:head>
  <meta name="theme-color" content="#f4f0e8" />
</svelte:head>

<header class="site-header">
  <a class="brand" href={resolve('/')}>Team <span>{activeContextLabel}</span></a>
  <nav aria-label="Primary navigation">
    {#each navItems as item (item.href)}
      <a
        aria-current={isActive(item) ? 'page' : undefined}
        class:active={isActive(item)}
        href={resolve(item.href)}
        title={item.label}
      >
        <NavIcon name={item.icon} />
        <span class="nav-label">{item.label}</span>
      </a>
    {/each}
  </nav>
  {#if data.isAuthenticated}
    <form class="logout-form" method="POST" action={resolve('/auth/logout')}>
      <button class="login-link" type="submit">Log out</button>
    </form>
  {:else}
    <a class="login-link" href={resolve('/auth/login')}>Coordinator login</a>
  {/if}
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
    align-items: center;
    display: flex;
    flex: 1;
    gap: 0.45rem;
    justify-content: center;
  }

  nav a,
  .login-link {
    color: var(--muted);
    font-size: 0.9rem;
    text-decoration: none;
  }

  nav a {
    align-items: center;
    border: 1px solid transparent;
    border-radius: 0.65rem;
    display: inline-flex;
    justify-content: center;
    min-height: 2.5rem;
    min-width: 2.5rem;
    padding: 0.55rem;
  }

  nav a.active {
    background: #fff4ef;
    border-color: #e2b8aa;
    color: var(--accent-dark);
  }

  .nav-label {
    height: 1px;
    margin: -1px;
    overflow: hidden;
    position: absolute;
    width: 1px;
    clip: rect(0 0 0 0);
    white-space: nowrap;
  }

  nav a:hover,
  nav a:focus-visible,
  .login-link:hover,
  .login-link:focus-visible {
    color: var(--accent-dark);
  }

  .login-link {
    appearance: none;
    background: none;
    border: 1px solid var(--line);
    border-radius: 999px;
    font: inherit;
    padding: 0.55rem 0.85rem;
    white-space: nowrap;
  }

  button.login-link {
    cursor: pointer;
  }

  .logout-form {
    display: contents;
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
    transition:
      border-color 120ms ease,
      box-shadow 120ms ease,
      transform 120ms ease;
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
      display: grid;
      flex-basis: 100%;
      gap: 0.35rem;
      grid-template-columns: repeat(6, minmax(0, 1fr));
      order: 3;
      overflow: visible;
      padding-bottom: 0;
    }

    nav a {
      padding: 0.4rem 0.2rem;
      width: 100%;
    }

    .page-shell {
      padding-top: 2.5rem;
    }
  }

  @media (max-width: 36rem) {
    .site-header,
    .page-shell {
      padding-left: max(1rem, env(safe-area-inset-left));
      padding-right: max(1rem, env(safe-area-inset-right));
    }

    .site-header {
      gap: 0.75rem;
      padding-bottom: 0.9rem;
      padding-top: 0.9rem;
    }

    nav a,
    .login-link {
      font-size: 0.8rem;
    }

    .login-link {
      padding: 0.5rem 0.7rem;
    }

    .page-shell {
      padding-bottom: max(3rem, env(safe-area-inset-bottom));
      padding-top: 1.75rem;
    }

    :global(.intro) {
      margin-bottom: 1.5rem;
    }

    :global(.lede) {
      font-size: 1rem;
    }

    :global(.panel) {
      border-radius: 1rem;
      padding: 1rem;
    }

    :global(.panel-heading) {
      align-items: start;
      flex-direction: column;
      gap: 0.65rem;
      margin-bottom: 1rem;
    }

    :global(.event-card) {
      align-items: start;
      gap: 0.75rem;
      grid-template-columns: 6.25rem minmax(0, 1fr);
      padding: 0.85rem;
    }

    :global(.event-date) {
      border-bottom: 0;
      border-right: 1px solid var(--line);
      padding-bottom: 0;
      padding-right: 0.75rem;
      width: auto;
    }

    :global(.event-date strong) {
      font-size: 1.1rem;
    }

    :global(.event-details h3) {
      font-size: 1rem;
    }
  }
</style>
