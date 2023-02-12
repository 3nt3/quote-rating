<script lang="ts">
  export let title: string;
  let show = false;
</script>

<div on:mouseenter={() => (show = true)} on:mouseleave={() => (show = false)} class="relative">
  <slot />
  <div
    class="transition-opacity absolute bg-gray-300 border-gray-400 border-2 z-10 text-slate-800 text-center left-0 right-0 w-auto top-[120%] px-4 py-0 rounded-md shadow-md"
    class:opacity-0={!show}
    class:opacity-100={show}
  >
    <span class="">{title}</span>
  </div>
</div>
