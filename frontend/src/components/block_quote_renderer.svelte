<script>
	import DoubleQuoteSvg from './double_quote_svg.svelte';
</script>

<blockquote class="my-3">
	<div class="fill-slate-300 " aria-hidden="true">
		<DoubleQuoteSvg />
	</div>
	<div class="p-4 text-justify break-words">
		<slot />
	</div>
	<div class="fill-slate-300 transform rotate-180" aria-hidden="true">
		<DoubleQuoteSvg />
	</div>
</blockquote>
