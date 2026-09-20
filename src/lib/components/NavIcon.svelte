<script>
  /** @type {'calendar' | 'chat' | 'clipboard' | 'history' | 'home' | 'settings' | 'users'} */
  export let name;
</script>

<svg aria-hidden="true" fill="none" viewBox="0 0 24 24">
  {#if name === 'home'}
    <path d="m3 10 9-7 9 7" />
    <path d="M5 9.5V21h14V9.5M9 21v-6h6v6" />
  {:else if name === 'calendar'}
    <rect height="17" rx="2" width="18" x="3" y="4" />
    <path d="M7 2v4M17 2v4M3 9h18" />
    <path d="M8 13h.01M12 13h.01M16 13h.01M8 17h.01M12 17h.01" />
  {:else if name === 'users'}
    <circle cx="9" cy="8" r="3" />
    <path d="M3 21v-1a6 6 0 0 1 12 0v1M16 4.5a3 3 0 0 1 0 5.8M18 14a5 5 0 0 1 3 4.5V21" />
  {:else if name === 'clipboard'}
    <rect height="16" rx="2" width="14" x="5" y="5" />
    <path d="M9 5V3h6v2M8 12l2 2 5-5" />
  {:else if name === 'chat'}
    <path
      d="M20 11.5a7.5 7.5 0 0 1-8 7.5 8.7 8.7 0 0 1-3.3-.6L4 20l1.6-3.8A7.2 7.2 0 0 1 4 11.5 7.5 7.5 0 0 1 12 4a7.5 7.5 0 0 1 8 7.5Z"
    />
    <path d="M8 12h.01M12 12h.01M16 12h.01" />
  {:else if name === 'history'}
    <path d="M3 12a9 9 0 1 0 3-6.7" />
    <path d="M3 4v5h5M12 7v5l3 2" />
  {:else if name === 'settings'}
    <path d="M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8Z" />
    <path
      d="m19.4 15 .1.1a2 2 0 1 1-2.8 2.8l-.1-.1a2 2 0 0 0-3.4 1.4v.3a2 2 0 1 1-4 0v-.3a2 2 0 0 0-3.4-1.4l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1A2 2 0 0 0 3.6 12a2 2 0 0 0-.6-1.4l-.1-.1A2 2 0 1 1 5.7 7.7l.1.1A2 2 0 0 0 9.2 6.4v-.3a2 2 0 1 1 4 0v.3a2 2 0 0 0 3.4 1.4l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a2 2 0 0 0 0 3.4Z"
    />
  {/if}
</svg>

<style>
  svg {
    height: 1.25rem;
    stroke: currentColor;
    stroke-linecap: round;
    stroke-linejoin: round;
    stroke-width: 1.8;
    width: 1.25rem;
  }
</style>
