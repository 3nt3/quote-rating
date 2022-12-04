<script async lang="ts">
	import IconButton from '@smui/icon-button';
	import CircularProgress from '@smui/circular-progress';
	import { Bar } from 'svelte-chartjs';

	import {
		Chart as ChartJS,
		Title,
		Tooltip,
		Legend,
		LinearScale,
		PointElement,
		CategoryScale,
		BarElement,
		Colors
	} from 'chart.js';
	ChartJS.register(
		Title,
		Tooltip,
		Legend,
		LinearScale,
		CategoryScale,
		PointElement,
		BarElement,
		Colors
	);

	import { onMount } from 'svelte';
	let quotes: any[] = [];
	async function getQuotes() {
		const res = await fetch('https://quotes.3nt3.de/api/leaderboard');
		quotes = await res.json();
	}

	let funniestPeople: any[] = [];
	let chartData: any = { labels: ['1', '2', '3'], datasets: [{ label: 'asdf', data: [1, 2, 3] }] };
	let averageChartData: any = {};

	async function getFunniestPeople() {
		const res = await fetch('https://quotes.3nt3.de/api/funniest-people');
		funniestPeople = (await res.json()).filter((x: any) => x.username !== null);
		chartData = {
			labels: funniestPeople.map((x) => x.username),
			datasets: [
				{ data: funniestPeople.map((x) => x.n_quotes), label: "Number of user's quotes" },
				{ data: funniestPeople.map((x) => x.sum_score), label: "Total score of user's quotes" }
			]
		};

		const sortedByAverageScore = funniestPeople.sort((a, b) => b.avg_score - a.avg_score);
		averageChartData = {
			labels: sortedByAverageScore.map((x) => x.username),
			datasets: [
				{ data: sortedByAverageScore.map((x) => x.avg_score), label: "Average score user's quotes" }
			]
		};
	}

	onMount(() => {
		getQuotes();
		getFunniestPeople();
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

<h2 class="mdc-typography--headline4">Funniest people</h2>
{#if funniestPeople.length > 0}
	<div class="chart">
		<Bar id="chart" data={chartData} options={{ responsive: true }} />
	</div>
	{#if averageChartData.datasets[0].data.length > 0}
		<div class="chart">
			<Bar data={averageChartData} options={{ responsive: true }} />
		</div>
	{/if}
{:else}
	<CircularProgress style="width: 32px; height: 32px" indeterminate />
{/if}

<h2 class="mdc-typography--headline4">Top 100 quotes</h2>
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

	.chart {
		height: 25rem;
	}

	pre {
		display: block;
		white-space: pre-wrap;
		word-wrap: break-word;
		hyphens: auto;
		-webkit-hyphens: auto;
	}
</style>
