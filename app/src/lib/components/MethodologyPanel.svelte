<script lang="ts">
    import { locale } from "$lib/i18n";
    import MethodologyPanelEn from "./methodology-panel/MethodologyPanelEn.svelte";
    import MethodologyPanelRu from "./methodology-panel/MethodologyPanelRu.svelte";
    import MethodologyPanelUk from "./methodology-panel/MethodologyPanelUk.svelte";

    const componentByLocale = {
        uk: MethodologyPanelUk,
        en: MethodologyPanelEn,
        ru: MethodologyPanelRu,
    } as const;

    let currentLocale = $derived($locale);
    let CurrentPanel = $derived(componentByLocale[currentLocale]);
</script>

<CurrentPanel />
