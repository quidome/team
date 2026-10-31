<!--
	The message of the week (story C of duties): the Dutch text to copy into the chat of the team.
-->
<script lang="ts">
	import { resolve } from '$app/paths';
	import MessagePage from '#lib/components/MessagePage.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<MessagePage
	title="Message of the week"
	back={resolve('/')}
	empty="No message this week."
	message={data.message}
/>
