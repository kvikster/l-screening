<script lang="ts">
    import "./layout.css";
    import {
        dictionary,
        initLocale,
        locale,
        localeLabels,
        setLocale,
    } from "$lib/i18n";

    let { children } = $props();

    const manifestHref = import.meta.env.VITE_STANDALONE
        ? "./manifest.webmanifest"
        : "/manifest.webmanifest";
    const themeColor = "#2563eb";

    type Theme = "auto" | "light" | "dark";
    let theme = $state<Theme>("auto");
    let dict = $derived($dictionary);
    let currentLocale = $derived($locale);

    function applyTheme(t: Theme) {
        const isDark =
            t === "dark" ||
            (t === "auto" &&
                window.matchMedia("(prefers-color-scheme: dark)").matches);
        document.documentElement.classList.toggle("dark", isDark);
    }

    $effect(() => {
        initLocale();

        const saved = (localStorage.getItem("theme") as Theme | null) ?? "auto";
        theme = saved;
        applyTheme(theme);

        const mq = window.matchMedia("(prefers-color-scheme: dark)");
        const onChange = () => {
            if (theme === "auto") applyTheme("auto");
        };
        mq.addEventListener("change", onChange);
        return () => mq.removeEventListener("change", onChange);
    });

    function cycleTheme() {
        const next: Theme =
            theme === "auto" ? "light" : theme === "light" ? "dark" : "auto";
        theme = next;
        localStorage.setItem("theme", next);
        applyTheme(next);
    }

    const icons: Record<Theme, string> = {
        auto: "M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z",
        light: "M12 2v2m0 16v2M4.93 4.93l1.41 1.41m11.32 11.32 1.41 1.41M2 12h2m16 0h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41M12 7a5 5 0 1 0 0 10A5 5 0 0 0 12 7Z",
        dark: "M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79Z",
    };
    let labels = $derived({
        auto: dict.themeAuto,
        light: dict.themeLight,
        dark: dict.themeDark,
    });
</script>

<svelte:head>
    <link rel="icon" href="./icons/icon-32.png" type="image/png" />
    <link rel="apple-touch-icon" href="./icons/icon-256.png" />
    <link rel="manifest" href={manifestHref} />
    <meta name="application-name" content="LC-MS Screening" />
    <meta name="apple-mobile-web-app-capable" content="yes" />
    <meta name="apple-mobile-web-app-status-bar-style" content="default" />
    <meta name="apple-mobile-web-app-title" content="LC-MS Screening" />
    <meta name="mobile-web-app-capable" content="yes" />
    <meta name="theme-color" content={themeColor} />
</svelte:head>

{@render children()}

<div
    class="fixed right-4 top-4 z-50 flex items-center gap-3 rounded-full border border-slate-200 bg-white px-3 py-2 text-xs font-medium text-slate-700 shadow-md dark:border-slate-700 dark:bg-slate-800 dark:text-slate-200"
>
    <label class="flex items-center gap-2">
        <span class="text-slate-500 dark:text-slate-400" aria-hidden="true">
            <svg
                class="h-4 w-4"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
            >
                <circle cx="12" cy="12" r="10" />
                <path d="M2 12h20" />
                <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
            </svg>
        </span>
        <select
            class="bg-transparent outline-none"
            value={currentLocale}
            onchange={(event) =>
                setLocale((event.currentTarget as HTMLSelectElement).value)}
            aria-label={dict.languageLabel}
        >
            {#each Object.entries(localeLabels) as [value, label]}
                <option value={value}>{label}</option>
            {/each}
        </select>
    </label>
    <button
        onclick={cycleTheme}
        title={labels[theme]}
        aria-label={labels[theme]}
        class="flex h-8 w-8 items-center justify-center rounded-full border border-slate-200 bg-white shadow-sm hover:bg-slate-50 dark:border-slate-600 dark:bg-slate-800 dark:hover:bg-slate-700"
    >
        <svg
            class="h-4 w-4 text-slate-600 dark:text-slate-300"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
        >
            <path d={icons[theme]} />
        </svg>
    </button>
</div>
