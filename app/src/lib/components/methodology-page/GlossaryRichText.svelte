<script lang="ts">
    import GlossaryTooltip from "./GlossaryTooltip.svelte";

    type Segment =
        | { type: "text"; value: string }
        | { type: "term"; value: string; definition: string };

    let { text, definitions = {} }: { text: string; definitions?: Record<string, string> } = $props();

    function isWordChar(char: string | undefined) {
        return Boolean(char && /[\p{L}\p{N}_]/u.test(char));
    }

    function requiresBoundary(alias: string) {
        return /^[\p{L}\p{N}_% ]+$/u.test(alias);
    }

    function matchesAt(source: string, alias: string, start: number) {
        const candidate = source.slice(start, start + alias.length);
        if (candidate.toLocaleLowerCase() !== alias.toLocaleLowerCase()) return false;
        if (!requiresBoundary(alias)) return true;

        const before = source[start - 1];
        const after = source[start + alias.length];
        return !isWordChar(before) && !isWordChar(after);
    }

    function buildSegments(source: string, defs: Record<string, string>): Segment[] {
        const aliases = Object.entries(defs)
            .filter(([, definition]) => Boolean(definition))
            .sort((a, b) => b[0].length - a[0].length);

        if (!aliases.length || !source) return [{ type: "text", value: source }];

        const segments: Segment[] = [];
        let cursor = 0;
        let buffer = "";

        while (cursor < source.length) {
            const match = aliases.find(([alias]) => matchesAt(source, alias, cursor));

            if (!match) {
                buffer += source[cursor];
                cursor += 1;
                continue;
            }

            if (buffer) {
                segments.push({ type: "text", value: buffer });
                buffer = "";
            }

            const [alias, definition] = match;
            segments.push({ type: "term", value: source.slice(cursor, cursor + alias.length), definition });
            cursor += alias.length;
        }

        if (buffer) {
            segments.push({ type: "text", value: buffer });
        }

        return segments;
    }

    let segments = $derived(buildSegments(text, definitions));
</script>

<span>
    {#each segments as segment}
        {#if segment.type === "term"}
            <GlossaryTooltip term={segment.value} definition={segment.definition} />
        {:else}
            {segment.value}
        {/if}
    {/each}
</span>
