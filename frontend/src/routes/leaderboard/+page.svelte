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
	<div class="sm:mt-12 w-[min(900px,90%)]">
		<a
			href="/"
			aria-label="Go back"
			class="inline-block justify-self-start ml-1 mr-auto justify-center rounded-full p-2 text-sm font-medium text-slate-300  focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 focus:ring-offset-gray-100 transition-all"
		>
			<svg
				xmlns="http://www.w3.org/2000/svg"
				viewBox="0 0 24 24"
				fill="currentColor"
				class="w-6 h-6"
				aria-hidden="true"
			>
				<path
					fill-rule="evenodd"
					d="M11.03 3.97a.75.75 0 010 1.06l-6.22 6.22H21a.75.75 0 010 1.5H4.81l6.22 6.22a.75.75 0 11-1.06 1.06l-7.5-7.5a.75.75 0 010-1.06l7.5-7.5a.75.75 0 011.06 0z"
					clip-rule="evenodd"
				/>
			</svg>
		</a>
		<div class=" bg-slate-800 rounded-xl p-6 shadow-lg mb-4">
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
	</div>
</main>
