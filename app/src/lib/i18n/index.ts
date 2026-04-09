import { browser } from "$app/environment";
import { derived, get, writable } from "svelte/store";
import { enMessages } from "./messages/en";
import { ruMessages } from "./messages/ru";
import { ukMessages } from "./messages/uk";

export const APP_LOCALES = ["uk", "en", "ru"] as const;
export type AppLocale = (typeof APP_LOCALES)[number];

export const DEFAULT_LOCALE: AppLocale = "uk";
const STORAGE_KEY = "locale";

const LOCALE_FORMATS: Record<AppLocale, string> = {
    uk: "uk-UA",
    en: "en-US",
    ru: "ru-RU",
};

const translations = {
    uk: ukMessages,
    en: enMessages,
    ru: ruMessages,
} as const;

type TranslationDict = typeof ukMessages;
export type TranslationKey = keyof TranslationDict;

function normalizeLocale(value: string | null | undefined): AppLocale {
    if (!value) return DEFAULT_LOCALE;

    const normalized = value.toLowerCase();
    if (normalized.startsWith("uk")) return "uk";
    if (normalized.startsWith("ru")) return "ru";
    if (normalized.startsWith("en")) return "en";
    return DEFAULT_LOCALE;
}

const localeStore = writable<AppLocale>(DEFAULT_LOCALE);
let initialized = false;

function applyLocaleSideEffects(locale: AppLocale) {
    if (!browser) return;
    localStorage.setItem(STORAGE_KEY, locale);
    document.documentElement.lang = locale;
}

export const locale = {
    subscribe: localeStore.subscribe,
};

export const dictionary = derived(localeStore, ($locale) => translations[$locale]);

export const localeLabels: Record<AppLocale, string> = {
    uk: "UA",
    en: "EN",
    ru: "RU",
};

export function initLocale(): void {
    if (!browser || initialized) return;
    initialized = true;

    const saved = localStorage.getItem(STORAGE_KEY);
    const detected = normalizeLocale(saved ?? navigator.language);
    localeStore.set(detected);
    applyLocaleSideEffects(detected);
}

export function setLocale(nextLocale: string): void {
    const normalized = normalizeLocale(nextLocale);
    localeStore.set(normalized);
    applyLocaleSideEffects(normalized);
}

export function t(
    key: TranslationKey,
    variables?: Record<string, string | number>,
): string {
    const template = translations[getCurrentLocale()][key];
    if (!variables) return template;

    return template.replace(/\{(\w+)\}/g, (_, variableName: string) => {
        const value = variables[variableName];
        return value === undefined ? `{${variableName}}` : String(value);
    });
}

export function getCurrentLocale(): AppLocale {
    return get(localeStore);
}

export function getNumberLocale(): string {
    return LOCALE_FORMATS[getCurrentLocale()];
}

export function getStatusLabel(status: string): string {
    if (status === "Real Compound") return t("realCompound");
    if (status === "Artifact") return t("artifact");
    return status;
}

export function getSampleTypeLabel(sampleType: string): string {
    if (sampleType === "sample") return t("sampleLabel");
    if (sampleType === "blank") return t("blankLabel");
    return sampleType;
}

export function getReplicateQualityLabel(quality: string): string {
    if (quality === "High") return t("qualityHigh");
    if (quality === "Moderate") return t("qualityModerate");
    if (quality === "Low") return t("qualityLow");
    return quality;
}
