<script lang="ts">
	import { API_URL } from '../../api';
	import { onMount } from 'svelte';
	import Spinner from '../../components/spinner.svelte';
	import QuoteComponent from '../../components/quote.svelte';
	import type { Quote } from '../../models';
	import { fade } from 'svelte/transition';

	let leaderboardLoading = true;
	let leaderboard: Quote[] | null = null;
	let leaderboardError = false;

	let funniestPeopleLoading = true;
	let funniestPeople = null;
	let funniestPeopleError = false;

	onMount(() => {
		fetchLeaderboard();
		fetchFunniestPeople();
	});

	async function fetchLeaderboard() {
		try {
			let res = await fetch(API_URL + '/leaderboard');
			leaderboard = await res.json();
			leaderboardError = false;
		} catch {
			leaderboardError = true;
		}
	}

	async function fetchFunniestPeople() {
		funniestPeopleLoading = true;

		try {
			let res = await fetch(API_URL + '/funniest-people');
			funniestPeople = await res.json();
			funniestPeopleError = false;
		} catch {
			funniestPeopleError = true;
		}

		funniestPeopleLoading = false;
	}

	async function onVote(id: number, vote: number) {
		const res = await fetch(API_URL + `/vote/${id}/${vote}`, { method: 'POST' });
		fetchLeaderboard();
	}
</script>

<main class="w-screen min-h-screen bg-slate-900 text-slate-300 flex justify-center">
	<div class="w-[min(900px,90%)] mt-12 bg-slate-800 rounded-xl p-6">
		<h1 class="text-2xl">Top 100</h1>
		<p>Pls don't spam we gotta appreciate the tastiest fruits 🍉</p>

		{#if !leaderboard}
			<div
				class="flex justify-center items-center h-72"
				in:fade={{ duration: 150 }}
				out:fade={{ duration: 150 }}
			>
				<Spinner />
			</div>
		{:else}
			<div class="flex flex-col gap-4 mt-4" in:fade={{ duration: 150 }}>
				{#each leaderboard || [] as quote}
					<QuoteComponent {quote} compact={true} {onVote} />
				{/each}
			</div>
		{/if}
	</div>
</main>
