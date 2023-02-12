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

  let quote: Quote | null = null;
  let quoteLoading = true;
  let quoteError = false;
  let nextQuote: Quote | null = null;

  interface Options {
    preferUnrated: Boolean;
    preferGood: Boolean;
  }

  let options: Options = {
    preferUnrated: true,
    preferGood: false
  };

  onMount(() => {
    fetchProgress();
    fetchQuote();
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

  async function fetchQuote() {
    quoteLoading = true;
    quote = nextQuote === null ? null : Object.assign({}, nextQuote);
    nextQuote = null;
    console.log('quote, nextQuote:', quote, nextQuote);
    try {
      if (!quote) {
        const res = await fetch(
          API_URL + `/quote?prefer_unrated=${options.preferUnrated}&only_good=${options.preferGood}`
        );
        quote = await res.json();
      }
      const res = await fetch(
        API_URL + `/quote?prefer_unrated=${options.preferUnrated}&only_good=${options.preferGood}`
      );
      nextQuote = await res.json();
      quoteError = false;
    } catch (e) {
      quoteError = true;
      console.log(e);
    }
    quoteLoading = false;
    console.log('quote, nextQuote:', quote, nextQuote);
  }

  async function vote(id: number, vote: number) {
    fetchQuote();
    const res = await fetch(API_URL + `/vote/${id}/${vote}`, { method: 'POST' });
    fetchProgress();
  }

  function preferUnratedChanged(newValue: string) {
    options.preferUnrated = newValue === 'yes';
    options.preferGood = newValue === 'yes' ? false : options.preferGood;
  }

  function preferGoodChanged(newValue: string) {
    options.preferGood = newValue === 'yes';
  }

  function handleKeydown(event: KeyboardEvent) {
    if (quote === null) {
      return;
    }
    switch (event.key) {
      case 'ArrowLeft':
        vote(quote.id, -1);
        break;
      case 'ArrowRight':
        vote(quote.id, 1);
        break;
      case ' ':
        fetchQuote();
        break;
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />
<main class="bg-slate-900 min-h-screen overflow-hidden flex flex-col">
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
    class="text-slate-200 flex col sm:row justify-center w-full h-full items-center px-4 overflow-hidden mt-8 md:mt-0 flex-1"
  >
    {#if quoteLoading && !quote}
      <p>
        {#if Math.random() < 0.2}
          protip: use the arrow keys to vote
        {:else if Math.random() < 0.2}
          protip: use the spacebar to skip
        {:else if Math.random() < 0.2}
          protip: use the dropdowns to filter
        {:else}
          Loading
        {/if}
      </p>
    {:else if !quoteError && quote}
      <div class="w-[min(800px,90%)] flex flex-col gap-4">
        <div class="flex justify-end gap-2 items-center">
          <a
            href="/"
            aria-label="Go back"
            class="justify-self-start mr-auto justify-center rounded-full p-2 text-sm font-medium text-slate-300  focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 focus:ring-offset-gray-100 transition-all"
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
          <Dropdown
            active={options.preferGood ? 'yes' : 'no'}
            options={{ yes: 'Yes', no: 'No' }}
            title={'Only good?'}
            onChange={preferGoodChanged}
            enabled={!options.preferUnrated}
          />
          <Dropdown
            active={options.preferUnrated ? 'yes' : 'no'}
            options={{ yes: 'Yes', no: 'No' }}
            title={'Prefer unrated?'}
            onChange={preferUnratedChanged}
          />
          <button
            class="px-4 rounded-md transition-all text-sm bg-indigo-500 hover:bg-indigo-600 self-stretch"
            on:click={() => {
              quote = null;
              nextQuote = null;
              fetchQuote();
            }}>Apply</button
          >
          <!-- <Dropdown /> -->
        </div>
        <div class="flex gap-4 sm:flex-row flex-col">
          <QuoteComponent {quote} onVote={vote} compact={false} />
        </div>
        <div class="flex justify-center">
          <button
            class="rounded-full p-2 ring-1 ring-slate-500 hover:ring-2 transition-shadow ease-in-out duration-300"
            on:click={fetchQuote}
            title="Get new quotes"
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
