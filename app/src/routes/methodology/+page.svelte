<script lang="ts">
    import { locale } from "$lib/i18n";
    import MethodologyPageEn from "$lib/components/methodology-page/MethodologyPageEn.svelte";
    import MethodologyPageRu from "$lib/components/methodology-page/MethodologyPageRu.svelte";
    import MethodologyPageUk from "$lib/components/methodology-page/MethodologyPageUk.svelte";

    const componentByLocale = {
        uk: MethodologyPageUk,
        en: MethodologyPageEn,
        ru: MethodologyPageRu,
    } as const;

    let currentLocale = $derived($locale);
    let CurrentPage = $derived(componentByLocale[currentLocale]);
</script>

<CurrentPage />
