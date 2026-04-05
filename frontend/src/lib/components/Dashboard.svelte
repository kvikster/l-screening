<script lang="ts">
  import { type DashboardProps } from "../json-render/catalog";
  import StatCard from "./StatCard.svelte";
  import { FileText, Database, ShieldAlert, CheckCircle2 } from "lucide-svelte";
  
  // In Svelte 5, @json-render/svelte passes the whole element
  let allProps = $props<any>();
  
  // Handle both Renderer (element.props) and Manual ({ ...props }) cases
  let element = $derived(allProps.element || { props: allProps });
  let title = $derived(element.props?.title || "No Title");
  let summary = $derived(element.props?.summary || []);
  let peaks = $derived(element.props?.peaks || []);
  
  let totalCompounds = $derived(summary.reduce((acc: number, s: any) => acc + (s.RealCompounds || 0), 0));
  let totalArtifacts = $derived(summary.reduce((acc: number, s: any) => acc + (s.Artifacts || 0), 0));
</script>

<div class="space-y-8 p-8 max-w-7xl mx-auto">
  <div class="flex items-center justify-between">
    <h2 class="text-3xl font-bold tracking-tight">{title}</h2>
  </div>

  <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
    <StatCard label="Total Real Compounds" value={totalCompounds} />
    <StatCard label="Total Artifacts" value={totalArtifacts} />
    <StatCard label="Total Peaks" value={summary.reduce((acc: number, s: any) => acc + (s.TotalPeaks || 0), 0)} />
    <StatCard label="Confirmed Replicates" value={summary.reduce((acc: number, s: any) => acc + (s.Confirmed || 0), 0)} />
  </div>

  <div class="rounded-xl border bg-card text-card-foreground shadow-sm">
    <div class="p-6">
      <h3 class="text-lg font-semibold leading-none tracking-tight mb-4">Screened Peaks</h3>
      <div class="relative w-full overflow-auto">
        <table class="w-full caption-bottom text-sm">
          <thead>
            <tr class="border-b transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted">
              <th class="h-10 px-2 text-left font-medium text-muted-foreground">RT (mean)</th>
              <th class="h-10 px-2 text-left font-medium text-muted-foreground">MZ (mean)</th>
              <th class="h-10 px-2 text-left font-medium text-muted-foreground">Area (avg)</th>
              <th class="h-10 px-2 text-left font-medium text-muted-foreground">Sample</th>
              <th class="h-10 px-2 text-left font-medium text-muted-foreground">Operator Marks</th>
              <th class="h-10 px-2 text-left font-medium text-muted-foreground">Status</th>
              <th class="h-10 px-2 text-left font-medium text-muted-foreground">Why?</th>
            </tr>
          </thead>
          <tbody class="[&_tr:last-child]:border-0">
            {#each peaks as peak}
              <tr class="border-b transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted">
                <td class="p-2 font-mono">{peak.RT_mean.toFixed(3)}</td>
                <td class="p-2 font-mono">{peak.MZ_mean.toFixed(2)}</td>
                <td class="p-2">{peak.Area_mean.toLocaleString()}</td>
                <td class="p-2">
                  <div class="flex flex-col">
                    <span class="capitalize font-semibold">{peak.SampleType}</span>
                    <span class="text-xs text-muted-foreground uppercase">{peak.Polarity}</span>
                  </div>
                </td>
                <td class="p-2">
                  <div class="flex items-center gap-2">
                    <div class="flex flex-col items-center">
                      <div 
                        class="w-4 h-4 rounded-full border border-black/10 shadow-sm" 
                        style="background-color: #{peak.Rep1_Color || 'e2e8f0'}"
                        title="Rep 1: {peak.Rep1_Mark || 'No Color'}"
                      ></div>
                      <span class="text-[10px] text-muted-foreground mt-0.5">R1</span>
                    </div>
                    <div class="flex flex-col items-center">
                      <div 
                        class="w-4 h-4 rounded-full border border-black/10 shadow-sm" 
                        style="background-color: #{peak.Rep2_Color || 'e2e8f0'}"
                        title="Rep 2: {peak.Rep2_Mark || 'No Color'}"
                      ></div>
                      <span class="text-[10px] text-muted-foreground mt-0.5">R2</span>
                    </div>
                  </div>
                </td>
                <td class="p-2">
                  <span class="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-semibold
                    {peak.Status === 'Real Compound' ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'}">
                    {#if peak.Status === 'Real Compound'}
                      <CheckCircle2 class="mr-1 h-3 w-3" />
                    {:else}
                      <ShieldAlert class="mr-1 h-3 w-3" />
                    {/if}
                    {peak.Status}
                  </span>
                </td>
                <td class="p-2">
                  <button 
                    class="text-blue-500 hover:underline text-xs bg-blue-50/50 px-2 py-1 rounded"
                    onclick={() => alert(JSON.stringify(peak.Why, null, 2))}
                  >
                    Logic Detail
                  </button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>
  </div>
</div>
