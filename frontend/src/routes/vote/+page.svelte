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

	interface Options {
		preferUnrated: Boolean;
	}

	let options: Options = {
		preferUnrated: true
	};

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
			const res = await fetch(API_URL + `/quote?prefer_unrated=${options.preferUnrated}`);
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
		options.preferUnrated = newValue === 'yes';
	}
</script>

<main class="bg-slate-900 min-h-screen overflow-hidden">
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
		class="text-slate-200 flex col sm:row justify-center w-full h-full items-center px-4 overflow-hidden mt-8 md:mt-0 min-h-screen"
	>
		{#if quotesLoading}
			Loading
		{:else if !quotesError}
			<div class="w-[min(800px,90%)] flex flex-col gap-4">
				<div class="flex justify-end gap-2">
					<Dropdown
						active={options.preferUnrated ? 'yes' : 'no'}
						options={{ yes: 'Yes', no: 'No' }}
						title={'Prefer unrated?'}
						onChange={onDropdownChange}
					/>
					<button
						class="px-4 rounded-md transition-all text-sm bg-indigo-500 hover:bg-indigo-600"
						on:click={() => {
							fetchQuotes();
						}}>Apply</button
					>
					<!-- <Dropdown /> -->
				</div>
				<div class="flex gap-4 sm:flex-row flex-col">
					{#each quotes as quote}
						<QuoteComponent {quote} onVote={vote} />
					{/each}
				</div>
				<div class="flex justify-center">
					<button
						class="stroke-teal-500 rounded-full p-2 ring-1 ring-teal-500 hover:ring-2 transition-shadow ease-in-out duration-300 "
						on:click={fetchQuotes}
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							fill="none"
							viewBox="0 0 24 24"
							stroke-width={1.5}
							stroke="currentColor"
							class="w-6 h-6"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0l3.181 3.183a8.25 8.25 0 0013.803-3.7M4.031 9.865a8.25 8.25 0 0113.803-3.7l3.181 3.182m0-4.991v4.99"
							/>
						</svg>
					</button>
				</div>
			</div>
		{:else}
			error talking to server 😥
		{/if}
	</div>
</main>
