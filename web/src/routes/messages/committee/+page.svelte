<!--
	The committee message (story D of duties): the Dutch text to copy for the referee committee.
-->
<script lang="ts">
	import { resolve } from '$app/paths';
	import MessagePage from '#lib/components/MessagePage.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<MessagePage
	title="Committee message"
	back={resolve('/')}
	empty="No message: no team chosen."
	message={data.message}
/>
