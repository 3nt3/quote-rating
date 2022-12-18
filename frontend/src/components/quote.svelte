<script lang="ts">
    import type { Quote } from 'src/models';
    import BlockQuoteRenderer from './block_quote_renderer.svelte';
    import SvelteMarkdown from 'svelte-markdown';
    import ListItemRenderer from './list_item_renderer.svelte';
    import ThreeDotsDropdown from './three_dots_dropdown.svelte';
    import CompactBlockQuoteRenderer from './compact_block_quote_renderer.svelte';

    export let quote: Quote;
    export let onVote: Function;
    export let compact: boolean;

    const quoteContent = quote.content
        .replace('\n', '\n')
        .replace(/^(-|--|–)/, '—')
        .replace('\n—', '\n\n—');
</script>

{#if !compact}
    <div
        class="bg-slate-800 rounded-xl p-6 flex-1 sm:w-80 flex flex-col gap-4 shadow-lg border-2 border-slate-700"
    >
        <div class="flex items-center justify-between">
            <div class="flex items-center gap-4">
                <img
                    class="rounded-full h-8"
                    src={quote.avatar_url}
                    alt={`${quote.username}'s avatar`}
                />
                <h1 class="">{quote.username}</h1>
            </div>
            <ThreeDotsDropdown
                links={[{ href: quote.message_link, label: 'View message in discord' }]}
            />
        </div>
        <!-- content -->
        <div class="">
            <SvelteMarkdown
                source={quoteContent}
                renderers={{ blockquote: BlockQuoteRenderer, listitem: ListItemRenderer }}
            />
        </div>
        <!-- actions -->
        <div class="flex justify-center gap-4 mt-auto px-2 items-center">
            <button
                on:click={() => onVote(quote.id, -1)}
                class="stroke-red-400 rounded-full p-2 ring-1 ring-red-400 hover:ring-2 transition-shadow ease-in-out duration-300"
                title="Downvote"
            >
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke-width="1.5"
                    class="w-6 h-6"
                >
                    <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                </svg>
            </button>
            <span class="bg-slate-700 px-3 rounded-full">{quote.score}</span>
            <button
                on:click={() => onVote(quote.id, +1)}
                class="stroke-teal-500 rounded-full p-2 ring-1 ring-teal-500 hover:ring-2 transition-shadow ease-in-out duration-300"
                title="Upvote"
            >
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke-width="1.5"
                    class="w-6 h-6"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        d="M21 8.25c0-2.485-2.099-4.5-4.688-4.5-1.935 0-3.597 1.126-4.312 2.733-.715-1.607-2.377-2.733-4.313-2.733C5.1 3.75 3 5.765 3 8.25c0 7.22 9 12 9 12s9-4.78 9-12z"
                    />
                </svg>
            </button>
        </div>
    </div>
{:else}
    <div class="bg-slate-800 shadow-lg p-4 rounded-md border-2 border-slate-700">
        <div class="flex flex-col sm:flex-row justify-between sm:items-center gap-4">
            <div>
                <div class="flex items-center gap-4">
                    <img
                        class="rounded-full h-8"
                        src={quote.avatar_url}
                        alt={`${quote.username}'s avatar`}
                    />
                    <h1 class="">{quote.username}</h1>
                </div>
                <!-- content -->
                <div class="">
                    <SvelteMarkdown
                        source={quoteContent}
                        renderers={{
                            blockquote: CompactBlockQuoteRenderer,
                            listitem: ListItemRenderer
                        }}
                    />
                </div>
            </div>
            <!-- actions -->
            <div class="flex justify-center gap-4 mt-auto px-2 items-center">
                <button
                    on:click={() => onVote(quote.id, -1)}
                    class="stroke-red-400 rounded-full p-2 ring-1 ring-red-400 hover:ring-2 transition-shadow ease-in-out duration-300"
                    title="Downvote"
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke-width="1.5"
                        class="w-6 h-6"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M6 18L18 6M6 6l12 12"
                        />
                    </svg>
                </button>
                <span class="bg-slate-700 px-3 rounded-full">{quote.score}</span>
                <button
                    on:click={() => onVote(quote.id, +1)}
                    class="stroke-teal-500 rounded-full p-2 ring-1 ring-teal-500 hover:ring-2 transition-shadow ease-in-out duration-300"
                    title="Upvote"
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke-width="1.5"
                        class="w-6 h-6"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M21 8.25c0-2.485-2.099-4.5-4.688-4.5-1.935 0-3.597 1.126-4.312 2.733-.715-1.607-2.377-2.733-4.313-2.733C5.1 3.75 3 5.765 3 8.25c0 7.22 9 12 9 12s9-4.78 9-12z"
                        />
                    </svg>
                </button>
            </div>
        </div>
    </div>
{/if}
