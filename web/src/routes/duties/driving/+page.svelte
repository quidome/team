<!--
	The driving overview (story E of duties): the Dutch text to copy with all coming drives.
-->
<script lang="ts">
	import { resolve } from '$app/paths';
	import MessagePage from '#lib/components/MessagePage.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<MessagePage
	title="Driving overview"
	back={resolve('/duties')}
	empty="No overview: no team chosen."
	message={data.message}
/>
