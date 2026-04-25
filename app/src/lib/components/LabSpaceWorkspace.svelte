<script lang="ts">
  import { onMount } from "svelte";
  import { Loader2 } from "lucide-svelte";
  import { dictionary, t } from "$lib/i18n";
  import {
    createDefaultSurrogateSpec,
    createEmptyParsedRow,
    defaultAnalyzerPreset,
    labSpaceRepository,
    validateSurrogateSpecs,
  } from "$lib/labspace/repository";
  import {
    composeLabAnalysisRun,
    resolveSelectedEntities,
    retagDatasetRowsForKind,
    validateDatasetForKind,
  } from "$lib/labspace/runtime";
  import { parseInputFile, screenRows } from "$lib/screening";
  import type {
    AnalyzerPreset,
    LabSpaceState,
    StoredDataset,
    StoredReport,
    StoredSurrogateSpec,
  } from "$lib/labspace/types";

  type AnalyzerDraft = {
    id: string | null;
    name: string;
    syncBlankWithReplicate: boolean;
    replicate_rt_tol: number;
    replicate_mz_tol: number;
    replicate_mz_mode: string;
    blank_rt_tol: number;
    blank_mz_tol: number;
    blank_mz_mode: string;
    signal_to_blank_min: number;
    cv_high_max: number;
    cv_moderate_max: number;
    min_area_difference: string;
  };

  let { onOpenReport } = $props<{ onOpenReport: (report: StoredReport) => void }>();

  let loading = $state(true);
  let busy = $state(false);
  let error = $state("");
  let success = $state("");
  let showWorkspaceModal = $state(false);
  let showDatasetEditorModal = $state(false);
  let showAnalyzerConfig = $state(false);
  let labState: LabSpaceState | null = $state(null);
  let analyzerDraft: AnalyzerDraft = $state(makeAnalyzerDraft(defaultAnalyzerPreset()));
  let importKind: StoredDataset["kind"] = $state("analysis");
  let dict = $derived($dictionary);

  const datasetKinds: StoredDataset["kind"][] = ["analysis", "blank", "surrogate_observed"];

  function makeAnalyzerDraft(preset: AnalyzerPreset): AnalyzerDraft {
    return {
      id: preset.id,
      name: preset.name,
      syncBlankWithReplicate: preset.syncBlankWithReplicate,
      replicate_rt_tol: preset.params.replicate_rt_tol,
      replicate_mz_tol: preset.params.replicate_mz_tol,
      replicate_mz_mode: preset.params.replicate_mz_mode,
      blank_rt_tol: preset.params.blank_rt_tol,
      blank_mz_tol: preset.params.blank_mz_tol,
      blank_mz_mode: preset.params.blank_mz_mode,
      signal_to_blank_min: preset.params.signal_to_blank_min,
      cv_high_max: preset.params.cv_high_max ?? 15,
      cv_moderate_max: preset.params.cv_moderate_max ?? 30,
      min_area_difference:
        preset.params.min_area_difference == null ? "" : String(preset.params.min_area_difference),
    };
  }

  function activeAnalyzer(state: LabSpaceState): AnalyzerPreset {
    return (
      state.analyzers.find((item) => item.id === state.meta.activeAnalyzerId) ??
      state.analyzers[0] ??
      defaultAnalyzerPreset()
    );
  }

  async function reloadState() {
    const next = await labSpaceRepository.loadState();
    labState = next;
    analyzerDraft = makeAnalyzerDraft(activeAnalyzer(next));
  }

  onMount(async () => {
    try {
      await reloadState();
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      loading = false;
    }
  });

  function resetFeedback() {
    error = "";
    success = "";
  }

  function selectedDataset(): StoredDataset | null {
    const state = labState;
    if (!state?.meta.selectedDatasetId) return null;
    return state.datasets.find((item) => item.id === state.meta.selectedDatasetId) ?? null;
  }

  function datasetTypeLabel(kind: StoredDataset["kind"]): string {
    if (kind === "blank") return dict.labspaceDatasetTypeBlank;
    if (kind === "surrogate_observed") return dict.labspaceDatasetTypeSurrogate;
    return dict.labspaceDatasetTypeNormal;
  }

  function datasetTypeOptions() {
    return datasetKinds.map((value) => ({
      value,
      label: datasetTypeLabel(value),
    }));
  }

  function datasetsByKind(kind: StoredDataset["kind"]): StoredDataset[] {
    return labState?.datasets.filter((item) => item.kind === kind) ?? [];
  }

  function buildDatasetSelectionPatch(
    state: LabSpaceState,
    datasetId: string,
    previousKind: StoredDataset["kind"] | null,
    nextKind: StoredDataset["kind"],
  ): Partial<LabSpaceState["meta"]> {
    const patch: Partial<LabSpaceState["meta"]> = {
      selectedDatasetId: datasetId,
    };

    if (previousKind === "analysis" && state.meta.selectedAnalysisDatasetId === datasetId) {
      patch.selectedAnalysisDatasetId = null;
    }
    if (previousKind === "blank" && state.meta.selectedBlankDatasetId === datasetId) {
      patch.selectedBlankDatasetId = null;
    }
    if (
      previousKind === "surrogate_observed" &&
      state.meta.selectedSurrogateDatasetId === datasetId
    ) {
      patch.selectedSurrogateDatasetId = null;
    }

    if (nextKind === "analysis" && !state.meta.selectedAnalysisDatasetId) {
      patch.selectedAnalysisDatasetId = datasetId;
    }
    if (nextKind === "blank" && !state.meta.selectedBlankDatasetId) {
      patch.selectedBlankDatasetId = datasetId;
    }
    if (nextKind === "surrogate_observed" && !state.meta.selectedSurrogateDatasetId) {
      patch.selectedSurrogateDatasetId = datasetId;
    }

    return patch;
  }

  function mergeMetaPatch(patch: Partial<LabSpaceState["meta"]>) {
    if (!labState) return;
    labState.meta = {
      ...labState.meta,
      ...patch,
    };
  }

  function patchForUsingDataset(dataset: StoredDataset): Partial<LabSpaceState["meta"]> {
    if (dataset.kind === "analysis") {
      return { selectedAnalysisDatasetId: dataset.id };
    }
    if (dataset.kind === "blank") {
      return { selectedBlankDatasetId: dataset.id };
    }
    return { selectedSurrogateDatasetId: dataset.id };
  }

  async function persistSelection(patch: Partial<LabSpaceState["meta"]>) {
    if (!labState) return;
    labState.meta = await labSpaceRepository.saveMeta({
      ...labState.meta,
      ...patch,
    });
  }

  async function saveMeta() {
    if (!labState) return;
    resetFeedback();
    busy = true;
    try {
      labState.meta = await labSpaceRepository.saveMeta(labState.meta);
      success = t("labspaceMetaSaved");
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      busy = false;
    }
  }

  async function handleDatasetImport(kind: StoredDataset["kind"], event: Event) {
    const target = event.currentTarget as HTMLInputElement;
    const file = target.files?.[0];
    if (!file) return;

    resetFeedback();
    busy = true;
    try {
      const parsedRows = await parseInputFile(file);
      const rows = kind === "analysis" ? parsedRows : retagDatasetRowsForKind(parsedRows, kind);
      const validationError = validateDatasetForKind(kind, rows);
      if (validationError) {
        throw new Error(validationError);
      }

      const dataset = await labSpaceRepository.upsertDataset({
        kind,
        name: file.name.replace(/\.[^.]+$/, ""),
        sourceName: file.name,
        rows,
      });
      await reloadState();
      if (!labState) return;

      const patch = buildDatasetSelectionPatch(labState, dataset.id, null, kind);
      await persistSelection(patch);
      success = t("labspaceDatasetImportedAs", {
        name: file.name,
        type: datasetTypeLabel(kind).toLowerCase(),
      });
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      target.value = "";
      busy = false;
    }
  }

  async function saveDataset(dataset: StoredDataset) {
    if (!labState) return;
    resetFeedback();
    busy = true;
    try {
      const validationError = validateDatasetForKind(dataset.kind, dataset.rows);
      if (validationError) {
        throw new Error(validationError);
      }
      const savedDataset = await labSpaceRepository.upsertDataset(dataset);
      labState.meta = await labSpaceRepository.saveMeta({
        ...labState.meta,
        selectedDatasetId: savedDataset.id,
      });
      await reloadState();
      success = t("labspaceDatasetSaved", { name: savedDataset.name });
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      busy = false;
    }
  }

  async function saveSelectedDataset() {
    const dataset = selectedDataset();
    if (!dataset) return;
    await saveDataset(dataset);
  }

  async function openDatasetEditor(datasetId: string) {
    await persistSelection({ selectedDatasetId: datasetId });
    showDatasetEditorModal = true;
  }

  function closeDatasetEditor() {
    showDatasetEditorModal = false;
  }

  function updateSelectedDatasetField(field: keyof StoredDataset, value: string) {
    if (!labState) return;
    const dataset = selectedDataset();
    if (!dataset) return;
    labState.datasets = labState.datasets.map((item) =>
      item.id === dataset.id ? { ...item, [field]: value } : item,
    );
  }

  function updateDatasetKind(datasetId: string, nextKind: StoredDataset["kind"]) {
    if (!labState) return;
    const dataset = labState.datasets.find((item) => item.id === datasetId);
    if (!dataset || dataset.kind === nextKind) return;

    const patch = buildDatasetSelectionPatch(labState, datasetId, dataset.kind, nextKind);
    labState.datasets = labState.datasets.map((item) =>
      item.id === datasetId
        ? {
            ...item,
            kind: nextKind,
            rows: retagDatasetRowsForKind(item.rows, nextKind),
          }
        : item,
    );
    mergeMetaPatch(patch);
  }

  function updateSelectedDatasetRow(
    rowIndex: number,
    field: keyof StoredDataset["rows"][number],
    value: string,
  ) {
    if (!labState) return;
    const dataset = selectedDataset();
    if (!dataset) return;

    labState.datasets = labState.datasets.map((item) => {
      if (item.id !== dataset.id) return item;

      const rows = item.rows.map((row, index) => {
        if (index !== rowIndex) return row;

        if (field === "RT" || field === "Area" || field === "Base Peak") {
          const parsed = value === "" ? null : Number(value);
          return {
            ...row,
            [field]:
              field === "Base Peak"
                ? parsed == null || Number.isNaN(parsed)
                  ? null
                  : parsed
                : Number.isNaN(parsed)
                  ? 0
                  : parsed,
          };
        }

        if (field === "Polarity") {
          const nextMark =
            row.operator_mark === "blank_positive" || row.operator_mark === "blank_negative"
              ? /neg/i.test(value)
                ? "blank_negative"
                : "blank_positive"
              : row.operator_mark;
          return {
            ...row,
            Polarity: value,
            operator_mark: nextMark,
          };
        }

        return {
          ...row,
          [field]: value,
        };
      });

      return {
        ...item,
        rows,
        rowCount: rows.length,
      };
    });
  }

  function getRowRoleValue(
    row: StoredDataset["rows"][number],
  ): "auto" | "blank" | "surrogate" {
    if (row.operator_mark === "blank_positive" || row.operator_mark === "blank_negative") {
      return "blank";
    }
    if (
      row.operator_mark === "surrogate" ||
      row.operator_mark === "surrogate_positive" ||
      row.operator_mark === "surrogate_negative"
    ) {
      return "surrogate";
    }
    return "auto";
  }

  function updateSelectedDatasetRowRole(
    rowIndex: number,
    role: "auto" | "blank" | "surrogate",
  ) {
    if (!labState) return;
    const dataset = selectedDataset();
    if (!dataset || dataset.kind !== "analysis") return;

    labState.datasets = labState.datasets.map((item) => {
      if (item.id !== dataset.id) return item;
      const rows = item.rows.map((row, index) => {
        if (index !== rowIndex) return row;
        if (role === "auto") {
          return { ...row, operator_mark: null, operator_color: null };
        }
        if (role === "surrogate") {
          return { ...row, operator_mark: "surrogate", operator_color: null };
        }
        return {
          ...row,
          operator_mark: /neg/i.test(row.Polarity) ? "blank_negative" : "blank_positive",
          operator_color: null,
        };
      });
      return {
        ...item,
        rows,
      };
    });
  }

  function addRowToSelectedDataset() {
    if (!labState) return;
    const dataset = selectedDataset();
    if (!dataset) return;
    labState.datasets = labState.datasets.map((item) =>
      item.id === dataset.id
        ? {
            ...item,
            rows: [...item.rows, createEmptyParsedRow()],
            rowCount: item.rows.length + 1,
          }
        : item,
    );
  }

  function removeRowFromSelectedDataset(rowIndex: number) {
    if (!labState) return;
    const dataset = selectedDataset();
    if (!dataset) return;
    labState.datasets = labState.datasets.map((item) => {
      if (item.id !== dataset.id) return item;
      const rows = item.rows.filter((_, index) => index !== rowIndex);
      return {
        ...item,
        rows,
        rowCount: rows.length,
      };
    });
  }

  async function deleteDataset(datasetId: string) {
    if (!labState) return;
    resetFeedback();
    busy = true;
    try {
      await labSpaceRepository.deleteDataset(datasetId);
      const nextMeta = {
        ...labState.meta,
        selectedDatasetId:
          labState.meta.selectedDatasetId === datasetId ? null : labState.meta.selectedDatasetId,
        selectedAnalysisDatasetId:
          labState.meta.selectedAnalysisDatasetId === datasetId
            ? null
            : labState.meta.selectedAnalysisDatasetId,
        selectedBlankDatasetId:
          labState.meta.selectedBlankDatasetId === datasetId
            ? null
            : labState.meta.selectedBlankDatasetId,
        selectedSurrogateDatasetId:
          labState.meta.selectedSurrogateDatasetId === datasetId
            ? null
            : labState.meta.selectedSurrogateDatasetId,
      };
      await labSpaceRepository.saveMeta(nextMeta);
      await reloadState();
      success = t("labspaceDatasetDeleted");
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      busy = false;
    }
  }

  function syncBlankWithReplicateDraft() {
    if (!analyzerDraft.syncBlankWithReplicate) return;
    analyzerDraft.blank_rt_tol = analyzerDraft.replicate_rt_tol;
    analyzerDraft.blank_mz_tol = analyzerDraft.replicate_mz_tol;
    analyzerDraft.blank_mz_mode = analyzerDraft.replicate_mz_mode;
  }

  async function saveAnalyzer() {
    if (!labState) return;
    resetFeedback();
    busy = true;
    try {
      const currentAnalyzer = activeAnalyzer(labState);
      const analyzer = await labSpaceRepository.upsertAnalyzer({
        id: currentAnalyzer.id,
        name: analyzerDraft.name.trim() || t("labspaceDefaultAnalyzer"),
        syncBlankWithReplicate: analyzerDraft.syncBlankWithReplicate,
        params: {
          replicate_rt_tol: Number(analyzerDraft.replicate_rt_tol),
          replicate_mz_tol: Number(analyzerDraft.replicate_mz_tol),
          replicate_mz_mode: analyzerDraft.replicate_mz_mode,
          blank_rt_tol: Number(analyzerDraft.blank_rt_tol),
          blank_mz_tol: Number(analyzerDraft.blank_mz_tol),
          blank_mz_mode: analyzerDraft.blank_mz_mode,
          signal_to_blank_min: Number(analyzerDraft.signal_to_blank_min),
          cv_high_max: Number(analyzerDraft.cv_high_max),
          cv_moderate_max: Number(analyzerDraft.cv_moderate_max),
          min_area_difference:
            analyzerDraft.min_area_difference.trim() === ""
              ? undefined
              : Number(analyzerDraft.min_area_difference),
        },
      });
      await reloadState();
      await persistSelection({ activeAnalyzerId: analyzer.id });
      success = t("labspaceConfigSaved");
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      busy = false;
    }
  }

  function updateSurrogateSpec(
    specId: string,
    field: keyof StoredSurrogateSpec,
    value: string,
  ) {
    if (!labState) return;
    labState.surrogateSpecs = labState.surrogateSpecs.map((spec) => {
      if (spec.id !== specId) return spec;
      if (field === "name") {
        return { ...spec, name: value };
      }
      const parsed = value.trim() === "" ? undefined : Number(value);
      return {
        ...spec,
        [field]: parsed == null || Number.isNaN(parsed) ? undefined : parsed,
      };
    });
  }

  function addSurrogateSpec() {
    if (!labState) return;
    labState.surrogateSpecs = [...labState.surrogateSpecs, createDefaultSurrogateSpec()];
  }

  function removeSurrogateSpec(specId: string) {
    if (!labState) return;
    labState.surrogateSpecs = labState.surrogateSpecs.filter((spec) => spec.id !== specId);
  }

  async function saveSurrogateSpecs() {
    if (!labState) return;
    resetFeedback();
    busy = true;
    try {
      const validationError = validateSurrogateSpecs(labState.surrogateSpecs);
      if (validationError) {
        throw new Error(validationError);
      }
      await labSpaceRepository.replaceSurrogateSpecs(labState.surrogateSpecs);
      await reloadState();
      success = t("labspaceSurrogateSpecsSaved");
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      busy = false;
    }
  }

  async function runAnalysis() {
    if (!labState) return;
    resetFeedback();
    busy = true;
    try {
      const resolved = resolveSelectedEntities(labState);
      const composed = composeLabAnalysisRun({
        analysisDataset: resolved.analysisDataset,
        blankDataset: resolved.blankDataset,
        surrogateDataset: resolved.surrogateDataset,
        analyzer: resolved.analyzer,
        surrogateSpecs: labState.surrogateSpecs,
      });
      const result = await screenRows(
        composed.rows,
        composed.runtimeParams,
        `${composed.analysisDataset.name}.labspace`,
      );
      const report = await labSpaceRepository.createReport({
        title: composed.reportTitle,
        datasetRefs: {
          analysisDatasetId: composed.analysisDataset.id,
          blankDatasetId: composed.blankDataset?.id ?? null,
          surrogateDatasetId: composed.surrogateDataset?.id ?? null,
        },
        analyzerId: composed.analyzer.id,
        runtimeParams: composed.runtimeParams,
        resultSnapshot: result,
      });
      await reloadState();
      await persistSelection({ selectedReportId: report.id });
      onOpenReport(report);
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      busy = false;
    }
  }

  async function openReport(report: StoredReport) {
    if (!labState) return;
    await persistSelection({ selectedReportId: report.id });
    onOpenReport(report);
  }

  async function deleteReport(reportId: string) {
    if (!labState) return;
    resetFeedback();
    busy = true;
    try {
      await labSpaceRepository.deleteReport(reportId);
      await reloadState();
      await persistSelection({
        selectedReportId:
          labState.meta.selectedReportId === reportId ? null : labState.meta.selectedReportId,
      });
      success = t("labspaceReportDeleted");
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      busy = false;
    }
  }
</script>

{#if loading}
  <div class="flex min-h-[40vh] items-center justify-center gap-3 text-slate-500 dark:text-slate-300">
    <Loader2 class="h-5 w-5 animate-spin" />
    <span>{dict.labspaceLoading}</span>
  </div>
{:else if !labState}
  <div class="rounded-2xl border border-red-200 bg-red-50 p-4 text-sm text-red-700 dark:border-red-900 dark:bg-red-950 dark:text-red-300">
    {dict.labspaceLoadFailed}
  </div>
{:else}
  <div class="mx-auto grid max-w-7xl gap-4 px-1 sm:gap-5 sm:px-0">
    <section class="max-w-5xl rounded-[1.5rem] border border-slate-200 bg-white p-4 shadow-sm dark:border-slate-700 dark:bg-slate-900 sm:rounded-[1.75rem]">
      <div class="flex flex-col gap-4 xl:flex-row xl:items-end xl:justify-between">
        <div class="max-w-2xl">
          <div class="flex flex-wrap items-center gap-2">
            <p class="text-xs font-semibold uppercase tracking-[0.18em] text-slate-400 dark:text-slate-500">{dict.labspaceRunEyebrow}</p>
            <button
              type="button"
              class="rounded-full border border-slate-200 px-3 py-1 text-xs font-medium text-slate-600 hover:bg-slate-50 dark:border-slate-600 dark:text-slate-300 dark:hover:bg-slate-800"
              onclick={() => (showWorkspaceModal = true)}
            >
              {dict.labspaceWorkspaceSettings}
            </button>
          </div>
          <h1 class="mt-2 text-xl font-semibold text-slate-900 dark:text-slate-50 sm:text-2xl">{dict.labspaceRunTitle}</h1>
          <p class="mt-1 text-sm text-slate-500 dark:text-slate-400">
            {dict.labspaceRunDesc}
          </p>
        </div>
        <button
          type="button"
          class="w-full rounded-full bg-slate-900 px-5 py-2.5 text-sm font-medium text-white hover:bg-slate-700 dark:bg-slate-100 dark:text-slate-900 sm:w-auto"
          onclick={runAnalysis}
          disabled={busy}
        >
          {dict.labspaceRunAction}
        </button>
      </div>

      <div class="mt-4 grid max-w-4xl gap-3 sm:grid-cols-2 xl:grid-cols-3">
        <label class="space-y-1 text-sm">
          <span class="font-medium text-slate-700 dark:text-slate-200">{dict.labspaceAnalysis}</span>
          <select bind:value={labState!.meta.selectedAnalysisDatasetId} onchange={() => persistSelection({ selectedAnalysisDatasetId: labState!.meta.selectedAnalysisDatasetId || null })} class="w-full rounded-xl border border-slate-200 bg-slate-50 px-3 py-2.5 text-sm dark:border-slate-600 dark:bg-slate-800 dark:text-slate-50">
            <option value="">{dict.labspaceSelectAnalysisDataset}</option>
            {#each datasetsByKind("analysis") as dataset}
              <option value={dataset.id}>{dataset.name}</option>
            {/each}
          </select>
        </label>
        <label class="space-y-1 text-sm">
          <span class="font-medium text-slate-700 dark:text-slate-200">{dict.blankColumn}</span>
          <select bind:value={labState!.meta.selectedBlankDatasetId} onchange={() => persistSelection({ selectedBlankDatasetId: labState!.meta.selectedBlankDatasetId || null })} class="w-full rounded-xl border border-slate-200 bg-slate-50 px-3 py-2.5 text-sm dark:border-slate-600 dark:bg-slate-800 dark:text-slate-50">
            <option value="">{dict.labspaceNoBlankDataset}</option>
            {#each datasetsByKind("blank") as dataset}
              <option value={dataset.id}>{dataset.name}</option>
            {/each}
          </select>
        </label>
        <label class="space-y-1 text-sm">
          <span class="font-medium text-slate-700 dark:text-slate-200">{dict.labspaceSurrogate}</span>
          <select bind:value={labState!.meta.selectedSurrogateDatasetId} onchange={() => persistSelection({ selectedSurrogateDatasetId: labState!.meta.selectedSurrogateDatasetId || null })} class="w-full rounded-xl border border-slate-200 bg-slate-50 px-3 py-2.5 text-sm dark:border-slate-600 dark:bg-slate-800 dark:text-slate-50">
            <option value="">{dict.labspaceNoSurrogateDataset}</option>
            {#each datasetsByKind("surrogate_observed") as dataset}
              <option value={dataset.id}>{dataset.name}</option>
            {/each}
          </select>
        </label>
      </div>
    </section>

    {#if error}
      <div class="rounded-2xl border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700 dark:border-red-900 dark:bg-red-950 dark:text-red-300">
        {error}
      </div>
    {/if}
    {#if success}
      <div class="rounded-2xl border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-700 dark:border-emerald-900 dark:bg-emerald-950 dark:text-emerald-300">
        {success}
      </div>
    {/if}

    <section class="grid gap-5 xl:grid-cols-[minmax(0,1.04fr)_minmax(23rem,0.96fr)]">
      <div class="space-y-5">
        <div class="rounded-[1.5rem] border border-slate-200 bg-white p-4 shadow-sm dark:border-slate-700 dark:bg-slate-900 sm:p-5">
          <div class="flex items-center justify-between gap-3">
            <div>
              <h2 class="text-lg font-semibold text-slate-900 dark:text-slate-50">{dict.labspaceDatasetsTitle}</h2>
              <p class="mt-1 text-sm text-slate-500 dark:text-slate-400">{dict.labspaceDatasetsDesc}</p>
            </div>
          </div>

          <div class="mt-4 grid gap-4">
            <div class="rounded-2xl border border-slate-200 bg-slate-50 p-3 dark:border-slate-700 dark:bg-slate-800">
              <div class="flex flex-col gap-3 xl:flex-row xl:items-end xl:justify-between">
                <div class="min-w-0">
                  <div class="text-sm font-medium text-slate-800 dark:text-slate-100">{dict.labspaceImportDataset}</div>
                </div>
                <div class="grid gap-2 sm:grid-cols-[10rem_minmax(0,1fr)] sm:items-end xl:min-w-[30rem]">
                  <label class="space-y-1 text-sm">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-slate-500 dark:text-slate-400">{dict.labspaceUploadType}</span>
                    <select bind:value={importKind} class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50">
                      {#each datasetTypeOptions() as option}
                        <option value={option.value}>{option.label}</option>
                      {/each}
                    </select>
                  </label>
                  <label class="space-y-1 text-sm">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-slate-500 dark:text-slate-400">{dict.chooseFile}</span>
                    <input type="file" accept=".xlsx,.xls,.csv,.tsv,.txt" onchange={(event) => handleDatasetImport(importKind, event)} class="block w-full text-[11px] text-slate-500 file:mr-2 file:rounded-full file:border-0 file:bg-slate-900 file:px-3 file:py-2 file:text-xs file:font-medium file:text-white dark:text-slate-400 dark:file:bg-slate-100 dark:file:text-slate-900" />
                  </label>
                </div>
              </div>
            </div>

            <div class="space-y-3 md:hidden">
              {#if labState.datasets.length === 0}
                <div class="rounded-2xl border border-dashed border-slate-300 px-4 py-6 text-center text-sm text-slate-500 dark:border-slate-600 dark:text-slate-400">
                  {dict.labspaceNoDatasetsImported}
                </div>
              {/if}
              {#each labState.datasets as dataset}
                <div class={`rounded-2xl border p-4 ${labState.meta.selectedDatasetId === dataset.id ? "border-blue-300 bg-blue-50/80 dark:border-blue-700 dark:bg-blue-950/20" : "border-slate-200 bg-white dark:border-slate-700 dark:bg-slate-900"}`}>
                  <div class="flex items-start justify-between gap-3">
                    <div class="min-w-0">
                      <div class="truncate font-medium text-slate-800 dark:text-slate-100">{dataset.name}</div>
                    </div>
                    <div class="rounded-full bg-slate-100 px-2.5 py-1 text-xs font-medium text-slate-600 dark:bg-slate-800 dark:text-slate-300">
                      {dataset.rowCount} rows
                    </div>
                  </div>
                  <div class="mt-3">
                    <label class="space-y-1 text-sm">
                      <span class="font-medium text-slate-700 dark:text-slate-200">{dict.labspaceDatasetType}</span>
                      <select value={dataset.kind} onchange={(event) => updateDatasetKind(dataset.id, (event.currentTarget as HTMLSelectElement).value as StoredDataset["kind"])} class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50">
                        {#each datasetTypeOptions() as option}
                          <option value={option.value}>{option.label}</option>
                        {/each}
                      </select>
                    </label>
                  </div>
                  <div class="mt-3 grid grid-cols-3 gap-2">
                    <button type="button" class="rounded-full border border-slate-200 px-3 py-2 text-xs hover:bg-slate-50 dark:border-slate-600 dark:hover:bg-slate-800" onclick={() => openDatasetEditor(dataset.id)}>{dict.labspaceEdit}</button>
                    <button type="button" class="rounded-full border border-slate-200 px-3 py-2 text-xs hover:bg-slate-50 dark:border-slate-600 dark:hover:bg-slate-800" onclick={() => persistSelection(patchForUsingDataset(dataset))}>{dict.labspaceUse}</button>
                    <button type="button" class="rounded-full border border-red-200 px-3 py-2 text-xs text-red-600 hover:bg-red-50 dark:border-red-900 dark:text-red-300 dark:hover:bg-red-950" onclick={() => deleteDataset(dataset.id)}>{dict.labspaceDelete}</button>
                  </div>
                </div>
              {/each}
            </div>

            <div class="hidden overflow-x-auto rounded-2xl border border-slate-200 dark:border-slate-700 md:block">
              <table class="min-w-full text-sm">
                <thead class="bg-slate-50 text-left text-slate-500 dark:bg-slate-800 dark:text-slate-400">
                  <tr>
                    <th class="px-3 py-3 font-medium">{dict.labspaceName}</th>
                    <th class="px-3 py-3 font-medium">{dict.labspaceDatasetType}</th>
                    <th class="px-3 py-3 font-medium">{dict.labspaceRows}</th>
                    <th class="px-3 py-3 font-medium">{dict.labspaceActions}</th>
                  </tr>
                </thead>
                <tbody>
                  {#if labState.datasets.length === 0}
                    <tr>
                      <td colspan="4" class="px-3 py-6 text-center text-slate-500 dark:text-slate-400">{dict.labspaceNoDatasetsImported}</td>
                    </tr>
                  {/if}
                  {#each labState.datasets as dataset}
                    <tr class={`border-t border-slate-100 dark:border-slate-800 ${labState.meta.selectedDatasetId === dataset.id ? "bg-blue-50/70 dark:bg-blue-950/20" : "bg-white dark:bg-slate-900"}`}>
                      <td class="px-3 py-3">
                        <div class="font-medium text-slate-800 dark:text-slate-100">{dataset.name}</div>
                      </td>
                      <td class="px-3 py-3">
                        <select value={dataset.kind} onchange={(event) => updateDatasetKind(dataset.id, (event.currentTarget as HTMLSelectElement).value as StoredDataset["kind"])} class="w-36 rounded-lg border border-slate-200 bg-white px-2 py-1.5 dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50">
                        {#each datasetTypeOptions() as option}
                          <option value={option.value}>{option.label}</option>
                        {/each}
                      </select>
                      </td>
                      <td class="px-3 py-3 text-slate-500 dark:text-slate-400">{dataset.rowCount}</td>
                      <td class="px-3 py-3">
                        <div class="flex flex-wrap gap-2">
                          <button type="button" class="rounded-full border border-slate-200 px-2 py-1 text-xs hover:bg-slate-50 dark:border-slate-600 dark:hover:bg-slate-800" onclick={() => openDatasetEditor(dataset.id)}>{dict.labspaceEdit}</button>
                          <button type="button" class="rounded-full border border-slate-200 px-2 py-1 text-xs hover:bg-slate-50 dark:border-slate-600 dark:hover:bg-slate-800" onclick={() => persistSelection(patchForUsingDataset(dataset))}>{dict.labspaceUse}</button>
                          <button type="button" class="rounded-full border border-red-200 px-2 py-1 text-xs text-red-600 hover:bg-red-50 dark:border-red-900 dark:text-red-300 dark:hover:bg-red-950" onclick={() => deleteDataset(dataset.id)}>{dict.labspaceDelete}</button>
                        </div>
                      </td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </div>
        </div>

      </div>

      <div class="space-y-5">
        <div class="rounded-[1.5rem] border border-slate-200 bg-white p-4 shadow-sm dark:border-slate-700 dark:bg-slate-900 sm:p-5">
          <div class="flex items-center justify-between gap-3">
            <div>
              <h2 class="text-lg font-semibold text-slate-900 dark:text-slate-50">{dict.labspaceReportsTitle}</h2>
              <p class="mt-1 text-sm text-slate-500 dark:text-slate-400">{dict.labspaceReportsDesc}</p>
            </div>
          </div>

          <div class="mt-4 space-y-2">
            {#if labState.reports.length === 0}
              <div class="rounded-xl border border-dashed border-slate-300 px-3 py-4 text-sm text-slate-500 dark:border-slate-600 dark:text-slate-400">{dict.labspaceNoReports}</div>
            {/if}
            {#each labState.reports as report}
              <div class="rounded-xl border border-slate-200 bg-slate-50 p-3 text-sm dark:border-slate-700 dark:bg-slate-800">
                <div class="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
                  <div>
                    <div class="font-medium text-slate-800 dark:text-slate-100">{report.title}</div>
                    <div class="mt-1 text-xs text-slate-500 dark:text-slate-400">{new Date(report.createdAt).toLocaleString()}</div>
                  </div>
                  <div class="grid grid-cols-2 gap-2 sm:flex">
                    <button type="button" class="rounded-full border border-slate-200 px-2 py-1 text-xs hover:bg-white dark:border-slate-600 dark:hover:bg-slate-900" onclick={() => openReport(report)}>{dict.labspaceOpen}</button>
                    <button type="button" class="rounded-full border border-red-200 px-2 py-1 text-xs text-red-600 hover:bg-red-50 dark:border-red-900 dark:text-red-300 dark:hover:bg-red-950" onclick={() => deleteReport(report.id)}>{dict.labspaceDelete}</button>
                  </div>
                </div>
              </div>
            {/each}
          </div>
        </div>
      </div>
    </section>
  </div>

  {#if showDatasetEditorModal}
    <div class="fixed inset-0 z-40 bg-slate-950/45"></div>
    <div class="fixed inset-0 z-50 flex items-start justify-center overflow-y-auto p-4 sm:p-8">
      <div class="w-full max-w-6xl rounded-[1.5rem] border border-slate-200 bg-white p-4 shadow-2xl dark:border-slate-700 dark:bg-slate-900 sm:rounded-[1.75rem] sm:p-5" role="dialog" aria-modal="true" aria-label={dict.labspaceDatasetEditorTitle}>
        <div class="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
          <div>
            <p class="text-xs font-semibold uppercase tracking-[0.18em] text-slate-400 dark:text-slate-500">{dict.labspaceDatasetEditorEyebrow}</p>
            <h2 class="mt-2 text-xl font-semibold text-slate-900 dark:text-slate-50 sm:text-2xl">
              {selectedDataset()?.name ?? dict.labspaceDatasetEditorTitle}
            </h2>
            <p class="mt-1 text-sm text-slate-500 dark:text-slate-400">{dict.labspaceDatasetEditorDesc}</p>
          </div>
          <div class="grid grid-cols-2 gap-2 sm:flex">
            {#if selectedDataset()}
              <button type="button" class="rounded-full border border-slate-200 px-3 py-2 text-sm hover:bg-slate-50 dark:border-slate-600 dark:text-slate-100 dark:hover:bg-slate-800" onclick={addRowToSelectedDataset}>{dict.labspaceAddRow}</button>
              <button type="button" class="rounded-full bg-slate-900 px-4 py-2 text-sm font-medium text-white hover:bg-slate-700 dark:bg-slate-100 dark:text-slate-900" onclick={saveSelectedDataset} disabled={busy}>{dict.labspaceSaveDataset}</button>
            {/if}
            <button type="button" class="col-span-2 rounded-full border border-slate-200 px-3 py-2 text-sm hover:bg-slate-50 dark:border-slate-600 dark:text-slate-100 dark:hover:bg-slate-800 sm:col-span-1" onclick={closeDatasetEditor}>{dict.close}</button>
          </div>
        </div>

        {#if !selectedDataset()}
          <div class="mt-4 rounded-2xl border border-dashed border-slate-300 px-4 py-8 text-sm text-slate-500 dark:border-slate-600 dark:text-slate-400">{dict.labspaceNoDatasetSelected}</div>
        {:else}
          <div class="mt-4 space-y-4">
            <div class="grid gap-3 sm:grid-cols-2">
              <label class="space-y-1 text-sm">
                <span class="font-medium text-slate-700 dark:text-slate-200">{dict.labspaceDatasetName}</span>
                <input value={selectedDataset()?.name ?? ""} oninput={(event) => updateSelectedDatasetField("name", (event.currentTarget as HTMLInputElement).value)} class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" />
              </label>
              <label class="space-y-1 text-sm">
                <span class="font-medium text-slate-700 dark:text-slate-200">{dict.labspaceDatasetType}</span>
                <select value={selectedDataset()?.kind ?? "analysis"} onchange={(event) => updateDatasetKind(selectedDataset()!.id, (event.currentTarget as HTMLSelectElement).value as StoredDataset["kind"])} class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50">
                  {#each datasetTypeOptions() as option}
                    <option value={option.value}>{option.label}</option>
                  {/each}
                </select>
              </label>
            </div>

            <div class="grid gap-3 sm:grid-cols-2">
              <label class="space-y-1 text-sm">
                <span class="font-medium text-slate-700 dark:text-slate-200">{dict.labspaceSourceName}</span>
                <input value={selectedDataset()?.sourceName ?? ""} oninput={(event) => updateSelectedDatasetField("sourceName", (event.currentTarget as HTMLInputElement).value)} class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" />
              </label>
              <div class="rounded-xl border border-slate-200 bg-slate-50 px-3 py-2 text-sm text-slate-600 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-300">
                {dict.labspaceFileMode}: <span class="font-medium">{datasetTypeLabel(selectedDataset()!.kind)}</span>
              </div>
            </div>

            <div class="space-y-3 md:hidden">
              {#each selectedDataset()?.rows ?? [] as row, rowIndex}
                <div class="rounded-2xl border border-slate-200 bg-slate-50 p-3 dark:border-slate-700 dark:bg-slate-800">
                  <div class="grid gap-3">
                    <label class="space-y-1 text-sm">
                      <span class="font-medium text-slate-700 dark:text-slate-200">{dict.labspaceRole}</span>
                      {#if selectedDataset()?.kind === "analysis"}
                        <select value={getRowRoleValue(row)} onchange={(event) => updateSelectedDatasetRowRole(rowIndex, (event.currentTarget as HTMLSelectElement).value as "auto" | "blank" | "surrogate")} class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50">
                          <option value="auto">{dict.labspaceRoleAuto}</option>
                          <option value="blank">{dict.labspaceDatasetTypeBlank}</option>
                          <option value="surrogate">{dict.labspaceDatasetTypeSurrogate}</option>
                        </select>
                      {:else}
                        <div class="rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm font-medium text-slate-600 dark:border-slate-600 dark:bg-slate-900 dark:text-slate-300">
                          {datasetTypeLabel(selectedDataset()!.kind)}
                        </div>
                      {/if}
                    </label>
                    <div class="grid grid-cols-2 gap-3">
                      <label class="space-y-1 text-sm">
                        <span class="font-medium text-slate-700 dark:text-slate-200">RT</span>
                        <input value={row.RT} oninput={(event) => updateSelectedDatasetRow(rowIndex, "RT", (event.currentTarget as HTMLInputElement).value)} type="number" step="0.001" class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" />
                      </label>
                      <label class="space-y-1 text-sm">
                        <span class="font-medium text-slate-700 dark:text-slate-200">{dict.labspaceBasePeak}</span>
                        <input value={row["Base Peak"] ?? ""} oninput={(event) => updateSelectedDatasetRow(rowIndex, "Base Peak", (event.currentTarget as HTMLInputElement).value)} type="number" step="0.001" class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" />
                      </label>
                    </div>
                    <div class="grid grid-cols-2 gap-3">
                      <label class="space-y-1 text-sm">
                        <span class="font-medium text-slate-700 dark:text-slate-200">{dict.areaMean}</span>
                        <input value={row.Area} oninput={(event) => updateSelectedDatasetRow(rowIndex, "Area", (event.currentTarget as HTMLInputElement).value)} type="number" step="0.1" class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" />
                      </label>
                      <label class="space-y-1 text-sm">
                        <span class="font-medium text-slate-700 dark:text-slate-200">{dict.polarity}</span>
                        <input value={row.Polarity} oninput={(event) => updateSelectedDatasetRow(rowIndex, "Polarity", (event.currentTarget as HTMLInputElement).value)} class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" />
                      </label>
                    </div>
                    <label class="space-y-1 text-sm">
                      <span class="font-medium text-slate-700 dark:text-slate-200">{dict.labspaceFile}</span>
                      <input value={row.File} oninput={(event) => updateSelectedDatasetRow(rowIndex, "File", (event.currentTarget as HTMLInputElement).value)} class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" />
                    </label>
                    <label class="space-y-1 text-sm">
                      <span class="font-medium text-slate-700 dark:text-slate-200">{dict.labspaceLabel}</span>
                      <input value={row.Label ?? ""} oninput={(event) => updateSelectedDatasetRow(rowIndex, "Label", (event.currentTarget as HTMLInputElement).value)} class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" />
                    </label>
                    <button type="button" class="rounded-full border border-red-200 px-3 py-2 text-sm text-red-600 hover:bg-red-50 dark:border-red-900 dark:text-red-300 dark:hover:bg-red-950" onclick={() => removeRowFromSelectedDataset(rowIndex)}>{dict.labspaceDeleteRow}</button>
                  </div>
                </div>
              {/each}
            </div>

            <div class="hidden overflow-x-auto md:block">
              <table class="min-w-full text-sm">
                <thead>
                  <tr class="text-left text-slate-500 dark:text-slate-400">
                    <th class="px-2 py-2 font-medium">{dict.labspaceRole}</th>
                    <th class="px-2 py-2 font-medium">RT</th>
                    <th class="px-2 py-2 font-medium">{dict.labspaceBasePeak}</th>
                    <th class="px-2 py-2 font-medium">{dict.areaMean}</th>
                    <th class="px-2 py-2 font-medium">{dict.polarity}</th>
                    <th class="px-2 py-2 font-medium">{dict.labspaceFile}</th>
                    <th class="px-2 py-2 font-medium">{dict.labspaceLabel}</th>
                    <th class="px-2 py-2 font-medium"></th>
                  </tr>
                </thead>
                <tbody>
                  {#each selectedDataset()?.rows ?? [] as row, rowIndex}
                    <tr class="border-t border-slate-100 dark:border-slate-800">
                      <td class="px-2 py-2">
                        {#if selectedDataset()?.kind === "analysis"}
                          <select value={getRowRoleValue(row)} onchange={(event) => updateSelectedDatasetRowRole(rowIndex, (event.currentTarget as HTMLSelectElement).value as "auto" | "blank" | "surrogate")} class="w-28 rounded-lg border border-slate-200 bg-white px-2 py-1.5 dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50">
                            <option value="auto">{dict.labspaceRoleAuto}</option>
                            <option value="blank">{dict.labspaceDatasetTypeBlank}</option>
                            <option value="surrogate">{dict.labspaceDatasetTypeSurrogate}</option>
                          </select>
                        {:else}
                          <div class="rounded-lg border border-slate-200 bg-slate-50 px-2 py-1.5 text-xs font-medium text-slate-600 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-300">
                            {datasetTypeLabel(selectedDataset()!.kind)}
                          </div>
                        {/if}
                      </td>
                      <td class="px-2 py-2"><input value={row.RT} oninput={(event) => updateSelectedDatasetRow(rowIndex, "RT", (event.currentTarget as HTMLInputElement).value)} type="number" step="0.001" class="w-24 rounded-lg border border-slate-200 bg-white px-2 py-1.5 dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" /></td>
                      <td class="px-2 py-2"><input value={row["Base Peak"] ?? ""} oninput={(event) => updateSelectedDatasetRow(rowIndex, "Base Peak", (event.currentTarget as HTMLInputElement).value)} type="number" step="0.001" class="w-28 rounded-lg border border-slate-200 bg-white px-2 py-1.5 dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" /></td>
                      <td class="px-2 py-2"><input value={row.Area} oninput={(event) => updateSelectedDatasetRow(rowIndex, "Area", (event.currentTarget as HTMLInputElement).value)} type="number" step="0.1" class="w-28 rounded-lg border border-slate-200 bg-white px-2 py-1.5 dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" /></td>
                      <td class="px-2 py-2"><input value={row.Polarity} oninput={(event) => updateSelectedDatasetRow(rowIndex, "Polarity", (event.currentTarget as HTMLInputElement).value)} class="w-28 rounded-lg border border-slate-200 bg-white px-2 py-1.5 dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" /></td>
                      <td class="px-2 py-2"><input value={row.File} oninput={(event) => updateSelectedDatasetRow(rowIndex, "File", (event.currentTarget as HTMLInputElement).value)} class="w-44 rounded-lg border border-slate-200 bg-white px-2 py-1.5 dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" /></td>
                      <td class="px-2 py-2"><input value={row.Label ?? ""} oninput={(event) => updateSelectedDatasetRow(rowIndex, "Label", (event.currentTarget as HTMLInputElement).value)} class="w-44 rounded-lg border border-slate-200 bg-white px-2 py-1.5 dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" /></td>
                      <td class="px-2 py-2"><button type="button" class="rounded-full border border-red-200 px-2 py-1 text-xs text-red-600 hover:bg-red-50 dark:border-red-900 dark:text-red-300 dark:hover:bg-red-950" onclick={() => removeRowFromSelectedDataset(rowIndex)}>{dict.labspaceDelete}</button></td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </div>
        {/if}
      </div>
    </div>
  {/if}

  {#if showWorkspaceModal}
    <div class="fixed inset-0 z-40 bg-slate-950/45"></div>
    <div class="fixed inset-0 z-50 flex items-start justify-center overflow-y-auto p-4 sm:p-8">
      <div class="w-full max-w-4xl rounded-[1.5rem] border border-slate-200 bg-white p-4 shadow-2xl dark:border-slate-700 dark:bg-slate-900 sm:rounded-[1.75rem] sm:p-5" role="dialog" aria-modal="true" aria-label={dict.labspaceWorkspaceSettings}>
        <div class="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
          <div>
            <p class="text-xs font-semibold uppercase tracking-[0.18em] text-slate-400 dark:text-slate-500">{dict.labspaceWorkspaceEyebrow}</p>
            <h2 class="mt-2 text-xl font-semibold text-slate-900 dark:text-slate-50 sm:text-2xl">{dict.labspaceWorkspaceSettings}</h2>
            <p class="mt-1 text-sm text-slate-500 dark:text-slate-400">{dict.labspaceWorkspaceDesc}</p>
          </div>
          <button type="button" class="w-full rounded-full border border-slate-200 px-3 py-2 text-sm hover:bg-slate-50 dark:border-slate-600 dark:text-slate-100 dark:hover:bg-slate-800 sm:w-auto sm:py-1.5" onclick={() => (showWorkspaceModal = false)}>{dict.close}</button>
        </div>

        <div class="mt-5 grid gap-5 lg:grid-cols-[0.9fr_1.1fr]">
          <div class="space-y-3 rounded-2xl border border-slate-200 bg-slate-50 p-4 dark:border-slate-700 dark:bg-slate-800">
            <label class="block space-y-1 text-sm">
              <span class="font-medium text-slate-700 dark:text-slate-200">{dict.labspaceName}</span>
              <input bind:value={labState.meta.name} class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" />
            </label>
            <label class="block space-y-1 text-sm">
              <span class="font-medium text-slate-700 dark:text-slate-200">{dict.labspaceNotes}</span>
              <textarea bind:value={labState.meta.notes} rows="5" class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50"></textarea>
            </label>
            <div class="text-xs text-slate-500 dark:text-slate-400">{dict.labspaceWorkspaceId}: {labState.meta.currentLabSpaceId}</div>
            <div class="flex justify-end">
              <button type="button" class="rounded-full bg-slate-900 px-4 py-2 text-sm font-medium text-white hover:bg-slate-700 dark:bg-slate-100 dark:text-slate-900" onclick={saveMeta} disabled={busy}>{dict.labspaceSaveWorkspace}</button>
            </div>
          </div>

          <div class="space-y-5">
            <div class="rounded-2xl border border-slate-200 bg-slate-50 p-4 dark:border-slate-700 dark:bg-slate-800">
              <div class="flex items-center justify-between gap-3">
                <div>
                  <h3 class="text-lg font-semibold text-slate-900 dark:text-slate-50">{dict.labspaceAnalyzerConfigTitle}</h3>
                  <p class="mt-1 text-sm text-slate-500 dark:text-slate-400">{dict.labspaceAnalyzerConfigDesc}</p>
                </div>
                <button type="button" class="rounded-full border border-slate-200 px-3 py-1.5 text-sm hover:bg-slate-50 dark:border-slate-600 dark:text-slate-100 dark:hover:bg-slate-800" onclick={() => (showAnalyzerConfig = !showAnalyzerConfig)}>
                  {showAnalyzerConfig ? dict.hideAdvancedParams : dict.showAdvancedParams}
                </button>
              </div>

              {#if showAnalyzerConfig}
                <div class="mt-4 space-y-3">
                  <label class="block space-y-1 text-sm">
                    <span class="font-medium text-slate-700 dark:text-slate-200">{dict.labspaceConfigurationLabel}</span>
                    <input bind:value={analyzerDraft.name} class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" />
                  </label>
                  <div class="grid gap-3 sm:grid-cols-2">
                    <label class="space-y-1 text-sm">
                      <span class="font-medium text-slate-700 dark:text-slate-200">{dict.labspaceSignalBlankMin}</span>
                      <input bind:value={analyzerDraft.signal_to_blank_min} type="number" min="0" step="0.1" class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" />
                    </label>
                    <label class="space-y-1 text-sm">
                      <span class="font-medium text-slate-700 dark:text-slate-200">{dict.labspaceMinAreaDifference}</span>
                      <input bind:value={analyzerDraft.min_area_difference} type="number" min="0" step="0.1" placeholder={dict.labspaceOptional} class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" />
                    </label>
                  </div>
                  <label class="inline-flex items-center gap-2 text-sm text-slate-700 dark:text-slate-200">
                    <input bind:checked={analyzerDraft.syncBlankWithReplicate} type="checkbox" class="h-4 w-4 rounded border-slate-300 text-blue-600" />
                    <span>{dict.labspaceSyncBlankTolerances}</span>
                  </label>
                  <div class="grid gap-3 sm:grid-cols-3">
                    <label class="space-y-1 text-sm">
                      <span class="font-medium text-slate-700 dark:text-slate-200">{dict.labspaceRepRt}</span>
                      <input bind:value={analyzerDraft.replicate_rt_tol} oninput={syncBlankWithReplicateDraft} type="number" min="0" step="0.01" class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" />
                    </label>
                    <label class="space-y-1 text-sm">
                      <span class="font-medium text-slate-700 dark:text-slate-200">{dict.labspaceRepMz}</span>
                      <input bind:value={analyzerDraft.replicate_mz_tol} oninput={syncBlankWithReplicateDraft} type="number" min="0" step="0.01" class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" />
                    </label>
                    <label class="space-y-1 text-sm">
                      <span class="font-medium text-slate-700 dark:text-slate-200">{dict.labspaceRepMode}</span>
                      <select bind:value={analyzerDraft.replicate_mz_mode} onchange={syncBlankWithReplicateDraft} class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50">
                        <option value="da">Da</option>
                        <option value="ppm">ppm</option>
                      </select>
                    </label>
                  </div>
                  <div class="grid gap-3 sm:grid-cols-3">
                    <label class="space-y-1 text-sm">
                      <span class="font-medium text-slate-700 dark:text-slate-200">{dict.labspaceBlankRt}</span>
                      <input bind:value={analyzerDraft.blank_rt_tol} type="number" min="0" step="0.01" class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" />
                    </label>
                    <label class="space-y-1 text-sm">
                      <span class="font-medium text-slate-700 dark:text-slate-200">{dict.labspaceBlankMz}</span>
                      <input bind:value={analyzerDraft.blank_mz_tol} type="number" min="0" step="0.01" class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" />
                    </label>
                    <label class="space-y-1 text-sm">
                      <span class="font-medium text-slate-700 dark:text-slate-200">{dict.labspaceBlankMode}</span>
                      <select bind:value={analyzerDraft.blank_mz_mode} class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50">
                        <option value="da">Da</option>
                        <option value="ppm">ppm</option>
                      </select>
                    </label>
                  </div>
                  <div class="grid gap-3 sm:grid-cols-2">
                    <label class="space-y-1 text-sm">
                      <span class="font-medium text-slate-700 dark:text-slate-200">{dict.labspaceCvHighMax}</span>
                      <input bind:value={analyzerDraft.cv_high_max} type="number" min="0" step="0.1" class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" />
                    </label>
                    <label class="space-y-1 text-sm">
                      <span class="font-medium text-slate-700 dark:text-slate-200">{dict.labspaceCvModerateMax}</span>
                      <input bind:value={analyzerDraft.cv_moderate_max} type="number" min="0" step="0.1" class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" />
                    </label>
                  </div>
                  <div class="flex justify-end">
                    <button type="button" class="rounded-full bg-slate-900 px-4 py-2 text-sm font-medium text-white hover:bg-slate-700 dark:bg-slate-100 dark:text-slate-900" onclick={saveAnalyzer} disabled={busy}>{dict.labspaceSaveConfig}</button>
                  </div>
                </div>
              {/if}
            </div>

            <div class="rounded-2xl border border-slate-200 bg-slate-50 p-4 dark:border-slate-700 dark:bg-slate-800">
              <div class="flex items-center justify-between gap-3">
                <div>
                  <h3 class="text-lg font-semibold text-slate-900 dark:text-slate-50">{dict.labspaceSurrogateSpecsTitle}</h3>
                  <p class="mt-1 text-sm text-slate-500 dark:text-slate-400">{dict.labspaceSurrogateSpecsDesc}</p>
                </div>
                <button type="button" class="rounded-full border border-slate-200 px-3 py-1.5 text-sm hover:bg-slate-50 dark:border-slate-600 dark:text-slate-100 dark:hover:bg-slate-800" onclick={addSurrogateSpec}>{dict.labspaceAddSpec}</button>
              </div>

              <div class="mt-4 space-y-3 md:hidden">
                {#if labState.surrogateSpecs.length === 0}
                  <div class="rounded-xl border border-dashed border-slate-300 px-3 py-4 text-sm text-slate-500 dark:border-slate-600 dark:text-slate-400">{dict.labspaceNoSurrogateSpecs}</div>
                {/if}
                {#each labState.surrogateSpecs as spec}
                  <div class="rounded-2xl border border-slate-200 bg-white p-3 dark:border-slate-700 dark:bg-slate-900">
                    <div class="grid gap-3">
                      <label class="space-y-1 text-sm">
                        <span class="font-medium text-slate-700 dark:text-slate-200">{dict.labspaceName}</span>
                        <input value={spec.name} oninput={(event) => updateSurrogateSpec(spec.id, "name", (event.currentTarget as HTMLInputElement).value)} class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" />
                      </label>
                      <div class="grid grid-cols-2 gap-3">
                        <label class="space-y-1 text-sm">
                          <span class="font-medium text-slate-700 dark:text-slate-200">{dict.labspaceExpectedRt}</span>
                          <input value={spec.expected_rt ?? ""} oninput={(event) => updateSurrogateSpec(spec.id, "expected_rt", (event.currentTarget as HTMLInputElement).value)} type="number" step="0.01" class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" />
                        </label>
                        <label class="space-y-1 text-sm">
                          <span class="font-medium text-slate-700 dark:text-slate-200">{dict.labspaceExpectedArea}</span>
                          <input value={spec.expected_area ?? ""} oninput={(event) => updateSurrogateSpec(spec.id, "expected_area", (event.currentTarget as HTMLInputElement).value)} type="number" step="0.1" class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" />
                        </label>
                      </div>
                      <div class="grid grid-cols-2 gap-3">
                        <label class="space-y-1 text-sm">
                          <span class="font-medium text-slate-700 dark:text-slate-200">{dict.labspaceExpectedMz}</span>
                          <input value={spec.expected_mz ?? ""} oninput={(event) => updateSurrogateSpec(spec.id, "expected_mz", (event.currentTarget as HTMLInputElement).value)} type="number" step="0.01" class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" />
                        </label>
                        <label class="space-y-1 text-sm">
                          <span class="font-medium text-slate-700 dark:text-slate-200">{dict.labspaceRtWindow}</span>
                          <input value={spec.rt_window ?? ""} oninput={(event) => updateSurrogateSpec(spec.id, "rt_window", (event.currentTarget as HTMLInputElement).value)} type="number" step="0.01" class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" />
                        </label>
                      </div>
                      <div class="grid grid-cols-2 gap-3">
                        <label class="space-y-1 text-sm">
                          <span class="font-medium text-slate-700 dark:text-slate-200">{dict.labspaceRecoveryMin}</span>
                          <input value={spec.recovery_min_pct ?? ""} oninput={(event) => updateSurrogateSpec(spec.id, "recovery_min_pct", (event.currentTarget as HTMLInputElement).value)} type="number" step="0.1" class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" />
                        </label>
                        <label class="space-y-1 text-sm">
                          <span class="font-medium text-slate-700 dark:text-slate-200">{dict.labspaceRecoveryMax}</span>
                          <input value={spec.recovery_max_pct ?? ""} oninput={(event) => updateSurrogateSpec(spec.id, "recovery_max_pct", (event.currentTarget as HTMLInputElement).value)} type="number" step="0.1" class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" />
                        </label>
                      </div>
                      <button type="button" class="rounded-full border border-red-200 px-3 py-2 text-sm text-red-600 hover:bg-red-50 dark:border-red-900 dark:text-red-300 dark:hover:bg-red-950" onclick={() => removeSurrogateSpec(spec.id)}>{dict.labspaceDeleteSpec}</button>
                    </div>
                  </div>
                {/each}
              </div>

              <div class="mt-4 hidden overflow-x-auto md:block">
                <table class="min-w-full text-sm">
                  <thead>
                    <tr class="text-left text-slate-500 dark:text-slate-400">
                      <th class="px-2 py-2 font-medium">{dict.labspaceName}</th>
                      <th class="px-2 py-2 font-medium">{dict.labspaceExpectedRt}</th>
                      <th class="px-2 py-2 font-medium">{dict.labspaceExpectedArea}</th>
                      <th class="px-2 py-2 font-medium">{dict.labspaceExpectedMz}</th>
                      <th class="px-2 py-2 font-medium">{dict.labspaceRtWindow}</th>
                      <th class="px-2 py-2 font-medium">{dict.labspaceRecoveryMin}</th>
                      <th class="px-2 py-2 font-medium">{dict.labspaceRecoveryMax}</th>
                      <th class="px-2 py-2 font-medium"></th>
                    </tr>
                  </thead>
                  <tbody>
                    {#if labState.surrogateSpecs.length === 0}
                      <tr>
                        <td colspan="8" class="px-2 py-4 text-slate-500 dark:text-slate-400">{dict.labspaceNoSurrogateSpecs}</td>
                      </tr>
                    {/if}
                    {#each labState.surrogateSpecs as spec}
                      <tr class="border-t border-slate-100 dark:border-slate-800">
                        <td class="px-2 py-2"><input value={spec.name} oninput={(event) => updateSurrogateSpec(spec.id, "name", (event.currentTarget as HTMLInputElement).value)} class="w-44 rounded-lg border border-slate-200 bg-white px-2 py-1.5 dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" /></td>
                        <td class="px-2 py-2"><input value={spec.expected_rt ?? ""} oninput={(event) => updateSurrogateSpec(spec.id, "expected_rt", (event.currentTarget as HTMLInputElement).value)} type="number" step="0.01" class="w-24 rounded-lg border border-slate-200 bg-white px-2 py-1.5 dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" /></td>
                        <td class="px-2 py-2"><input value={spec.expected_area ?? ""} oninput={(event) => updateSurrogateSpec(spec.id, "expected_area", (event.currentTarget as HTMLInputElement).value)} type="number" step="0.1" class="w-28 rounded-lg border border-slate-200 bg-white px-2 py-1.5 dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" /></td>
                        <td class="px-2 py-2"><input value={spec.expected_mz ?? ""} oninput={(event) => updateSurrogateSpec(spec.id, "expected_mz", (event.currentTarget as HTMLInputElement).value)} type="number" step="0.01" class="w-24 rounded-lg border border-slate-200 bg-white px-2 py-1.5 dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" /></td>
                        <td class="px-2 py-2"><input value={spec.rt_window ?? ""} oninput={(event) => updateSurrogateSpec(spec.id, "rt_window", (event.currentTarget as HTMLInputElement).value)} type="number" step="0.01" class="w-24 rounded-lg border border-slate-200 bg-white px-2 py-1.5 dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" /></td>
                        <td class="px-2 py-2"><input value={spec.recovery_min_pct ?? ""} oninput={(event) => updateSurrogateSpec(spec.id, "recovery_min_pct", (event.currentTarget as HTMLInputElement).value)} type="number" step="0.1" class="w-24 rounded-lg border border-slate-200 bg-white px-2 py-1.5 dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" /></td>
                        <td class="px-2 py-2"><input value={spec.recovery_max_pct ?? ""} oninput={(event) => updateSurrogateSpec(spec.id, "recovery_max_pct", (event.currentTarget as HTMLInputElement).value)} type="number" step="0.1" class="w-24 rounded-lg border border-slate-200 bg-white px-2 py-1.5 dark:border-slate-600 dark:bg-slate-900 dark:text-slate-50" /></td>
                        <td class="px-2 py-2"><button type="button" class="rounded-full border border-red-200 px-2 py-1 text-xs text-red-600 hover:bg-red-50 dark:border-red-900 dark:text-red-300 dark:hover:bg-red-950" onclick={() => removeSurrogateSpec(spec.id)}>{dict.labspaceDelete}</button></td>
                      </tr>
                    {/each}
                  </tbody>
                </table>
              </div>

              <div class="mt-4 flex justify-end">
                <button type="button" class="rounded-full bg-slate-900 px-4 py-2 text-sm font-medium text-white hover:bg-slate-700 dark:bg-slate-100 dark:text-slate-900" onclick={saveSurrogateSpecs} disabled={busy}>{dict.labspaceSaveSpecs}</button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  {/if}
{/if}
