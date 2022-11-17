<script async lang="ts">
	import Card, { Content, Actions, Media, MediaContent } from '@smui/card';
	import IconButton, { Icon } from '@smui/icon-button';
	import Button, { Label } from '@smui/button';
	import CircularProgressIndicator from '@smui/circular-progress';
	import { onMount } from 'svelte';
	let quotes: any = [];
	let loading = true;

	onMount(async () => {
		await getQuotes();
	});

	async function getQuotes() {
		loading = true;
		const res = await fetch('https://quotes.3nt3.de/api/quote');
		quotes = await res.json();
		loading = false;
	}

	async function vote(id: number) {
		quotes = quotes.map((x: any) => {
			if (x === undefined) return;
			if (x.id == id) {
				return { ...x, score: x.score + 1 };
			} else {
				return { ...x, score: x.score - 1 };
			}
		});

		await fetch(`https://quotes.3nt3.de/api/vote/${id}/1`, { method: 'post' });

		const otherOne = quotes.filter((x: any) => x && x.id != id)[0].id;
		await fetch(`https://quotes.3nt3.de/api/vote/${otherOne}/-1`, { method: 'post' });

		setTimeout(async () => await getQuotes(), 300);
		// const otherQuote = quotes.filter((q) => q.id != id)[0].id;
		// await fetch(`https://quotes.3nt3.de/api/vote/{otherQuote}/-1`);
	}
</script>

<h1 class="mdc-typography--headline3">Vote</h1>
<div id="container">
	<div id="quotes">
		{#if loading}
			<CircularProgressIndicator style="width: 32px; height: 32px" class="spinner" indeterminate />
		{:else}
			{#each quotes as quote}
				<div class="quote">
					<Card>
						<Media class="card-media-16x9" aspectRatio="16x9">
							<MediaContent
								style={`background-image: url(${quote.avatar_url}); background-size: cover; background-position: center;`}
							/>
						</Media>
						<Content class="quote-content">
							<code class="code">{quote.content}</code>
							<p
								class={`mdc-typography--body1 ${
									quote.score > 0 ? 'green' : quote.score < 0 ? 'red' : ''
								}`}
							>
								Score: {quote.score}
							</p></Content
						>
						<Actions>
							<IconButton class="material-icons" on:click={() => vote(quote.id)}
								>favorite_border</IconButton
							>
						</Actions>
					</Card>
				</div>
			{/each}
		{/if}
	</div>
	<Button on:click={getQuotes}
		><Icon class="material-icons">shuffle</Icon><Label>Skip</Label></Button
	>
</div>

<style lang="scss">
	#container {
		width: 100vw;
		display: flex;
		justify-content: center;
		align-items: center;
		flex-direction: column;
		gap: 2rem;
	}
	#quotes {
		display: flex;
		gap: 1rem;
		justify-content: center;
		min-height: 300px;

		transition: all 1s ease-in-out;

		width: min(1000px, 90%);
	}
	.quote {
		flex: 1 !important;
		code {
			word-break: break-all;
			hyphens: auto;
			white-space: pre;
		}

		width: 50%;

		.green {
			color: #4caf50;
		}
		.red {
			color: #f44336;
		}
	}

	.spinner {
		margin: auto;
		alig-self: center;
	}

	@media screen and (max-width: 600px) {
		#container {
			width: min(1000px, 90%);
			margin: 0 auto;
		}
		#quotes {
			flex-direction: column;
			.quote {
				width: 100%;
			}
		}
	}
</style>
