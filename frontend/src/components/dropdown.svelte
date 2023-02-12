<script lang="ts">
    import { onMount } from 'svelte';
    import { scale } from 'svelte/transition';
    let show = false;
    let menu: HTMLDivElement | null = null;

    export let active: string;
    export let options: { [key: string]: string };
    export let title: string;
    export let onChange: Function;
    export let enabled: boolean =true;

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

    const optionsArray: { i: number; id: string; label: string }[] = Object.keys(options).map(
        (x, i) => ({ i, id: x, label: options[x] })
    );
</script>

<div class="relative inline-block text-left" bind:this={menu}>
    <div>
        <button
            type="button"
            class="inline-flex w-full justify-center rounded-md border border-slate-700 bg-slate-800 px-4 py-2 text-xs sm:text-sm font-medium text-slate-300 shadow-sm enabled:hover:bg-slate-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 focus:ring-offset-gray-100 transition-all"
            class:opacity-50={!enabled}
            id="menu-button"
            aria-expanded="true"
            aria-haspopup="true"
            disabled={!enabled}
            on:click={() => (show = !show)}
        >
            {title}: {options[active]}
            <!-- Heroicon name: mini/chevron-down -->
            <svg
                class="-mr-1 ml-2 h-5 w-5"
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 20 20"
                fill="currentColor"
                aria-hidden="true"
            >
                <path
                    fill-rule="evenodd"
                    d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z"
                    clip-rule="evenodd"
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
            class="absolute right-0 z-100 mt-2 w-56 origin-top-right rounded-md bg-slate-800 shadow-lg shadow-slate-900 ring-1 ring-black ring-opacity-5 focus:outline-none"
            role="menu"
            aria-orientation="vertical"
            aria-labelledby="menu-button"
            tabindex="-1"
        >
            <div class="py-1" role="none">
                {#each optionsArray as item}
                    <!-- Active: "bg-gray-100 text-gray-900", Not Active: "text-gray-700" -->
                    <button
                        on:click={() => {
                            onChange(item.id);
                            show = false;
                        }}
                        class="text-slate-300 bg-slate-800 block px-4 py-2 text-sm w-full text-left hover:bg-slate-700 transition-all"
                        class:font-bold={active === item.id}
                        role="menuitem"
                        tabindex="-1"
                        id="menu-item-0">{item.label}</button
                    >
                {/each}
            </div>
        </div>
    {/if}
</div>
