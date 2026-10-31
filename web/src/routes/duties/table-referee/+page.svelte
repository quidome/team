<!--
	The table and referee overview (story E of duties): the Dutch text to copy with all coming table
	and referee places of the team.
-->
<script lang="ts">
	import { resolve } from '$app/paths';
	import MessagePage from '#lib/components/MessagePage.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<MessagePage
	title="Table and referee overview"
	back={resolve('/duties')}
	empty="No overview: no team chosen."
	message={data.message}
/>
