<script async lang="ts">
	async function getQuotes() {
		const res = await fetch('http://localhost:8000/leaderboard');
		return await res.json();
	}
</script>

<h1>Leaderboard</h1>
<ol>
	{#await getQuotes() then quotes}
		{#each quotes as quote}
			<li>
				<div>
					<pre>{quote.content}</pre>
					<p>Score: {quote.score}</p>
				</div>
			</li>
		{/each}
	{/await}
</ol>
