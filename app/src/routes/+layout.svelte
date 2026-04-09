<script lang="ts">
    import "./layout.css";
    import { initLocale } from "$lib/i18n";
    import { theme, applyTheme } from "$lib/theme";

    let { children } = $props();

    const manifestHref = import.meta.env.VITE_STANDALONE
        ? "./manifest.webmanifest"
        : "/manifest.webmanifest";
    const themeColor = "#2563eb";

    $effect(() => {
        initLocale();

        const saved = (localStorage.getItem("theme") as "auto" | "light" | "dark" | null) ?? "auto";
        theme.set(saved);
        applyTheme(saved);

        const mq = window.matchMedia("(prefers-color-scheme: dark)");
        const onChange = () => {
            const current = localStorage.getItem("theme") ?? "auto";
            if (current === "auto") applyTheme("auto");
        };
        mq.addEventListener("change", onChange);
        return () => mq.removeEventListener("change", onChange);
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
