<script async lang="ts">
	import IconButton from '@smui/icon-button';
	import CircularProgress from '@smui/circular-progress';
	import { onMount } from 'svelte';
	let quotes: any[] = [];
	async function getQuotes() {
		const res = await fetch('https://quotes.3nt3.de/api/leaderboard');
		quotes = await res.json();
	}

	onMount(() => {
		getQuotes();
	});

	async function vote(id: number, vote: -1 | 1) {
		quotes = quotes.map((x: any) => {
			if (x.id == id) {
				return { ...x, score: x.score + vote };
			} else {
				return x;
			}
		});

		await fetch(`https://quotes.3nt3.de/api/vote/${id}/${vote}`, { method: 'post' });

		setTimeout(() => getQuotes(), 300);
		// const otherQuote = quotes.filter((q) => q.id != id)[0].id;
		// await fetch(`https://quotes.3nt3.de/api/vote/{otherQuote}/-1`);
	}
</script>

<h1 class="mdc-typography--headline3">Leaderboard</h1>
<p class="mdc-typography--body1">Please only vote once so the data isn't skewed</p>
<ol>
	{#if quotes.length > 0}
		{#each quotes as quote}
			<li>
				<div class="mdc-typography--body1">
					<pre>{quote.content}</pre>
					<p>Score: {quote.score}</p>
					<div id="actions">
						<IconButton class="material-icons" on:click={() => vote(quote.id, 1)}
							>favorite</IconButton
						>
						<IconButton class="material-icons" on:click={() => vote(quote.id, -1)}
							>remove_circle_outline</IconButton
						>
						<div />
					</div>
				</div>
			</li>
		{/each}
	{:else}
		<p>Loading</p>
		<CircularProgress style="width: 32px; height: 32px" indeterminate />
	{/if}
</ol>

<style>
	#actions {
		display: flex;
	}
</style>
