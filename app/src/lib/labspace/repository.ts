import type { ParsedRow } from "$lib/screening";
import type {
  AnalyzerPreset,
  AnalyzerRuntimeParams,
  LabSpaceMeta,
  LabSpaceState,
  StoredDataset,
  StoredReport,
  StoredSurrogateSpec,
} from "./types";
import { LAB_SPACE_ID } from "./types";

type StoreName = "app_meta" | "datasets" | "surrogate_specs" | "analyzers" | "reports";

interface StoredEntity {
  id: string;
}

export interface LabSpaceAdapter {
  get<T>(store: StoreName, key: string): Promise<T | undefined>;
  getAll<T>(store: StoreName): Promise<T[]>;
  put<T extends StoredEntity>(store: StoreName, value: T): Promise<void>;
  delete(store: StoreName, key: string): Promise<void>;
}

const DB_NAME = "lcms-labspace";
const DB_VERSION = 1;
const STORE_NAMES: StoreName[] = [
  "app_meta",
  "datasets",
  "surrogate_specs",
  "analyzers",
  "reports",
];

function nowIso(): string {
  return new Date().toISOString();
}

function makeId(prefix: string): string {
  if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
    return `${prefix}_${crypto.randomUUID()}`;
  }
  return `${prefix}_${Date.now()}_${Math.random().toString(36).slice(2, 10)}`;
}

export function defaultAnalyzerParams(): AnalyzerRuntimeParams {
  return {
    replicate_rt_tol: 0.1,
    replicate_mz_tol: 0.3,
    replicate_mz_mode: "da",
    blank_rt_tol: 0.1,
    blank_mz_tol: 0.3,
    blank_mz_mode: "da",
    signal_to_blank_min: 3,
    cv_high_max: 15,
    cv_moderate_max: 30,
    min_area_difference: undefined,
  };
}

export function defaultAnalyzerPreset(): AnalyzerPreset {
  const timestamp = nowIso();
  return {
    id: makeId("analyzer"),
    name: "Default analyzer",
    params: defaultAnalyzerParams(),
    syncBlankWithReplicate: true,
    createdAt: timestamp,
    updatedAt: timestamp,
  };
}

export function defaultLabSpaceMeta(activeAnalyzerId: string | null): LabSpaceMeta {
  const timestamp = nowIso();
  return {
    id: LAB_SPACE_ID,
    currentLabSpaceId: LAB_SPACE_ID,
    name: "Default LabSpace",
    notes: "",
    activeAnalyzerId,
    selectedAnalysisDatasetId: null,
    selectedBlankDatasetId: null,
    selectedSurrogateDatasetId: null,
    selectedDatasetId: null,
    selectedReportId: null,
    createdAt: timestamp,
    updatedAt: timestamp,
  };
}

export function createIndexedDbAdapter(): LabSpaceAdapter {
  async function openDb(): Promise<IDBDatabase> {
    const indexedDb = globalThis.indexedDB;
    if (!indexedDb) {
      throw new Error("IndexedDB is not available in this environment.");
    }

    return new Promise((resolve, reject) => {
      const request = indexedDb.open(DB_NAME, DB_VERSION);
      request.onupgradeneeded = () => {
        const db = request.result;
        for (const storeName of STORE_NAMES) {
          if (!db.objectStoreNames.contains(storeName)) {
            db.createObjectStore(storeName, { keyPath: "id" });
          }
        }
      };
      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error ?? new Error("Failed to open IndexedDB."));
    });
  }

  async function withStore<T>(
    storeName: StoreName,
    mode: IDBTransactionMode,
    handler: (store: IDBObjectStore) => IDBRequest<T>,
  ): Promise<T> {
    const db = await openDb();
    return new Promise((resolve, reject) => {
      const tx = db.transaction(storeName, mode);
      const store = tx.objectStore(storeName);
      const request = handler(store);

      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error ?? new Error(`IndexedDB request failed for ${storeName}.`));
      tx.oncomplete = () => db.close();
      tx.onerror = () => reject(tx.error ?? new Error(`IndexedDB transaction failed for ${storeName}.`));
      tx.onabort = () => reject(tx.error ?? new Error(`IndexedDB transaction aborted for ${storeName}.`));
    });
  }

  return {
    get: async <T>(store: StoreName, key: string) =>
      withStore<T | undefined>(store, "readonly", (objectStore) => objectStore.get(key)),
    getAll: async <T>(store: StoreName) =>
      withStore<T[]>(store, "readonly", (objectStore) => objectStore.getAll()),
    put: async <T extends StoredEntity>(store: StoreName, value: T) => {
      await withStore<IDBValidKey>(store, "readwrite", (objectStore) => objectStore.put(value));
    },
    delete: async (store: StoreName, key: string) => {
      await withStore<undefined>(store, "readwrite", (objectStore) => objectStore.delete(key));
    },
  };
}

function sortByUpdatedAtDesc<T extends { updatedAt?: string; createdAt?: string }>(items: T[]): T[] {
  return [...items].sort((left, right) => {
    const a = left.updatedAt ?? left.createdAt ?? "";
    const b = right.updatedAt ?? right.createdAt ?? "";
    return b.localeCompare(a);
  });
}

export function createMemoryLabSpaceAdapter(): LabSpaceAdapter {
  const stores = new Map<StoreName, Map<string, StoredEntity>>();
  for (const storeName of STORE_NAMES) {
    stores.set(storeName, new Map());
  }

  return {
    async get<T>(store: StoreName, key: string) {
      return stores.get(store)?.get(key) as T | undefined;
    },
    async getAll<T>(store: StoreName) {
      return Array.from(stores.get(store)?.values() ?? []) as T[];
    },
    async put<T extends StoredEntity>(store: StoreName, value: T) {
      stores.get(store)?.set(value.id, structuredClone(value));
    },
    async delete(store: StoreName, key: string) {
      stores.get(store)?.delete(key);
    },
  };
}

export function createLabSpaceRepository(adapter: LabSpaceAdapter) {
  async function ensureDefaults(): Promise<LabSpaceMeta> {
    const existingAnalyzers = sortByUpdatedAtDesc(await adapter.getAll<AnalyzerPreset>("analyzers"));
    let activeAnalyzerId = existingAnalyzers[0]?.id ?? null;

    if (existingAnalyzers.length === 0) {
      const analyzer = defaultAnalyzerPreset();
      await adapter.put("analyzers", analyzer);
      activeAnalyzerId = analyzer.id;
    }

    const existingMeta = await adapter.get<LabSpaceMeta>("app_meta", LAB_SPACE_ID);
    if (existingMeta) {
      return existingMeta;
    }

    const meta = defaultLabSpaceMeta(activeAnalyzerId);
    await adapter.put("app_meta", meta);
    return meta;
  }

  return {
    async loadState(): Promise<LabSpaceState> {
      let meta = await ensureDefaults();
      const analyzers = sortByUpdatedAtDesc(await adapter.getAll<AnalyzerPreset>("analyzers"));

      if (!meta.activeAnalyzerId && analyzers[0]) {
        meta = {
          ...meta,
          activeAnalyzerId: analyzers[0].id,
          updatedAt: nowIso(),
        };
        await adapter.put("app_meta", meta);
      }

      return {
        meta,
        datasets: sortByUpdatedAtDesc(await adapter.getAll<StoredDataset>("datasets")),
        surrogateSpecs: await adapter.getAll<StoredSurrogateSpec>("surrogate_specs"),
        analyzers,
        reports: sortByUpdatedAtDesc(await adapter.getAll<StoredReport>("reports")),
      };
    },

    async saveMeta(meta: LabSpaceMeta): Promise<LabSpaceMeta> {
      const nextMeta = {
        ...meta,
        updatedAt: nowIso(),
      };
      await adapter.put("app_meta", nextMeta);
      return nextMeta;
    },

    async upsertDataset(input: Omit<StoredDataset, "id" | "createdAt" | "updatedAt" | "rowCount"> & { id?: string }): Promise<StoredDataset> {
      const existing = input.id
        ? await adapter.get<StoredDataset>("datasets", input.id)
        : undefined;
      const timestamp = nowIso();
      const dataset: StoredDataset = {
        id: input.id ?? makeId("dataset"),
        kind: input.kind,
        name: input.name,
        sourceName: input.sourceName,
        rows: input.rows,
        rowCount: input.rows.length,
        createdAt: existing?.createdAt ?? timestamp,
        updatedAt: timestamp,
      };
      await adapter.put("datasets", dataset);
      return dataset;
    },

    async deleteDataset(id: string): Promise<void> {
      await adapter.delete("datasets", id);
    },

    async upsertAnalyzer(input: Omit<AnalyzerPreset, "id" | "createdAt" | "updatedAt"> & { id?: string }): Promise<AnalyzerPreset> {
      const existing = input.id
        ? await adapter.get<AnalyzerPreset>("analyzers", input.id)
        : undefined;
      const timestamp = nowIso();
      const analyzer: AnalyzerPreset = {
        id: input.id ?? makeId("analyzer"),
        name: input.name,
        params: input.params,
        syncBlankWithReplicate: input.syncBlankWithReplicate,
        createdAt: existing?.createdAt ?? timestamp,
        updatedAt: timestamp,
      };
      await adapter.put("analyzers", analyzer);
      return analyzer;
    },

    async deleteAnalyzer(id: string): Promise<void> {
      await adapter.delete("analyzers", id);
    },

    async replaceSurrogateSpecs(specs: StoredSurrogateSpec[]): Promise<void> {
      const existing = await adapter.getAll<StoredSurrogateSpec>("surrogate_specs");
      for (const spec of existing) {
        await adapter.delete("surrogate_specs", spec.id);
      }
      for (const spec of specs) {
        await adapter.put("surrogate_specs", spec);
      }
    },

    async createReport(input: Omit<StoredReport, "id" | "createdAt">): Promise<StoredReport> {
      const report: StoredReport = {
        ...input,
        id: makeId("report"),
        createdAt: nowIso(),
      };
      await adapter.put("reports", report);
      return report;
    },

    async deleteReport(id: string): Promise<void> {
      await adapter.delete("reports", id);
    },
  };
}

export const labSpaceRepository = createLabSpaceRepository(createIndexedDbAdapter());

export function createDefaultSurrogateSpec(): StoredSurrogateSpec {
  return {
    id: makeId("surrogate_spec"),
    name: "",
    expected_rt: 0,
    expected_area: 0,
  };
}

export function createEmptyParsedRow(): ParsedRow {
  return {
    RT: 0,
    "Base Peak": null,
    Area: 0,
    Polarity: "Positive",
    File: "",
    Label: "",
    operator_color: null,
    operator_mark: null,
  };
}

export function validateSurrogateSpecs(specs: StoredSurrogateSpec[]): string | null {
  const invalid = specs.find((spec) => !spec.name.trim() || spec.expected_area <= 0);
  if (!invalid) {
    return null;
  }
  return "Each surrogate spec needs a name and expected area > 0.";
}
