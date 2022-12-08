<script lang="ts">
	import { onMount } from 'svelte';
	import { scale } from 'svelte/transition';
	let show = false;
	let menu: HTMLDivElement | null = null;

	export let links: {
		href: string;
		label: string;
	}[];

	onMount(() => {
		const handleOutsideClick = (event: any) => {
			if (show && menu && !menu.contains(event.target)) {
				show = false;
			}
		};

		const handleEscape = (event: { key: string }) => {
			if (show && event.key === 'Escape') {
				show = false;
			}
		};

		// add events when element is added to the DOM
		document.addEventListener('click', handleOutsideClick, false);
		document.addEventListener('keyup', handleEscape, false);

		// remove events when element is removed from the DOM
		return () => {
			document.removeEventListener('click', handleOutsideClick, false);
			document.removeEventListener('keyup', handleEscape, false);
		};
	});
</script>

<div class="relative inline-block text-left" bind:this={menu}>
	<div>
		<button
			type="button"
			class="inline-flex w-full justify-center rounded-full bg-slate-800 p-1 text-sm font-medium text-slate-300  hover:bg-slate-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 focus:ring-offset-gray-100 transition-all"
			id="menu-button"
			aria-expanded="true"
			aria-haspopup="true"
			on:click={() => (show = !show)}
		>
			<svg
				xmlns="http://www.w3.org/2000/svg"
				fill="none"
				viewBox="0 0 24 24"
				stroke-width="1.5"
				stroke="currentColor"
				class="w-6 h-6"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					d="M12 6.75a.75.75 0 110-1.5.75.75 0 010 1.5zM12 12.75a.75.75 0 110-1.5.75.75 0 010 1.5zM12 18.75a.75.75 0 110-1.5.75.75 0 010 1.5z"
				/>
			</svg>
		</button>
	</div>

	<!--
    Dropdown menu, show/hide based on menu state.

    Entering: "transition ease-out duration-100"
      From: "transform opacity-0 scale-95"
      To: "transform opacity-100 scale-100"
    Leaving: "transition ease-in duration-75"
      From: "transform opacity-100 scale-100"
      To: "transform opacity-0 scale-95"
  -->
	{#if show}
		<div
			in:scale={{ duration: 100, start: 0.95 }}
			out:scale={{ duration: 75, start: 0.95 }}
			class="absolute right-0 z-10 mt-2 w-56 origin-top-right rounded-md bg-slate-800 shadow-lg shadow-slate-900 ring-1 ring-black ring-opacity-5 focus:outline-none"
			role="menu"
			aria-orientation="vertical"
			aria-labelledby="menu-button"
			tabindex="-1"
		>
			<div class="py-1" role="none">
				{#each links as item}
					<!-- Active: "bg-gray-100 text-gray-900", Not Active: "text-gray-700" -->
					<a
						href={item.href}
						class="text-slate-300 bg-slate-800 block px-4 py-2 text-sm w-full text-left hover:bg-slate-700 transition-all"
						role="menuitem"
						tabindex="-1"
						target="_blank"
						id="menu-item-0"
						>{item.label}
					</a>
				{/each}
			</div>
		</div>
	{/if}
</div>
