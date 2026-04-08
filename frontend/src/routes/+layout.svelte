<script lang="ts">
    import "./layout.css";
    import favicon from "$lib/assets/favicon.svg";

    let { children } = $props();

    const manifestHref = import.meta.env.VITE_STANDALONE
        ? "./manifest.webmanifest"
        : "/manifest.webmanifest";
    const themeColor = "#2563eb";
</script>

<svelte:head>
    <link rel="icon" href={favicon} />
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
