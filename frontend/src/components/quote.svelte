<script lang="ts">
	import type { Quote } from 'src/models';
	import BlockQuoteRenderer from './block_quote_renderer.svelte';
	import SvelteMarkdown from 'svelte-markdown';

	export let quote: Quote;

	const quoteContent = quote.content.replace('\n', '\n\n').replace('-', '—');
</script>

<div class="bg-slate-800 rounded-xl p-6 flex-1 w-80">
	<h1 class="text-xl">{quote.username}</h1>
	<div class="content">
		<SvelteMarkdown source={quoteContent} renderers={{ blockquote: BlockQuoteRenderer }} />
	</div>
</div>

<style lang="scss">
</style>
