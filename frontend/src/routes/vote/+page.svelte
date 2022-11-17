<script async lang="ts">
	import Card, { Content, Actions, Media, MediaContent } from '@smui/card';
	import IconButton, { Icon } from '@smui/icon-button';
	import Button, { Label } from '@smui/button';
	import CircularProgressIndicator from '@smui/circular-progress';
	import { onMount } from 'svelte';
	let quotes: any = [];

	onMount(async () => {
		await getQuotes();
	});

	async function getQuotes() {
		const res = await fetch('http://localhost:8000/quote');
		quotes = await res.json();
	}

	async function vote(id: number) {
		await fetch(`http://localhost:8000/vote/${id}/1`, { method: 'post' });
		await getQuotes();
		// const otherQuote = quotes.filter((q) => q.id != id)[0].id;
		// await fetch(`http://localhost:8000/vote/{otherQuote}/-1`);
	}
</script>

<h1 class="mdc-typography--headline3">Vote</h1>
<p class="mdc-typography--body1">Click on either of the quotes to vote for it</p>
<div id="container">
	<div id="quotes">
		{#each quotes as quote}
			<div class="quote">
				<Card>
					<Media class="card-media-16x9" aspectRatio="16x9">
						<MediaContent
							style={`background-image: url(${quote.avatar_url}); background-size: cover; background-position: center;`}
						/>
					</Media>
					<Content class="quote">
						<code class="code">{quote.content}</code>
						<p
							class={`mdc-typography--body1 ${
								quote.score > 0 ? 'green' : quote.score < 0 ? 'red' : ''
							}`}
						>
							Score: {quote.score}
						</p></Content
					>
					<Actions
						><IconButton class="material-icons" on:click={() => vote(quote.id)}
							>favorite_border</IconButton
						></Actions
					>
				</Card>
			</div>
		{/each}
	</div>
	<Button on:click={getQuotes}
		><Icon class="material-icons">shuffle</Icon><Label>Skip</Label></Button
	>
</div>

<style lang="scss">
	#container {
		width: 100vw;
		height: calc(100vh - 5rem);
		display: flex;
		justify-content: center;
		align-items: center;
		flex-direction: column;
	}
	#quotes {
		display: flex;
		gap: 1rem;
		align-items: stretch;

		width: min(1000px, 90%);
	}
	.quote {
		flex: 1 !important;
		code {
			word-break: break-all;
			hyphens: auto;
			white-space: pre;
		}

		width: 50% !important;

		.green {
			color: #4caf50;
		}
		.red {
			color: #f44336;
		}
	}
</style>
