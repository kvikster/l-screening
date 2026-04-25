<!--
  @file MethodologyVisualizerRu.svelte
  @description
  Русскоязычная интерактивная визуализация конвейера LC-MS скрининга.
  Двухпанельный макет: вертикальный степпер шагов и контентная карточка в пределах viewport.
  У каждого шага две вкладки: Обзор и Справка.
-->
<script lang="ts">
    import GlossaryRichText from "./GlossaryRichText.svelte";
    import MathFormula from "./MathFormula.svelte";
    import { fade } from 'svelte/transition';
    import { cubicOut } from 'svelte/easing';
    import { generateSampleData } from "$lib/screening";
    import { t } from "$lib/i18n";

    let { defs = {} }: { defs?: Record<string, string> } = $props();

    async function handleDownloadSample(format: "csv" | "xlsx") {
        try {
            await generateSampleData(format);
        } catch (e) {
            console.error(e);
        }
    }

    // ── Steps ────────────────────────────────────────────────────────────
    const steps = [
        {
            id: "glossary",
            title: "Глоссарий",
            short: "Ключевые термины и символы",
            tone: "slate",
            summary: "Этот шаг определяет ключевые LC-MS термины, используемые во всей методологии: RT, m/z, Blank, Replicate, S/B и Confidence score.",
            deepDive: "Используйте его как общий словарь перед просмотром самого конвейера. Те же термины дальше появляются как inline tooltip-подсказки во вкладках Обзор и Справка.",
            input: ["Базовые доменные термины", "Символы в формулах", "Сокращения для решений"],
            action: ["Просмотреть карточки глоссария", "Навести на подсвеченные термины", "Согласовать лексику для всех следующих шагов"],
            output: ["Единое толкование терминов методологии"],
            formula: "Термин -> Определение -> Согласованное толкование",
            formulaExplanation: "Глоссарий не преобразует данные. Он стандартизирует язык, которым описываются формулы, правила и решения на следующих шагах."
        },
        {
            id: "excel",
            title: "Входные данные и валидация",
            short: "Чтение Excel и очистка",
            tone: "slate",
            summary: "Рабочая книга считывается, и система автоматически выбирает лист с наибольшим количеством обязательных колонок (RT, Area, m/z). Все неполные строки отбрасываются.",
            deepDive: "Этот шаг обеспечивает целостность данных. Строки без базовых LC-MS параметров не допускаются к дальнейшему расчёту и отклоняются до любых научных выводов.",
            input: ["Рабочая книга (один или несколько листов)", "Строки пиков"],
            action: ["Выбрать лист с лучшим совпадением", "Удалить неполные строки"],
            output: ["Валидные строки для анализа"],
            formula: "Sheet = MaxMatch(Headers, Req)  AND  Valid = (RT>0 & Area>0)",
            formulaExplanation: "Мы ищем лист, где названия колонок лучше всего соответствуют требованиям. Остаются только строки с базовыми числовыми данными."
        },
        {
            id: "classify",
            title: "Назначение ролей",
            short: "Sample / Blank / Rep",
            tone: "violet",
            summary: "Каждой строке назначается тип образца (Sample или Blank), после чего она попадает в соответствующую корзину репликата. Далее все расчёты выполняются отдельно для каждой полярности.",
            deepDive: "Если оператор раскрасил ячейки в Excel, система доверяет этому цвету. Если цвета нет, используется разбор имени файла.",
            input: ["Валидные строки", "Цвета ячеек Excel", "Имена файлов"],
            action: ["Считать метки оператора", "Применить логику имени файла"],
            output: ["Группировка по (SampleType, Polarity)"],
            formula: "Role = ColorMap[CellColor] || FileNameLogic(Name)",
            formulaExplanation: "Цветовые метки имеют наивысший приоритет. Если их нет, система анализирует текстовое имя на наличие паттернов."
        },
        {
            id: "replicates",
            title: "Кластеризация репликатов",
            short: "Подтверждение пиков",
            tone: "blue",
            summary: "Пики одного образца, но из разных измерений, объединяются в группы. Мы проверяем, что одно и то же соединение присутствует в нескольких независимых измерениях.",
            deepDive: "Жадная кластеризация: пик с наибольшей площадью становится зерном. Из каждой другой корзины репликатов выбирается ближайший пик в пространстве (RT + m/z). Если совпадения найдены как минимум в 2 корзинах, кластер подтверждается. Алгоритм также поддерживает режим RT-only (GC-FID, LC-UV и другие приборы без масс-спектрометра): если в наборе данных отсутствует колонка «Base Peak», кластеризация и blank subtraction выполняются только по RT, а штраф confidence score за отсутствие m/z не применяется.",
            input: ["Пики в корзинах репликатов"],
            action: ["Сортировать по убыванию Area", "Усреднять центроид при добавлении", "Проверять окна допуска"],
            output: ["Подтверждённые кластеры"],
            formula: "|RT_cand \u2212 RT_cent| \u2264 Tol_RT  AND  \u0394m/z \u2264 Tol_m/z",
            formulaExplanation: "Чтобы объединить пики из разных файлов, их время удерживания и масса должны быть почти одинаковыми в пределах настроенных допусков."
        },
        {
            id: "blank",
            title: "Вычитание Blank",
            short: "Сравнение с фоном",
            tone: "cyan",
            summary: "Blank-пробы проходят независимую кластеризацию. Затем каждый подтверждённый sample сопоставляется с ближайшим blank-сигналом для оценки фонового шума.",
            deepDive: "Мы ищем ближайший кластер в blank по координатам (RT + m/z). Если расстояния равны, выбирается больший blank, чтобы фильтрация фона оставалась консервативной.",
            input: ["Sample-кластеры", "Blank-кластеры"],
            action: ["Найти ближайший blank-кластер той же полярности", "Вычислить S/B ratio"],
            output: ["Пары Sample ↔ Blank с ratio"],
            formula: "Ratio (S/B) = mean(Area_sample) / mean(Area_blank)",
            formulaExplanation: "Средняя площадь в образце делится на среднюю площадь в blank. Усреднение защищает от случайных всплесков."
        },
        {
            id: "parallel_merge",
            title: "Слияние параллельных проб",
            short: "sample_1 ∩ sample_2",
            tone: "indigo",
            summary: "Подтверждённые sample-кластеры из разных параллельных проб (sample_1, sample_2 …) сводятся в единую строку. Blank-статус каждой пробы уже известен до слияния.",
            deepDive: "Слияние взвешивается по количеству репликатов. Если только одна проба имела blank match, blank_area_mean агрегируется только по ней. S/B ratio и статус пересчитываются на агрегированном уровне, поэтому одна шумная проба не искажает итоговый вывод.",
            input: ["Per-sample кластеры со статусом blank subtraction"],
            action: ["Жадная кластеризация между пробами (RT + m/z)", "Взвешенное усреднение (area, RT, mz)", "Агрегация blank_area_mean по источникам с match", "Пересчёт S/B и confidence_score"],
            output: ["Merged ConfirmedRow с Why.BlankSubtraction.PerSource[]"],
            formula: "S/B_merged = Area_merged / blank_area_mean_weighted",
            formulaExplanation: "Агрегированный blank_area_mean рассчитывается взвешенным средним только по тем источникам, у которых был blank match. Пробы без match не занижают знаменатель и не маскируют реальный сигнал."
        },
        {
            id: "decision",
            title: "Классификация",
            short: "Artifact / Real Compound",
            tone: "green",
            summary: "Если найден соответствующий blank, а сигнал образца не превышает фон достаточно сильно по порогу S/B, он отклоняется как артефакт.",
            deepDive: "Это бинарное разделение является ключевым бизнес-результатом. Оно позволяет химику сосредоточиться только на реальных чистых соединениях и отбросить загрязнение.",
            input: ["Signal-to-Blank (S/B) ratio", "Порог решения"],
            action: ["Сравнить S/B ratio с порогом"],
            output: ["Решение: Artifact или Real Compound"],
            formula: "Status = (Ratio < Threshold) ? \"Artifact\" : \"Real Compound\"",
            formulaExplanation: "Если сигнал образца недостаточно явно выделяется на фоне blank, мы классифицируем его как Artifact."
        },
        {
            id: "output",
            title: "Итоги и аудит",
            short: "Аудиторский след",
            tone: "rose",
            summary: "Каждый шаг, ведущий к финальному выводу, вместе с итоговыми статистическими метриками сохраняется для регуляторного контроля.",
            deepDive: "Подробный журнал решений сохраняется в формате JSON. Также рассчитываются средние RT, m/z, площадь, вариабельность (CV%) и бонусы/штрафы показателя доверия.",
            input: ["Подтверждённые кластеры со статусом"],
            action: ["Вычислить CV% и средние", "Рассчитать confidence score", "Сериализовать decision trail"],
            output: ["Итоговая таблица (Excel для аудита)"],
            formula: "Score = 100 \u2212 \u03a3(Penalties) + Bonus",
            formulaExplanation: "Рейтинг надёжности от 0 до 100. Баллы снимаются за сдвиги массы/RT и разброс репликатов, а добавляются за идеальное отделение от blank."
        }
    ];

    // ── Reference data (tables, lists) ──────────────────────────────────
    const columns = [
        { col: "RT",        type: "number", desc: "Время удерживания хроматографического пика",              ex: "2.345" },
        { col: "Base Peak", type: "number", desc: "(Опционально) m/z доминирующего иона в масс-спектре. Отсутствие колонки активирует режим RT-only (GC-FID, LC-UV).", ex: "195.08" },
        { col: "Polarity",  type: "string", desc: "Полярность ионизации: positive / negative",               ex: "positive" },
        { col: "File",      type: "string", desc: "Имя файла, используемое для назначения репликатов",       ex: "1_pos.d" },
        { col: "Area",      type: "number", desc: "Площадь пика, пропорциональная содержанию аналита",       ex: "1250000" },
        { col: "Label",     type: "string", desc: "(Опционально) метка оператора или название соединения",   ex: "Caffeine" },
    ];

    const operatorMarks = [
        { name: "sample_rep1",     color: "#ff00ff", label: "Sample, репликат 1" },
        { name: "sample_rep2",     color: "#ffff00", label: "Sample, репликат 2" },
        { name: "blank_positive",  color: "#00ffff", label: "Blank" },
        { name: "blank_negative",  color: "#00ff00", label: "Blank" },
    ];

    const outputFields = [
        { field: "RT_mean",            desc: "Средний RT подтверждённого кластера по всем репликатам." },
        { field: "MZ_mean",            desc: "Средний m/z подтверждённого кластера." },
        { field: "Area_mean",          desc: "Средняя площадь пика без целочисленного округления." },
        { field: "AreaCVPct",          desc: "CV% между площадями репликатов в кластере." },
        { field: "ReplicateQuality",   desc: "High (CV% \u2264 15) / Moderate (\u2264 30) / Low (> 30) — уровень воспроизводимости." },
        { field: "SignalToBlankRatio", desc: "S/B ratio для ближайшего подтверждённого blank-пика." },
        { field: "ConfidenceScore",    desc: "Итоговый показатель доверия 0–100 после blank subtraction." },
        { field: "Status",             desc: "Результат классификации: Real Compound или Artifact." },
        { field: "Why",                desc: "JSON-объект с полным аудиторским следом решения (RT/mz, CV, S/B, пороги)." },
    ];

    const params = [
        { name: "replicate_rt_tol",   def: "0.1",  unit: "мин",     used: "Кластеризация репликатов" },
        { name: "replicate_mz_tol",   def: "0.3",  unit: "Da / ppm", used: "Кластеризация репликатов" },
        { name: "blank_rt_tol",       def: "0.1",  unit: "мин",     used: "Blank subtraction" },
        { name: "blank_mz_tol",       def: "0.3",  unit: "Da / ppm", used: "Blank subtraction" },
        { name: "signal_to_blank_min",   def: "3.0",  unit: "ratio",   used: "Решение Artifact / Real Compound" },
        { name: "min_area_difference",    def: "—",    unit: "counts",  used: "(Опционально) абсолютный порог AreaDiff; Artifact если area_sample−area_blank < порога" },
        { name: "cv_high_max",        def: "15",   unit: "%",       used: "ReplicateQuality = High" },
        { name: "cv_moderate_max",    def: "30",   unit: "%",       used: "ReplicateQuality = Moderate" },
        { name: "mz_available",       def: "true", unit: "bool",    used: "Автоопределяется: false для RT-only наборов данных (без колонки Base Peak)" },
    ];

    const glossary = [
        { term: "RT",               def: "Retention Time — время удерживания аналита в колонке, измеряется в минутах." },
        { term: "m/z",              def: "Mass-to-charge ratio — центральная координата масс-спектрометрического сигнала." },
        { term: "CV%",              def: "Coefficient of Variation — относительная вариабельность площадей пиков между репликатами. CV% = (sample std / mean) × 100. Для n=2: std = |v₁−v₂|/√2. Более низкий CV означает лучшую воспроизводимость." },
        { term: "S/B",              def: "Signal-to-Blank ratio — площадь sample-пика, делённая на площадь ближайшего подтверждённого blank-пика." },
        { term: "Blank",            def: "Холостая проба (только растворитель, без аналита), используемая для выявления фонового сигнала и загрязнения." },
        { term: "Replicate",        def: "Независимое повторное измерение одного и того же образца. Для подтверждения пика требуется как минимум 2 корзины репликатов." },
        { term: "Confidence score", def: "Показатель 0–100, который стартует со 100 и штрафуется за отклонения RT/m/z, CV% и S/B ratio." },
        { term: "ppm",              def: "Parts per million — относительный допуск по m/z для высокоразрешающих приборов." },
        { term: "Da",               def: "Dalton — абсолютный допуск по m/z в дальтонах." },
        { term: "Replicate bucket", def: "Все пики из одного файла или с одной операторской меткой. Каждый файл/метка формирует отдельную корзину." },
    ];

    let resolvedGlossary = $derived({
        ...Object.fromEntries(glossary.map(({ term, def }) => [term, def])),
        ...defs
    });

    let glossaryDefinitions = $derived({
        ...resolvedGlossary,
        blank: resolvedGlossary.Blank,
        "blank-проба": resolvedGlossary.Blank,
        "blank-пробы": resolvedGlossary.Blank,
        "blank-пик": resolvedGlossary.Blank,
        "blank-пики": resolvedGlossary.Blank,
        replicate: resolvedGlossary.Replicate,
        "репликат": resolvedGlossary.Replicate,
        "репликаты": resolvedGlossary.Replicate,
        "корзина репликатов": resolvedGlossary["Replicate bucket"],
        "корзины репликатов": resolvedGlossary["Replicate bucket"],
        "confidence score": resolvedGlossary["Confidence score"],
        "показатель доверия": resolvedGlossary["Confidence score"],
        "S/B ratio": resolvedGlossary["S/B"],
        ratio: resolvedGlossary["S/B"]
    });

    const formulaMarkupById: Record<string, string> = {
        glossary: '<math xmlns="http://www.w3.org/1998/Math/MathML" display="block"><mrow><mtext>Термин</mtext><mspace width="0.4em"/><mo>→</mo><mspace width="0.4em"/><mtext>Определение</mtext><mspace width="0.4em"/><mo>→</mo><mspace width="0.4em"/><mtext>Согласованное толкование</mtext></mrow></math>',
        excel: '<math xmlns="http://www.w3.org/1998/Math/MathML" display="block"><mtable columnalign="left"><mtr><mtd><mi>Sheet</mi><mo>=</mo><mi>MaxMatch</mi><mo>(</mo><mi>Headers</mi><mo>,</mo><mi>Req</mi><mo>)</mo></mtd></mtr><mtr><mtd><mi>Valid</mi><mo>=</mo><mo>(</mo><mi>RT</mi><mo>&gt;</mo><mn>0</mn><mspace width="0.3em"/><mo>∧</mo><mspace width="0.3em"/><mi>Area</mi><mo>&gt;</mo><mn>0</mn><mo>)</mo></mtd></mtr></mtable></math>',
        classify: '<math xmlns="http://www.w3.org/1998/Math/MathML" display="block"><mrow><mi>Role</mi><mo>=</mo><mi>ColorMap</mi><mo>(</mo><mi>CellColor</mi><mo>)</mo><mspace width="0.6em"/><mo>∨</mo><mspace width="0.6em"/><mi>FileNameLogic</mi><mo>(</mo><mi>Name</mi><mo>)</mo></mrow></math>',
        replicates: '<math xmlns="http://www.w3.org/1998/Math/MathML" display="block"><mtable columnalign="left"><mtr><mtd><mo>|</mo><msub><mi>RT</mi><mtext>cand</mtext></msub><mo>−</mo><msub><mi>RT</mi><mtext>cent</mtext></msub><mo>|</mo><mo>≤</mo><msub><mi>Tol</mi><mtext>RT</mtext></msub></mtd></mtr><mtr><mtd><mi>Δm/z</mi><mo>≤</mo><msub><mi>Tol</mi><mtext>m/z</mtext></msub></mtd></mtr></mtable></math>',
        blank: '<math xmlns="http://www.w3.org/1998/Math/MathML" display="block"><mtable columnalign="left"><mtr><mtd><mi>Ratio</mi><mo>(</mo><mi>S</mi><mo>/</mo><mi>B</mi><mo>)</mo><mo>=</mo></mtd></mtr><mtr><mtd><mfrac><mrow><mi>mean</mi><mo>(</mo><msub><mi>Area</mi><mtext>sample</mtext></msub><mo>)</mo></mrow><mrow><mi>mean</mi><mo>(</mo><msub><mi>Area</mi><mtext>blank</mtext></msub><mo>)</mo></mrow></mfrac></mtd></mtr></mtable></math>',
        decision: '<math xmlns="http://www.w3.org/1998/Math/MathML" display="block"><mrow><mi>Status</mi><mo>=</mo><mrow><mo>{</mo><mtable columnalign="left left" rowspacing="0.2em"><mtr><mtd><mtext>Artifact,</mtext></mtd><mtd><mtext>если Ratio &lt; Threshold</mtext></mtd></mtr><mtr><mtd><mtext>Real Compound,</mtext></mtd><mtd><mtext>иначе</mtext></mtd></mtr></mtable></mrow></mrow></math>',
        output: '<math xmlns="http://www.w3.org/1998/Math/MathML" display="block"><mrow><mi>Score</mi><mo>=</mo><mn>100</mn><mo>−</mo><mo>∑</mo><mo>(</mo><mi>Penalties</mi><mo>)</mo><mo>+</mo><mi>Bonus</mi></mrow></math>',
    };

    // ── State ────────────────────────────────────────────────────────────
    let current = $state(0);
    let tab: 'overview' | 'reference' = $state('overview');
    let activeStep = $derived(steps[current]);
    let activeFormulaMarkup = $derived(formulaMarkupById[activeStep.id] ?? '');
    let contentEl: HTMLDivElement | undefined = $state();

    let lastScrollTime = 0;
    const scrollThrottle = 400;
    const scrollEdgeThreshold = 2; // px tolerance for "at boundary" checks

    function handleWheel(e: WheelEvent) {
        if (Math.abs(e.deltaX) > Math.abs(e.deltaY)) return;
        if (Math.abs(e.deltaY) <= 10) return;

        // If the content area is scrollable, only navigate steps at scroll boundaries
        if (contentEl) {
            const { scrollTop, scrollHeight, clientHeight } = contentEl;
            const isScrollable = scrollHeight > clientHeight + scrollEdgeThreshold;
            const atTop = scrollTop <= scrollEdgeThreshold;
            const atBottom = scrollTop + clientHeight >= scrollHeight - scrollEdgeThreshold;

            if (isScrollable) {
                if (e.deltaY > 0 && !atBottom) return;  // let content scroll down
                if (e.deltaY < 0 && !atTop) return;     // let content scroll up
            }
        }

        const now = Date.now();
        if (now - lastScrollTime < scrollThrottle) return;

        if (e.deltaY > 0 && current < steps.length - 1) {
            current++;
            lastScrollTime = now;
            e.preventDefault();
            if (contentEl) contentEl.scrollTop = 0;
        } else if (e.deltaY < 0 && current > 0) {
            current--;
            lastScrollTime = now;
            e.preventDefault();
            if (contentEl) contentEl.scrollTop = 0;
        }
    }

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === 'ArrowDown' || e.key === 'ArrowRight') {
            if (current < steps.length - 1) { current++; e.preventDefault(); scrollContentToTop(); }
        } else if (e.key === 'ArrowUp' || e.key === 'ArrowLeft') {
            if (current > 0) { current--; e.preventDefault(); scrollContentToTop(); }
        }
    }

    function scrollContentToTop() {
        if (contentEl) contentEl.scrollTop = 0;
    }

    const toneBadge: Record<string, string> = {
        slate:  "border-slate-300 bg-slate-50 text-slate-700 dark:border-slate-600 dark:bg-slate-800 dark:text-slate-300",
        violet: "border-violet-300 bg-violet-50 text-violet-700 dark:border-violet-700 dark:bg-violet-900/30 dark:text-violet-300",
        blue:   "border-blue-300 bg-blue-50 text-blue-700 dark:border-blue-700 dark:bg-blue-900/30 dark:text-blue-300",
        cyan:   "border-cyan-300 bg-cyan-50 text-cyan-700 dark:border-cyan-700 dark:bg-cyan-900/30 dark:text-cyan-300",
        green:  "border-emerald-300 bg-emerald-50 text-emerald-700 dark:border-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-300",
        rose:   "border-rose-300 bg-rose-50 text-rose-700 dark:border-rose-700 dark:bg-rose-900/30 dark:text-rose-300",
    };

    const toneDot: Record<string, string> = {
        slate:  "border-slate-400 bg-slate-400",
        violet: "border-violet-500 bg-violet-500",
        blue:   "border-blue-500 bg-blue-500",
        cyan:   "border-cyan-500 bg-cyan-500",
        green:  "border-emerald-500 bg-emerald-500",
        rose:   "border-rose-500 bg-rose-500",
    };
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
    onwheel={handleWheel}
    onkeydown={handleKeydown}
    role="region"
    aria-label="Methodology visualizer"
    tabindex="0"
    class="flex h-[calc(100dvh-10rem)] min-h-[520px] max-h-[800px] overflow-hidden rounded-3xl border border-slate-200 bg-white shadow-xl shadow-slate-200/40 focus:outline-none dark:border-slate-700/60 dark:bg-slate-900/80 dark:shadow-none"
>
    <!-- ================================================================= -->
    <!-- LEFT: Vertical Stepper Nav                                        -->
    <!-- ================================================================= -->
    <nav class="flex w-52 shrink-0 flex-col border-r border-slate-100 bg-slate-50/60 dark:border-slate-800 dark:bg-slate-900/60">
        <div class="px-4 pt-5 pb-2">
            <h2 class="text-[10px] font-bold uppercase tracking-[0.15em] text-slate-400 dark:text-slate-500">Конвейер</h2>
        </div>

        <div class="flex-1 overflow-y-auto px-4 pb-4">
            <div class="relative">
                <div class="absolute left-[11px] top-[14px] bottom-[14px] w-px bg-slate-200 dark:bg-slate-700"></div>
                <div
                    class="absolute left-[11px] top-[14px] w-px bg-blue-500 transition-all duration-500 ease-out"
                    style="height: {steps.length > 1 ? (current / (steps.length - 1)) * 100 : 0}%"
                ></div>

                {#each steps as st, i}
                    <button
                        onclick={() => (current = i)}
                        class="group relative flex w-full items-start gap-3 py-2.5 text-left transition-colors"
                    >
                        <div class="relative z-10 flex h-[22px] w-[22px] shrink-0 items-center justify-center rounded-full border-2 transition-all duration-300
                            {current === i
                                ? `${toneDot[st.tone] ?? 'border-blue-500 bg-blue-500'} text-white shadow-md scale-110`
                                : i < current
                                    ? 'border-blue-400 bg-blue-50 text-blue-600 dark:border-blue-500 dark:bg-blue-900/40 dark:text-blue-400'
                                    : 'border-slate-300 bg-white text-slate-400 group-hover:border-slate-400 dark:border-slate-600 dark:bg-slate-800 dark:text-slate-500'
                            }">
                            <span class="text-[9px] font-bold">{i}</span>
                        </div>
                        <div class="min-w-0 pt-px">
                            <p class="truncate text-[11px] font-semibold leading-tight transition-colors
                                {current === i ? 'text-slate-900 dark:text-white' : 'text-slate-500 dark:text-slate-400 group-hover:text-slate-700 dark:group-hover:text-slate-300'}">
                                <GlossaryRichText text={st.title} definitions={glossaryDefinitions} />
                            </p>
                            <p class="mt-0.5 truncate text-[10px] text-slate-400 dark:text-slate-500">{st.short}</p>
                        </div>
                    </button>
                {/each}
            </div>
        </div>

        <div class="shrink-0 border-t border-slate-100 px-4 py-2.5 dark:border-slate-800">
            <p class="text-center text-[10px] text-slate-400 dark:text-slate-500">
                <kbd class="rounded border border-slate-200 bg-white px-1 py-0.5 font-mono text-[9px] dark:border-slate-700 dark:bg-slate-800">&uarr;</kbd>
                <kbd class="rounded border border-slate-200 bg-white px-1 py-0.5 font-mono text-[9px] dark:border-slate-700 dark:bg-slate-800">&darr;</kbd>
                или scroll
            </p>
        </div>
    </nav>

    <!-- ================================================================= -->
    <!-- RIGHT: Content Panel                                              -->
    <!-- ================================================================= -->
    <div class="flex flex-1 flex-col min-w-0">
        <!-- Progress bar -->
        <div class="h-1 shrink-0 bg-slate-100 dark:bg-slate-800">
            <div class="h-full bg-gradient-to-r from-blue-500 to-indigo-500 transition-all duration-500 ease-out" style="width: {steps.length > 1 ? (current / (steps.length - 1)) * 100 : 0}%"></div>
        </div>

        <!-- Tab switcher -->
        <div class="shrink-0 flex items-center gap-1 border-b border-slate-100 px-5 pt-3 pb-0 dark:border-slate-800">
            <button
                onclick={() => (tab = 'overview')}
                class="rounded-t-lg px-3 py-1.5 text-xs font-semibold transition-colors
                    {tab === 'overview'
                        ? 'border-b-2 border-blue-500 text-blue-600 dark:text-blue-400'
                        : 'text-slate-400 hover:text-slate-600 dark:text-slate-500 dark:hover:text-slate-300'}"
            >
                Обзор
            </button>
            <button
                onclick={() => (tab = 'reference')}
                class="rounded-t-lg px-3 py-1.5 text-xs font-semibold transition-colors
                    {tab === 'reference'
                        ? 'border-b-2 border-blue-500 text-blue-600 dark:text-blue-400'
                        : 'text-slate-400 hover:text-slate-600 dark:text-slate-500 dark:hover:text-slate-300'}"
            >
                Справка
            </button>
        </div>

        <!-- Content area -->
        <div bind:this={contentEl} class="flex-1 overflow-y-auto p-5 lg:p-6">
            {#key `${current}-${tab}`}
            <div in:fade={{ duration: 200, easing: cubicOut }}>

                <!-- Header (shared) -->
                <div class="mb-3 flex items-center gap-3">
                    <span class="inline-flex items-center rounded-full border px-2.5 py-0.5 font-mono text-[10px] font-bold uppercase tracking-widest {toneBadge[activeStep.tone] ?? toneBadge.slate}">
                        {activeStep.id}
                    </span>
                    <h3 class="text-lg font-extrabold tracking-tight text-slate-900 lg:text-xl dark:text-white">
                        <GlossaryRichText text={activeStep.title} definitions={glossaryDefinitions} />
                    </h3>
                </div>

                <!-- ====================================================== -->
                <!-- TAB: Overview                                           -->
                <!-- ====================================================== -->
                {#if tab === 'overview'}
                    <p class="mb-4 text-sm leading-relaxed text-slate-600 dark:text-slate-300">
                        <GlossaryRichText text={activeStep.summary} definitions={glossaryDefinitions} />
                    </p>

                    <!-- I/O Grid -->
                    <div class="mb-4 grid grid-cols-3 gap-2.5">
                        <div class="rounded-xl border border-slate-100 bg-slate-50/60 p-3 dark:border-slate-800 dark:bg-slate-800/30">
                            <p class="mb-1.5 text-[9px] font-bold uppercase tracking-widest text-slate-400 dark:text-slate-500">Вход</p>
                            <ul class="space-y-1">
                                {#each activeStep.input as item}
                                    <li class="flex items-start gap-1.5 text-[11px] leading-snug text-slate-600 dark:text-slate-300">
                                        <span class="mt-[5px] h-1 w-1 shrink-0 rounded-full bg-slate-300 dark:bg-slate-600"></span>
                                        <GlossaryRichText text={item} definitions={glossaryDefinitions} />
                                    </li>
                                {/each}
                            </ul>
                        </div>
                        <div class="rounded-xl border-l-2 border-blue-400 bg-blue-50/40 p-3 dark:bg-blue-900/10">
                            <p class="mb-1.5 text-[9px] font-bold uppercase tracking-widest text-blue-500 dark:text-blue-400">Процесс</p>
                            <ul class="space-y-1">
                                {#each activeStep.action as item}
                                    <li class="flex items-start gap-1.5 text-[11px] font-medium leading-snug text-slate-700 dark:text-slate-200">
                                        <svg class="mt-0.5 h-3 w-3 shrink-0 text-blue-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5"><path stroke-linecap="round" stroke-linejoin="round" d="M13 7l5 5m0 0l-5 5m5-5H6"></path></svg>
                                        <GlossaryRichText text={item} definitions={glossaryDefinitions} />
                                    </li>
                                {/each}
                            </ul>
                        </div>
                        <div class="rounded-xl border border-emerald-100 bg-emerald-50/40 p-3 dark:border-emerald-900/30 dark:bg-emerald-900/10">
                            <p class="mb-1.5 text-[9px] font-bold uppercase tracking-widest text-emerald-500 dark:text-emerald-400">Выход</p>
                            <ul class="space-y-1">
                                {#each activeStep.output as item}
                                    <li class="flex items-start gap-1.5 text-[11px] leading-snug text-slate-600 dark:text-slate-300">
                                        <svg class="mt-0.5 h-3 w-3 shrink-0 text-emerald-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5"><path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7"></path></svg>
                                        <GlossaryRichText text={item} definitions={glossaryDefinitions} />
                                    </li>
                                {/each}
                            </ul>
                        </div>
                    </div>

                    <!-- Detail + Math -->
                    <div class="grid gap-2.5 lg:grid-cols-[minmax(0,0.9fr)_minmax(0,1.1fr)]">
                        <div class="rounded-xl border border-slate-100 bg-slate-50/50 p-3.5 dark:border-slate-800 dark:bg-slate-800/20">
                            <h4 class="mb-1 flex items-center gap-1.5 text-[10px] font-bold uppercase tracking-wider text-slate-500 dark:text-slate-400">
                                <svg class="h-3.5 w-3.5 text-indigo-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
                                Детально
                            </h4>
                            <p class="text-[11px] leading-[1.35rem] text-slate-600 dark:text-slate-400"><GlossaryRichText text={activeStep.deepDive} definitions={glossaryDefinitions} /></p>
                        </div>
                        <div class="rounded-xl border border-indigo-100 bg-indigo-50/30 p-4 dark:border-indigo-900/30 dark:bg-indigo-900/10">
                            <h4 class="mb-1.5 text-[10px] font-bold uppercase tracking-wider text-indigo-600 dark:text-indigo-400">Математика и логика</h4>
                            <div class="mb-1.5">
                                <MathFormula markup={activeFormulaMarkup} label={activeStep.formula} />
                            </div>
                            <p class="text-xs leading-5 text-slate-600 dark:text-slate-400"><GlossaryRichText text={activeStep.formulaExplanation} definitions={glossaryDefinitions} /></p>
                        </div>
                    </div>

                <!-- ====================================================== -->
                <!-- TAB: Reference                                          -->
                <!-- ====================================================== -->
                {:else}
                    <div class="space-y-5 text-xs leading-relaxed text-slate-600 dark:text-slate-400">

                        {#if activeStep.id === 'glossary'}
                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Общая терминология</h4>
                                <p><GlossaryRichText text="Глоссарий вынесен в шаг 0, потому что все следующие этапы используют те же термины в резюме, правилах, формулах и финальном аудите." definitions={glossaryDefinitions} /></p>
                            </div>

                            <div class="grid grid-cols-1 gap-1.5 sm:grid-cols-2">
                                {#each glossary as g}
                                    <div class="rounded-lg border border-slate-100 bg-slate-50/50 p-2.5 dark:border-slate-800 dark:bg-slate-800/20">
                                        <span class="font-mono font-semibold text-blue-700 dark:text-blue-400">{g.term}</span>
                                        <p class="mt-0.5 text-[10px] leading-snug text-slate-500 dark:text-slate-400"><GlossaryRichText text={g.def} definitions={glossaryDefinitions} /></p>
                                    </div>
                                {/each}
                            </div>

                        <!-- Step 1: Input & Validation -->
                        {:else if activeStep.id === 'excel'}
                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Входные данные</h4>
                                <p><GlossaryRichText text="Система принимает Excel (.xlsx / .xls) или CSV/TSV/TXT файлы с LC-MS пиковыми данными, автоматически выбирает наиболее подходящий лист (для Excel) или парсит первый лист (для CSV), и ожидает как минимум два репликата плюс одну blank-пробу." definitions={glossaryDefinitions} /></p>
                                <div class="mt-3 flex gap-2">
                                    <button onclick={() => handleDownloadSample("csv")} class="rounded-lg border border-slate-200 bg-slate-50 px-3 py-1.5 text-xs font-medium text-slate-700 hover:bg-slate-100 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-300 dark:hover:bg-slate-700">
                                        {t("downloadSampleCsv")}
                                    </button>
                                    <button onclick={() => handleDownloadSample("xlsx")} class="rounded-lg border border-slate-200 bg-slate-50 px-3 py-1.5 text-xs font-medium text-slate-700 hover:bg-slate-100 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-300 dark:hover:bg-slate-700">
                                        {t("downloadSampleXlsx")}
                                    </button>
                                </div>
                            </div>

                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Обязательные колонки Excel</h4>
                                <div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
                                    <table class="w-full text-[11px]">
                                        <thead class="bg-slate-50 dark:bg-slate-800">
                                            <tr>
                                                <th class="px-3 py-2 text-left font-semibold text-slate-700 dark:text-slate-300">Колонка</th>
                                                <th class="px-3 py-2 text-left font-semibold text-slate-700 dark:text-slate-300">Тип</th>
                                                <th class="px-3 py-2 text-left font-semibold text-slate-700 dark:text-slate-300">Описание</th>
                                                <th class="px-3 py-2 text-left font-semibold text-slate-700 dark:text-slate-300">Пример</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-slate-100 dark:divide-slate-700">
                                            {#each columns as c}
                                                <tr>
                                                    <td class="px-3 py-1.5 font-mono font-semibold text-blue-700 dark:text-blue-400">{c.col}</td>
                                                    <td class="px-3 py-1.5 text-slate-500 dark:text-slate-400">{c.type}</td>
                                                    <td class="px-3 py-1.5"><GlossaryRichText text={c.desc} definitions={glossaryDefinitions} /></td>
                                                    <td class="px-3 py-1.5 font-mono text-slate-500 dark:text-slate-400">{c.ex}</td>
                                                </tr>
                                            {/each}
                                        </tbody>
                                    </table>
                                </div>
                            </div>

                        <!-- Step 2: Role Assignment -->
                        {:else if activeStep.id === 'classify'}
                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Метки оператора</h4>
                                <p class="mb-3"><GlossaryRichText text='Цветовые метки ячеек имеют приоритет над именем файла. Без меток тип выводится из имени файла: файлы с "blank" становятся blank; файлы вида 1_*.d, 2_*.d становятся sample_1, sample_2 и т.д.' definitions={glossaryDefinitions} /></p>
                                <div class="grid grid-cols-2 gap-2">
                                    {#each operatorMarks as m}
                                        <div class="flex items-center gap-2.5 rounded-lg border border-slate-200 bg-slate-50 p-2.5 dark:border-slate-700 dark:bg-slate-800">
                                            <span class="h-4 w-4 shrink-0 rounded-full border border-black/10 dark:border-white/10" style="background:{m.color}"></span>
                                            <div>
                                                <span class="font-mono font-semibold text-slate-900 dark:text-slate-100">{m.name}</span>
                                                <span class="ml-1.5 text-slate-400">{m.label}</span>
                                            </div>
                                        </div>
                                    {/each}
                                </div>
                            </div>

                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Группировка</h4>
                                <p><GlossaryRichText text="После назначения ролей строки группируются по паре (SampleType, Polarity). Каждая группа далее обрабатывается независимо через кластеризацию и blank subtraction." definitions={glossaryDefinitions} /></p>
                            </div>

                        <!-- Step 3: Replicate Clustering -->
                        {:else if activeStep.id === 'replicates'}
                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Жадная кластеризация репликатов</h4>
                                <p><GlossaryRichText text="Пики сортируются по убыванию площади. Каждый пик по очереди становится зерном кластера. Из каждой другой корзины жадно выбирается ближайший неиспользованный пик к текущему центроиду (RT и m/z усредняются при добавлении членов). Кластер подтверждается, когда охватывает 2 или более разные корзины." definitions={glossaryDefinitions} /></p>
                            </div>

                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Правило включения</h4>
                                <div class="rounded-lg border border-blue-100 bg-blue-50/50 p-3 font-mono text-[11px] dark:border-blue-900/40 dark:bg-blue-900/10">
                                    <p>|RT_candidate &minus; RT_centroid| &le; replicate_rt_tol</p>
                                    <p>|mz_candidate &minus; mz_centroid| &le; replicate_mz_tol (Da или ppm)</p>
                                </div>
                                <p class="mt-2"><GlossaryRichText text="При равенстве выбирается минимальная дистанция (RT fraction + mz fraction), затем большая площадь." definitions={glossaryDefinitions} /></p>
                            </div>

                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Параметры</h4>
                                <div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
                                    <table class="w-full text-[11px]">
                                        <thead class="bg-slate-50 dark:bg-slate-800">
                                            <tr>
                                                <th class="px-3 py-2 text-left font-semibold">Параметр</th>
                                                <th class="px-3 py-2 text-left font-semibold">По умолч.</th>
                                                <th class="px-3 py-2 text-left font-semibold">Единица</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-slate-100 dark:divide-slate-700">
                                            {#each params.filter(p => p.used === 'Кластеризация репликатов') as p}
                                                <tr>
                                                    <td class="px-3 py-1.5 font-mono font-semibold text-blue-700 dark:text-blue-400">{p.name}</td>
                                                    <td class="px-3 py-1.5">{p.def}</td>
                                                    <td class="px-3 py-1.5 text-slate-500">{p.unit}</td>
                                                </tr>
                                            {/each}
                                        </tbody>
                                    </table>
                                </div>
                            </div>

                        <!-- Step 4: Blank Subtraction -->
                        {:else if activeStep.id === 'blank'}
                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Blank subtraction</h4>
                                <p><GlossaryRichText text="Blank проходит ту же кластеризацию независимо. Каждый подтверждённый sample-пик затем сопоставляется с подтверждёнными blank-пиками в пределах blank_rt_tol / blank_mz_tol. Выбирается ближайшее совпадение по RT+m/z, а при равенстве — больший blank." definitions={glossaryDefinitions} /></p>
                            </div>

                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Правила классификации</h4>
                                <div class="space-y-1.5">
                                    <div class="flex items-start gap-2 rounded-lg border border-rose-100 bg-rose-50/50 p-2.5 dark:border-rose-900/30 dark:bg-rose-900/10">
                                        <span class="mt-0.5 font-mono font-bold text-rose-600 dark:text-rose-400">Artifact</span>
                                        <span><GlossaryRichText text={"найдено blank-совпадение и S/B < signal_to_blank_min (или площадь blank = 0)"} definitions={glossaryDefinitions} /></span>
                                    </div>
                                    <div class="flex items-start gap-2 rounded-lg border border-emerald-100 bg-emerald-50/50 p-2.5 dark:border-emerald-900/30 dark:bg-emerald-900/10">
                                        <span class="mt-0.5 font-mono font-bold text-emerald-600 dark:text-emerald-400">Real Compound</span>
                                        <span><GlossaryRichText text={"blank-совпадения нет или S/B >= порог"} definitions={glossaryDefinitions} /></span>
                                    </div>
                                </div>
                            </div>

                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Параметры</h4>
                                <div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
                                    <table class="w-full text-[11px]">
                                        <thead class="bg-slate-50 dark:bg-slate-800">
                                            <tr>
                                                <th class="px-3 py-2 text-left font-semibold">Параметр</th>
                                                <th class="px-3 py-2 text-left font-semibold">По умолч.</th>
                                                <th class="px-3 py-2 text-left font-semibold">Единица</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-slate-100 dark:divide-slate-700">
                                            {#each params.filter(p => p.used.includes('Blank') || p.used.includes('Artifact')) as p}
                                                <tr>
                                                    <td class="px-3 py-1.5 font-mono font-semibold text-blue-700 dark:text-blue-400">{p.name}</td>
                                                    <td class="px-3 py-1.5">{p.def}</td>
                                                    <td class="px-3 py-1.5 text-slate-500">{p.unit}</td>
                                                </tr>
                                            {/each}
                                        </tbody>
                                    </table>
                                </div>
                            </div>

                        <!-- Step 5: Classification / Confidence Score -->
                        {:else if activeStep.id === 'decision'}
                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Формула Confidence Score</h4>
                                <p><GlossaryRichText text="Стартует со 100. Штрафы применяются последовательно, а результат ограничивается диапазоном [0, 100]." definitions={glossaryDefinitions} /></p>
                            </div>

                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Этап 1 &mdash; Репликаты</h4>
                                <div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
                                    <table class="w-full text-[11px]">
                                        <thead class="bg-slate-50 dark:bg-slate-800">
                                            <tr>
                                                <th class="px-3 py-2 text-left font-semibold">Фактор</th>
                                                <th class="px-3 py-2 text-left font-semibold">Штраф</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-slate-100 dark:divide-slate-700">
                                            <tr><td class="px-3 py-1.5">Близость RT</td><td class="px-3 py-1.5 font-mono">&minus;(mean_RT_delta / rt_tol) &times; 20, max &minus;20</td></tr>
                                            <tr><td class="px-3 py-1.5">Близость m/z</td><td class="px-3 py-1.5 font-mono">&minus;(mean_mz_delta / mz_tol) &times; 25, max &minus;25</td></tr>
                                            <tr><td class="px-3 py-1.5">CV% High</td><td class="px-3 py-1.5 font-mono">0</td></tr>
                                            <tr><td class="px-3 py-1.5">CV% Moderate</td><td class="px-3 py-1.5 font-mono">&minus;12</td></tr>
                                            <tr><td class="px-3 py-1.5">CV% Low</td><td class="px-3 py-1.5 font-mono">&minus;(12 + (CV &minus; cv_moderate_max) &times; 0.7), max &minus;35</td></tr>
                                            <tr><td class="px-3 py-1.5">CV% отсутствует</td><td class="px-3 py-1.5 font-mono">&minus;10</td></tr>
                                            <tr><td class="px-3 py-1.5">Не сведено по цвету</td><td class="px-3 py-1.5 font-mono">&minus;5</td></tr>
                                        </tbody>
                                    </table>
                                </div>
                            </div>

                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Этап 2 &mdash; Blank</h4>
                                <div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
                                    <table class="w-full text-[11px]">
                                        <thead class="bg-slate-50 dark:bg-slate-800">
                                            <tr>
                                                <th class="px-3 py-2 text-left font-semibold">Условие</th>
                                                <th class="px-3 py-2 text-left font-semibold">Коррекция</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-slate-100 dark:divide-slate-700">
                                            <tr><td class="px-3 py-1.5">Нет blank-совпадения</td><td class="px-3 py-1.5 font-mono text-emerald-600 dark:text-emerald-400">+3</td></tr>
                                            <tr><td class="px-3 py-1.5">S/B &ge; порог</td><td class="px-3 py-1.5 font-mono text-emerald-600 dark:text-emerald-400">+min(5, (S/B &minus; threshold) &times; 0.5)</td></tr>
                                            <tr><td class="px-3 py-1.5">S/B &lt; порог (Artifact)</td><td class="px-3 py-1.5 font-mono text-rose-600 dark:text-rose-400">&minus;(15 + deficit &times; 30), max &minus;45</td></tr>
                                        </tbody>
                                    </table>
                                </div>
                            </div>

                        <!-- Step 6: Output & Audit -->
                        {:else if activeStep.id === 'output'}
                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Выходные поля</h4>
                                <div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
                                    <table class="w-full text-[11px]">
                                        <thead class="bg-slate-50 dark:bg-slate-800">
                                            <tr>
                                                <th class="px-3 py-2 text-left font-semibold">Поле</th>
                                                <th class="px-3 py-2 text-left font-semibold">Описание</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-slate-100 dark:divide-slate-700">
                                            {#each outputFields as f}
                                                <tr>
                                                    <td class="px-3 py-1.5 font-mono font-semibold text-blue-700 dark:text-blue-400 whitespace-nowrap">{f.field}</td>
                                                    <td class="px-3 py-1.5"><GlossaryRichText text={f.desc} definitions={glossaryDefinitions} /></td>
                                                </tr>
                                            {/each}
                                        </tbody>
                                    </table>
                                </div>
                            </div>

                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Все параметры</h4>
                                <div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
                                    <table class="w-full text-[11px]">
                                        <thead class="bg-slate-50 dark:bg-slate-800">
                                            <tr>
                                                <th class="px-3 py-2 text-left font-semibold">Параметр</th>
                                                <th class="px-3 py-2 text-left font-semibold">По умолч.</th>
                                                <th class="px-3 py-2 text-left font-semibold">Единица</th>
                                                <th class="px-3 py-2 text-left font-semibold">Используется в</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-slate-100 dark:divide-slate-700">
                                            {#each params as p}
                                                <tr>
                                                    <td class="px-3 py-1.5 font-mono font-semibold text-blue-700 dark:text-blue-400">{p.name}</td>
                                                    <td class="px-3 py-1.5">{p.def}</td>
                                                    <td class="px-3 py-1.5 text-slate-500">{p.unit}</td>
                                                    <td class="px-3 py-1.5"><GlossaryRichText text={p.used} definitions={glossaryDefinitions} /></td>
                                                </tr>
                                            {/each}
                                        </tbody>
                                    </table>
                                </div>
                                <p class="mt-2 text-[10px] text-slate-400"><GlossaryRichText text="Параметры RT/m/z/S/B можно изменять на основной форме. cv_high_max и cv_moderate_max сейчас фиксированы." definitions={glossaryDefinitions} /></p>
                            </div>
                        {/if}
                    </div>
                {/if}

            </div>
            {/key}
        </div>

        <!-- Bottom navigation -->
        <div class="shrink-0 flex items-center justify-between border-t border-slate-100 px-5 py-2.5 dark:border-slate-800">
            <button
                onclick={() => current > 0 && current--}
                class="flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-xs font-semibold text-slate-500 transition-colors hover:bg-slate-100 hover:text-slate-900 disabled:pointer-events-none disabled:opacity-30 dark:text-slate-400 dark:hover:bg-slate-800 dark:hover:text-white"
                disabled={current === 0}
            >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none"><path d="M15 18L9 12L15 6" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>
                Назад
            </button>

            <div class="flex items-center gap-1.5">
                {#each steps as _, i}
                    <button
                        onclick={() => (current = i)}
                        class="h-1.5 rounded-full transition-all duration-300
                            {current === i ? 'w-5 bg-blue-500' : 'w-1.5 bg-slate-300 hover:bg-slate-400 dark:bg-slate-600 dark:hover:bg-slate-500'}"
                        aria-label="Перейти к шагу {i}"
                    ></button>
                {/each}
            </div>

            <button
                onclick={() => current < steps.length - 1 && current++}
                class="flex items-center gap-1.5 rounded-lg bg-slate-900 px-4 py-1.5 text-xs font-semibold text-white shadow-sm transition-transform hover:scale-105 disabled:pointer-events-none disabled:opacity-30 dark:bg-white dark:text-slate-900"
                disabled={current === steps.length - 1}
            >
                Далее
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none"><path d="M9 18L15 12L9 6" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>
            </button>
        </div>
    </div>
</div>
