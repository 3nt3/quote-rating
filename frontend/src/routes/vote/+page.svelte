<script lang="ts">
	// TODO: don't just copy this from the other file
	import { API_URL } from '../../../src/api';
	import { onMount } from 'svelte';
	import QuoteComponent from '../../components/quote.svelte';
	import type { Quote } from '../../models';
	import Dropdown from '../../components/dropdown.svelte';

	let progress = 100;
	let progressLoading = true;
	let progressError = false;

	let quotes: Quote[] = [];
	let quotesLoading = true;
	let quotesError = false;

	let preferUnrated = true;

	onMount(() => {
		fetchProgress();
		fetchQuotes();
	});

	async function fetchProgress() {
		progressLoading = true;
		try {
			const res = await fetch(API_URL + '/stats?format=json');
			const stats = await res.json();
			progress = (stats.num_rated / stats.num_quotes) * 100;
			progressError = false;
		} catch {
			progressError = true;
		}

		progressLoading = false;
	}

	async function fetchQuotes() {
		quotesLoading = true;
		try {
			const res = await fetch(API_URL + `/quote?preferUnrated=${preferUnrated}`);
			quotes = await res.json();
			quotesError = false;
		} catch {
			quotesError = true;
		}
		quotesLoading = false;
	}

	async function vote(id: number, vote: number) {
		const res = await fetch(API_URL + `/vote/${id}/${vote}`, { method: 'POST' });
		fetchQuotes();
		fetchProgress();
	}

	function onDropdownChange(newValue: string) {
		preferUnrated = newValue === 'yes';
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
			<div class="bg-indigo-400 h-1" style="width: {progress ? progress : 100}%" />
		</div>
		<p
			class="text-xs text-slate-200 opacity-0 ease-in-out duration-300"
			class:opacity-100={!progressLoading}
		>
			{#if !progressError}
				{progress.toFixed(2)}% rated
			{:else}
				error talking to server 😥
			{/if}
		</p>
	</div>
	<div
		class="text-slate-200 flex col sm:row justify-center w-full h-full items-center px-4 overflow-hidden"
	>
		{#if quotesLoading}
			Loading
		{:else if !quotesError}
			<div class="w-[min(800px,90%)] flex flex-col gap-4">
				<div class="flex justify-end">
					<Dropdown
						active={preferUnrated ? 'yes' : 'no'}
						options={{ yes: 'yes', no: 'no' }}
						title={'Prefer unrated?'}
						onChange={onDropdownChange}
					/>
					<!-- <Dropdown /> -->
				</div>
				<div class="flex gap-4 sm:flex-row flex-col">
					{#each quotes as quote}
						<QuoteComponent {quote} onVote={vote} />
					{/each}
				</div>
			</div>
		{:else}
			error talking to server 😥
		{/if}
	</div>
</main>
