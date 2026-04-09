<script lang="ts">
  let allProps = $props<any>();
  let element = $derived(allProps.element || { props: allProps });
  let label = $derived(element.props?.label || "N/A");
  let value = $derived(element.props?.value || 0);
  let description = $derived(element.props?.description);
</script>

<div class="rounded-xl border border-slate-200 bg-white shadow-sm dark:border-slate-700 dark:bg-slate-800">
  <div class="flex flex-row items-center justify-between space-y-0 p-6 pb-2">
    <h3 class="text-sm font-medium tracking-tight text-slate-600 dark:text-slate-400">{label}</h3>
  </div>
  <div class="p-6 pt-0">
    <div class="text-2xl font-bold text-slate-900 dark:text-slate-50">{value}</div>
    {#if description}
      <p class="mt-1 text-xs text-slate-400 dark:text-slate-500">{description}</p>
    {/if}
  </div>
</div>
