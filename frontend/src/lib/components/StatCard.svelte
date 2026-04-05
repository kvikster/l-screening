<script lang="ts">
  // In Svelte 5, @json-render/svelte passes the whole element
  let allProps = $props<any>();
  
  // Handle both Renderer (element.props) and Manual ({ ...props }) cases
  let element = $derived(allProps.element || { props: allProps });
  let label = $derived(element.props?.label || "N/A");
  let value = $derived(element.props?.value || 0);
  let description = $derived(element.props?.description);
</script>

<div class="rounded-xl border bg-card text-card-foreground shadow">
  <div class="p-6 flex flex-row items-center justify-between space-y-0 pb-2">
    <h3 class="tracking-tight text-sm font-medium">{label}</h3>
  </div>
  <div class="p-6 pt-0">
    <div class="text-2xl font-bold">{value}</div>
    {#if description}
      <p class="text-xs text-muted-foreground mt-1">{description}</p>
    {/if}
  </div>
</div>
