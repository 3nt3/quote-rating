<script lang="ts">
	import { API_URL } from '../../src/api';
	import { onMount } from 'svelte';

	let progress = 0;
	let progressLoading = true;
	let progressError = false;

	onMount(() => {
		getProgress();
	});

	async function getProgress() {
		progressLoading = true;
		try {
			let res = await fetch(API_URL + '/stats?format=json');
			let stats = await res.json();
			progress = (stats.num_rated / stats.num_quotes) * 100;
			progressError = false;
		} catch {
			progressError = true;
		}
		progressLoading = false;
	}
</script>

<main class="bg-slate-900 h-screen overflow-hidden">
	<div
		class="flex items-center flex-col gap-1 ease-in-out transition-color transition-opacity duration-300 w-screen"
	>
		<div
			class="w-screen bg-slate-800 transition-color duration-300"
			class:animate-pulse={progressLoading}
			class:bg-red-500={progressError}
		>
			<div class="bg-indigo-400 h-1" style="width: {progress ? progress : 0}%" />
		</div>
		<p
			class="text-xs text-slate-300 opacity-0 ease-in-out duration-300"
			class:opacity-100={!progressLoading}
		>
			{#if !progressError}
				{progress.toFixed(0)}% rated
			{:else}
				error talking to server 😥
			{/if}
		</p>
	</div>
	<div class="flex justify-center items-center flex-col text-center text-slate-200 h-screen">
		<h1 class="text-4xl font-bold">quotes.3nt3.de</h1>
		<p class="text-xl">"because there's just too many of them"</p>
		<div
			class="flex flex-col sm:flex-row gap-3 items-center w-full px-3 justify-center mt-3 font-bold"
		>
			<a
				href="/vote"
				class="flex items-center justify-center bg-slate-800 rounded-xl w-full sm:w-72 h-32 border-2 border-slate-700"
				>Vote</a
			>
			<a
				href="/leaderboard"
				class="flex items-center justify-center  bg-slate-800 rounded-xl w-full sm:w-72 h-32 border-2 border-slate-700"
				>Leaderboard</a
			>
		</div>
	</div>
	<!-- content -->
</main>
