<!--
  @file GlossaryTooltip.svelte
  @description
  A lightweight, pure CSS/Svelte tooltip component to wrap glossary terms.
  Provides instant context to domain terminology without requiring the user to navigate to a glossary table.
-->
<script lang="ts">
    /**
     * The term and definition passed as props.
     */
    let { term, definition = "" }: { term: string, definition?: string } = $props();
</script>

<div class="group relative inline-block cursor-help">
    <!-- Anchor text: styled with a subtle dashed underline to indicate interactivity -->
    <span class="inline-flex items-center gap-1 border-b border-dashed border-blue-400/50 font-mono text-blue-700 transition-colors hover:border-blue-600 hover:text-blue-900 dark:border-blue-500/50 dark:text-blue-400 dark:hover:border-blue-300 dark:hover:text-blue-300">
        {term}
        <svg class="h-3 w-3 opacity-40 transition-opacity group-hover:opacity-100" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <circle cx="12" cy="12" r="10" />
            <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3" />
            <path d="M12 17h.01" />
        </svg>
    </span>

    {#if definition}
    <!-- 
      Tooltip Body 
      - Absolutely positioned above the term
      - Invisible by default (opacity-0, scale-95, pointer-events-none)
      - Appears on group hover with a smooth transition
      - Uses glassmorphism (backdrop-blur) for a premium feel
    -->
    <div class="pointer-events-none absolute bottom-full left-1/2 z-[100] mb-2 w-64 -translate-x-1/2 scale-95 opacity-0 transition-all duration-200 ease-out group-hover:scale-100 group-hover:opacity-100">
        <div class="rounded-xl border border-white/20 bg-slate-900/90 p-3 shadow-2xl backdrop-blur-md dark:border-slate-700/50 dark:bg-slate-800/95">
            <p class="mb-1 font-mono text-[10px] font-bold uppercase tracking-widest text-blue-400">{term}</p>
            <p class="text-xs leading-relaxed text-slate-100">{definition}</p>
        </div>
        <!-- Little triangle pointer pointing down -->
        <div class="absolute left-1/2 top-full -mt-[1px] h-2 w-2 -translate-x-1/2 rotate-45 border-b border-r border-white/20 bg-slate-900/90 dark:border-slate-700/50 dark:bg-slate-800/95"></div>
    </div>
    {/if}
</div>
