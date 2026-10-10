<script lang="ts">
	import './layout.css';
	import { resolve } from '$app/paths';
	import { Button } from '#lib/components/ui/button/index.js';
	import type { LayoutProps } from './$types';

	let { data, children }: LayoutProps = $props();
</script>

<div class="flex min-h-dvh flex-col">
	<header
		class="sticky top-0 z-10 border-b bg-background/90 pt-[env(safe-area-inset-top)] backdrop-blur"
	>
		<div class="mx-auto flex h-14 max-w-2xl items-center justify-between gap-3 px-4">
			<a href={resolve('/')} class="flex items-center gap-2 text-lg font-semibold tracking-tight">
				<span class="size-3 rounded-full bg-primary" aria-hidden="true"></span>
				Team
			</a>
			{#if data.me}
				<div class="flex min-w-0 items-center gap-2">
					<span class="truncate text-sm text-muted-foreground" data-testid="user-name">
						{data.me.name}
					</span>
					<form method="post" action="/auth/logout" data-sveltekit-reload>
						<Button type="submit" variant="ghost" size="sm">Log out</Button>
					</form>
				</div>
			{/if}
		</div>
	</header>

	<main
		class="mx-auto w-full max-w-2xl flex-1 px-4 py-6 pb-[calc(1.5rem+env(safe-area-inset-bottom))]"
	>
		{@render children()}
	</main>
</div>
