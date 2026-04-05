import { defineRegistry } from "@json-render/svelte";
import { catalog } from "./catalog";
import Dashboard from "../components/Dashboard.svelte";
import StatCard from "../components/StatCard.svelte";

export const { registry } = defineRegistry(catalog, {
  components: {
    Dashboard,
    MetricCard: StatCard,
  },
});
