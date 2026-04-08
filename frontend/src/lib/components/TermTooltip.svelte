<script lang="ts">
    let { term, def }: { term: string; def: string } = $props();
    let open = $state(false);
    let timer: ReturnType<typeof setTimeout>;

    function show() {
        clearTimeout(timer);
        open = true;
    }
    function hide() {
        timer = setTimeout(() => (open = false), 120);
    }
</script>

<span class="relative inline-block">
    <button
        class="cursor-help border-b border-dashed border-blue-400 text-inherit hover:border-blue-600 dark:border-blue-500 dark:hover:border-blue-300"
        onclick={() => (open = !open)}
        onmouseenter={show}
        onmouseleave={hide}
        onblur={hide}
        type="button"
    >{term}</button>
    {#if open}
        <span
            role="tooltip"
            class="absolute bottom-full left-1/2 z-50 mb-2 w-64 -translate-x-1/2 rounded-xl bg-slate-900 px-3 py-2.5 text-xs leading-5 text-slate-100 shadow-xl dark:bg-slate-700"
            onmouseenter={show}
            onmouseleave={hide}
        >
            {def}
            <span class="absolute left-1/2 top-full -translate-x-1/2 border-x-4 border-t-4 border-x-transparent border-t-slate-900 dark:border-t-slate-700"></span>
        </span>
    {/if}
</span>
