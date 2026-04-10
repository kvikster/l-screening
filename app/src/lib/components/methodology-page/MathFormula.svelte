<script lang="ts">
    let { markup, label }: { markup: string; label?: string } = $props();
</script>

<div class="math-formula overflow-hidden text-slate-900 dark:text-indigo-100" role="img" aria-label={label}>
    {@html markup}
</div>

<style>
    .math-formula {
        -webkit-overflow-scrolling: touch;
    }

    .math-formula :global(math) {
        display: block;
        width: 100%;
        min-width: 0;
        font-size: clamp(0.82rem, 1.3vw, 0.96rem);
        line-height: 1.25;
    }

    .math-formula :global(mtext) {
        font-family: inherit;
        font-size: 0.84em;
    }

    .math-formula :global(mtable) {
        width: 100%;
    }
 </style>
