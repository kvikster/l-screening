import { z } from "zod";
import { defineCatalog } from "@json-render/core";
import { schema } from "@json-render/svelte/schema";

export const PeakSchema = z.object({
  RT_mean: z.number(),
  MZ_mean: z.number(),
  Area_mean: z.number(),
  Polarity: z.string(),
  SampleType: z.string(),
  Status: z.string(),
  Why: z.object({
    Rep1_RT: z.number(),
    Rep2_RT: z.number(),
    Rep1_MZ: z.number(),
    Rep2_MZ: z.number(),
    Matches: z.boolean(),
    BlankMatch: z.boolean().optional(),
    BlankDetail: z.object({
      RT: z.number(),
      MZ: z.number(),
    }).optional(),
  }).passthrough(),
}).passthrough();

export const SummarySchema = z.object({
  Sample: z.string(),
  Polarity: z.string(),
  TotalPeaks: z.number(),
  Confirmed: z.number(),
  Artifacts: z.number(),
  RealCompounds: z.number(),
}).passthrough();

export const DashboardPropsSchema = z.object({
  title: z.string(),
  summary: z.array(SummarySchema).default([]),
  peaks: z.array(PeakSchema).default([]),
}).passthrough();

export const MetricCardPropsSchema = z.object({
  label: z.string(),
  value: z.union([z.string(), z.number()]),
  description: z.string().optional(),
}).passthrough();

export const catalog = defineCatalog(schema, {
  components: {
    Dashboard: {
      props: DashboardPropsSchema,
      description: "Main dashboard layout",
    },
    MetricCard: {
      props: MetricCardPropsSchema,
      description: "Metric display card",
    },
  },
  actions: {},
});

export type Catalog = typeof catalog;
export type DashboardProps = z.infer<typeof DashboardPropsSchema>;
export type MetricCardProps = z.infer<typeof MetricCardPropsSchema>;
export type Peak = z.infer<typeof PeakSchema>;
export type Summary = z.infer<typeof SummarySchema>;
