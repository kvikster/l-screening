<!--
  @file MethodologyVisualizerUk.svelte
  @description
  Україномовна інтерактивна візуалізація конвеєра LC-MS скринінгу.
  Двопанельний макет: вертикальний степер кроків + контентна картка в межах viewport.
  Кожен крок має дві вкладки: Огляд і Довідка.
-->
<script lang="ts">
    import GlossaryRichText from "./GlossaryRichText.svelte";
    import MathFormula from "./MathFormula.svelte";
    import { fade } from 'svelte/transition';
    import { cubicOut } from 'svelte/easing';

    let { defs = {} }: { defs?: Record<string, string> } = $props();

    // ── Steps ────────────────────────────────────────────────────────────
    const steps = [
        {
            id: "glossary",
            title: "Глосарій",
            short: "Ключові терміни й символи",
            tone: "slate",
            summary: "Цей крок визначає ключові LC-MS терміни, що використовуються в усій методології: RT, m/z, Blank, Replicate, S/B і Confidence score.",
            deepDive: "Використовуйте його як спільний словник перед переглядом самого конвеєра. Ті самі терміни далі з'являються як inline tooltip-підказки у вкладках Огляд і Довідка.",
            input: ["Базові доменні терміни", "Символи у формулах", "Скорочення для рішень"],
            action: ["Переглянути картки глосарію", "Навести на підсвічені терміни", "Узгодити лексику для всіх наступних кроків"],
            output: ["Єдине тлумачення термінів методології"],
            formula: "Термін -> Визначення -> Узгоджене тлумачення",
            formulaExplanation: "Глосарій не перетворює дані. Він стандартизує мову, якою описуються формули, правила та рішення в наступних кроках."
        },
        {
            id: "excel",
            title: "Вхідні дані та валідація",
            short: "Читання Excel та очистка",
            tone: "slate",
            summary: "Робоча книга зчитується, і система автоматично вибирає аркуш, що містить найбільше обов'язкових колонок (RT, Area, m/z). Усі неповні рядки відкидаються.",
            deepDive: "Цей крок гарантує цілісність даних. Рядки без базових LC-MS параметрів не допускаються до подальшого розрахунку і відхиляються до будь-яких наукових висновків.",
            input: ["Робоча книга (один або кілька аркушів)", "Рядки піків"],
            action: ["Обрати аркуш із найкращим збігом", "Видалити неповні рядки"],
            output: ["Валідні рядки для аналізу"],
            formula: "Sheet = MaxMatch(Headers, Req)  AND  Valid = (RT>0 & Area>0)",
            formulaExplanation: "Ми шукаємо аркуш, де назви колонок найкраще відповідають вимогам. Залишаються лише рядки з базовими числовими даними."
        },
        {
            id: "classify",
            title: "Призначення ролей",
            short: "Sample / Blank / Rep",
            tone: "violet",
            summary: "Кожному рядку призначається тип зразка (Sample або Blank), після чого він потрапляє у відповідний кошик репліки. Далі всі обчислення виконуються окремо для кожної полярності.",
            deepDive: "Якщо оператор розфарбував клітинки в Excel, система довіряє цьому кольору. Якщо кольору немає, використовується розбір імені файлу.",
            input: ["Валідні рядки", "Кольори клітинок Excel", "Імена файлів"],
            action: ["Зчитати мітки оператора", "Застосувати логіку імені файлу"],
            output: ["Групування за (SampleType, Polarity)"],
            formula: "Role = ColorMap[CellColor] || FileNameLogic(Name)",
            formulaExplanation: "Кольорові мітки мають найвищий пріоритет. Якщо їх немає, система аналізує текстову назву на наявність патернів."
        },
        {
            id: "replicates",
            title: "Кластеризація реплікатів",
            short: "Підтвердження піків",
            tone: "blue",
            summary: "Піки одного зразка, але з різних вимірювань, об'єднуються в групи. Ми перевіряємо, що та сама сполука присутня в кількох незалежних вимірах.",
            deepDive: "Жадібна кластеризація: пік із найбільшою площею стає зерном. Із кожного іншого кошика реплікатів вибирається найближчий пік у просторі (RT + m/z). Якщо збіги знайдено щонайменше у 2 кошиках, кластер підтверджується.",
            input: ["Піки в кошиках реплікатів"],
            action: ["Сортувати за спаданням Area", "Усереднювати центроїд при додаванні", "Перевіряти вікна допуску"],
            output: ["Підтверджені кластери"],
            formula: "|RT_cand \u2212 RT_cent| \u2264 Tol_RT  AND  \u0394m/z \u2264 Tol_m/z",
            formulaExplanation: "Щоб об'єднати піки з різних файлів, їхній час утримання й маса мають бути майже однаковими в межах налаштованих допусків."
        },
        {
            id: "blank",
            title: "Blank Subtraction",
            short: "Порівняння з фоном",
            tone: "cyan",
            summary: "Blank-проби проходять незалежну кластеризацію. Кожен підтверджений sample зіставляється з найближчим blank-сигналом для оцінки фонового шуму.",
            deepDive: "Ми шукаємо найближчий кластер у blank за координатами (RT + m/z). Якщо відстані рівні, вибирається більший blank, щоб фільтрація фону була консервативною.",
            input: ["Sample-кластери", "Blank-кластери"],
            action: ["Знайти найближчий blank-кластер тієї ж полярності", "Обчислити S/B ratio"],
            output: ["Пари Sample ↔ Blank з ratio"],
            formula: "Ratio (S/B) = mean(Area_sample) / mean(Area_blank)",
            formulaExplanation: "Середня площа в зразку ділиться на середню площу в blank. Усереднення захищає від випадкових сплесків."
        },
        {
            id: "decision",
            title: "Класифікація",
            short: "Artifact / Real Compound",
            tone: "green",
            summary: "Якщо відповідний blank знайдено, а сигнал зразка не перевищує фон достатньо сильно за порогом S/B, він відхиляється як артефакт.",
            deepDive: "Цей бінарний поділ є ключовим бізнес-результатом. Він дозволяє хіміку зосередитися лише на реальних чистих сполуках і відсіяти забруднення.",
            input: ["Signal-to-Blank (S/B) ratio", "Поріг рішення"],
            action: ["Порівняти S/B ratio з порогом"],
            output: ["Рішення: Artifact або Real Compound"],
            formula: "Status = (Ratio < Threshold) ? \"Artifact\" : \"Real Compound\"",
            formulaExplanation: "Якщо сигнал зразка недостатньо виразно відділяється від blank, ми класифікуємо його як Artifact."
        },
        {
            id: "output",
            title: "Підсумок і аудит",
            short: "Аудиторський слід",
            tone: "rose",
            summary: "Кожен крок, що веде до фінального висновку, разом із підсумковими статистичними метриками зберігається для регуляторного контролю.",
            deepDive: "Детальний журнал рішень зберігається у форматі JSON. Також обчислюються середні RT, m/z, площа, варіабельність (CV%) і бонуси/штрафи показника довіри.",
            input: ["Підтверджені кластери зі статусом"],
            action: ["Обчислити CV% і середні", "Розрахувати confidence score", "Серіалізувати decision trail"],
            output: ["Підсумкова таблиця (Excel для аудиту)"],
            formula: "Score = 100 \u2212 \u03a3(Penalties) + Bonus",
            formulaExplanation: "Рейтинг надійності від 0 до 100. Бали знімаються за зсуви маси/RT і розкид реплікатів, а додаються за ідеальне відокремлення від blank."
        }
    ];

    // ── Reference data (tables, lists) ──────────────────────────────────
    const columns = [
        { col: "RT",        type: "number", desc: "Час утримання хроматографічного піка",                    ex: "2.345" },
        { col: "Base Peak", type: "number", desc: "m/z домінуючого іона в мас-спектрі",                      ex: "195.08" },
        { col: "Polarity",  type: "string", desc: "Полярність іонізації: positive / negative",               ex: "positive" },
        { col: "File",      type: "string", desc: "Ім'я файлу, що використовується для призначення реплік", ex: "1_pos.d" },
        { col: "Area",      type: "number", desc: "Площа піка, пропорційна вмісту аналіту",                  ex: "1250000" },
        { col: "Label",     type: "string", desc: "(Опційно) мітка оператора або назва сполуки",             ex: "Caffeine" },
    ];

    const operatorMarks = [
        { name: "sample_rep1",     color: "#ff00ff", label: "Sample, репліка 1" },
        { name: "sample_rep2",     color: "#ffff00", label: "Sample, репліка 2" },
        { name: "blank_positive",  color: "#00ffff", label: "Blank" },
        { name: "blank_negative",  color: "#00ff00", label: "Blank" },
    ];

    const outputFields = [
        { field: "RT_mean",            desc: "Середній RT підтвердженого кластера по всіх реплікатах." },
        { field: "MZ_mean",            desc: "Середній m/z підтвердженого кластера." },
        { field: "Area_mean",          desc: "Середня площа піка без цілочисельного обрізання." },
        { field: "AreaCVPct",          desc: "CV% між площами реплікатів у кластері." },
        { field: "ReplicateQuality",   desc: "High (CV% \u2264 15) / Moderate (\u2264 30) / Low (> 30) — рівень відтворюваності." },
        { field: "SignalToBlankRatio", desc: "S/B ratio для найближчого підтвердженого blank-піка." },
        { field: "ConfidenceScore",    desc: "Підсумковий показник довіри 0–100 після blank subtraction." },
        { field: "Status",             desc: "Результат класифікації: Real Compound або Artifact." },
        { field: "Why",                desc: "JSON-об'єкт із повним аудиторським слідом рішення (RT/mz, CV, S/B, пороги)." },
    ];

    const params = [
        { name: "replicate_rt_tol",   def: "0.1",  unit: "хв",      used: "Кластеризація реплікатів" },
        { name: "replicate_mz_tol",   def: "0.3",  unit: "Da / ppm", used: "Кластеризація реплікатів" },
        { name: "blank_rt_tol",       def: "0.1",  unit: "хв",      used: "Blank subtraction" },
        { name: "blank_mz_tol",       def: "0.3",  unit: "Da / ppm", used: "Blank subtraction" },
        { name: "signal_to_blank_min",def: "3.0",  unit: "ratio",   used: "Рішення Artifact / Real Compound" },
        { name: "cv_high_max",        def: "15",   unit: "%",       used: "ReplicateQuality = High" },
        { name: "cv_moderate_max",    def: "30",   unit: "%",       used: "ReplicateQuality = Moderate" },
    ];

    const glossary = [
        { term: "RT",               def: "Retention Time — час утримання аналіту в колонці, вимірюється в хвилинах." },
        { term: "m/z",              def: "Mass-to-charge ratio — центральна координата мас-спектрометричного сигналу." },
        { term: "CV%",              def: "Coefficient of Variation — відносна варіабельність площ піків між реплікатами. CV% = (sample std / mean) × 100. Для n=2: std = |v₁−v₂|/√2. Нижчий CV означає кращу відтворюваність." },
        { term: "S/B",              def: "Signal-to-Blank ratio — площа sample-піка, поділена на площу найближчого підтвердженого blank-піка." },
        { term: "Blank",            def: "Холоста проба (лише розчинник, без аналіту), що використовується для виявлення фонового сигналу та забруднення." },
        { term: "Replicate",        def: "Незалежне повторне вимірювання того самого зразка. Для підтвердження піка потрібно щонайменше 2 кошики реплікатів." },
        { term: "Confidence score", def: "Показник 0–100, що стартує зі 100 і штрафується за відхилення RT/m/z, CV% та S/B ratio." },
        { term: "ppm",              def: "Parts per million — відносний допуск по m/z для високороздільних приладів." },
        { term: "Da",               def: "Dalton — абсолютний допуск по m/z у дальтонах." },
        { term: "Replicate bucket", def: "Усі піки з одного файлу або з однієї операторської мітки. Кожен файл/мітка формує окремий кошик." },
    ];

    let resolvedGlossary = $derived({
        ...Object.fromEntries(glossary.map(({ term, def }) => [term, def])),
        ...defs
    });

    let glossaryDefinitions = $derived({
        ...resolvedGlossary,
        blank: resolvedGlossary.Blank,
        "blank-проба": resolvedGlossary.Blank,
        "blank-проби": resolvedGlossary.Blank,
        "blank-пік": resolvedGlossary.Blank,
        "blank-піки": resolvedGlossary.Blank,
        replicate: resolvedGlossary.Replicate,
        "реплікат": resolvedGlossary.Replicate,
        "реплікати": resolvedGlossary.Replicate,
        "кошик реплікатів": resolvedGlossary["Replicate bucket"],
        "кошики реплікатів": resolvedGlossary["Replicate bucket"],
        "confidence score": resolvedGlossary["Confidence score"],
        "показник довіри": resolvedGlossary["Confidence score"],
        "S/B ratio": resolvedGlossary["S/B"],
        ratio: resolvedGlossary["S/B"]
    });

    const formulaMarkupById: Record<string, string> = {
        glossary: '<math xmlns="http://www.w3.org/1998/Math/MathML" display="block"><mrow><mtext>Термін</mtext><mspace width="0.4em"/><mo>→</mo><mspace width="0.4em"/><mtext>Визначення</mtext><mspace width="0.4em"/><mo>→</mo><mspace width="0.4em"/><mtext>Узгоджене тлумачення</mtext></mrow></math>',
        excel: '<math xmlns="http://www.w3.org/1998/Math/MathML" display="block"><mtable columnalign="left"><mtr><mtd><mi>Sheet</mi><mo>=</mo><mi>MaxMatch</mi><mo>(</mo><mi>Headers</mi><mo>,</mo><mi>Req</mi><mo>)</mo></mtd></mtr><mtr><mtd><mi>Valid</mi><mo>=</mo><mo>(</mo><mi>RT</mi><mo>&gt;</mo><mn>0</mn><mspace width="0.3em"/><mo>∧</mo><mspace width="0.3em"/><mi>Area</mi><mo>&gt;</mo><mn>0</mn><mo>)</mo></mtd></mtr></mtable></math>',
        classify: '<math xmlns="http://www.w3.org/1998/Math/MathML" display="block"><mrow><mi>Role</mi><mo>=</mo><mi>ColorMap</mi><mo>(</mo><mi>CellColor</mi><mo>)</mo><mspace width="0.6em"/><mo>∨</mo><mspace width="0.6em"/><mi>FileNameLogic</mi><mo>(</mo><mi>Name</mi><mo>)</mo></mrow></math>',
        replicates: '<math xmlns="http://www.w3.org/1998/Math/MathML" display="block"><mtable columnalign="left"><mtr><mtd><mo>|</mo><msub><mi>RT</mi><mtext>cand</mtext></msub><mo>−</mo><msub><mi>RT</mi><mtext>cent</mtext></msub><mo>|</mo><mo>≤</mo><msub><mi>Tol</mi><mtext>RT</mtext></msub></mtd></mtr><mtr><mtd><mi>Δm/z</mi><mo>≤</mo><msub><mi>Tol</mi><mtext>m/z</mtext></msub></mtd></mtr></mtable></math>',
        blank: '<math xmlns="http://www.w3.org/1998/Math/MathML" display="block"><mtable columnalign="left"><mtr><mtd><mi>Ratio</mi><mo>(</mo><mi>S</mi><mo>/</mo><mi>B</mi><mo>)</mo><mo>=</mo></mtd></mtr><mtr><mtd><mfrac><mrow><mi>mean</mi><mo>(</mo><msub><mi>Area</mi><mtext>sample</mtext></msub><mo>)</mo></mrow><mrow><mi>mean</mi><mo>(</mo><msub><mi>Area</mi><mtext>blank</mtext></msub><mo>)</mo></mrow></mfrac></mtd></mtr></mtable></math>',
        decision: '<math xmlns="http://www.w3.org/1998/Math/MathML" display="block"><mrow><mi>Status</mi><mo>=</mo><mrow><mo>{</mo><mtable columnalign="left left" rowspacing="0.2em"><mtr><mtd><mtext>Artifact,</mtext></mtd><mtd><mtext>якщо Ratio &lt; Threshold</mtext></mtd></mtr><mtr><mtd><mtext>Real Compound,</mtext></mtd><mtd><mtext>інакше</mtext></mtd></mtr></mtable></mrow></mrow></math>',
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
            <h2 class="text-[10px] font-bold uppercase tracking-[0.15em] text-slate-400 dark:text-slate-500">Конвеєр</h2>
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
                або scroll
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
                Огляд
            </button>
            <button
                onclick={() => (tab = 'reference')}
                class="rounded-t-lg px-3 py-1.5 text-xs font-semibold transition-colors
                    {tab === 'reference'
                        ? 'border-b-2 border-blue-500 text-blue-600 dark:text-blue-400'
                        : 'text-slate-400 hover:text-slate-600 dark:text-slate-500 dark:hover:text-slate-300'}"
            >
                Довідка
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
                            <p class="mb-1.5 text-[9px] font-bold uppercase tracking-widest text-slate-400 dark:text-slate-500">Вхід</p>
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
                            <p class="mb-1.5 text-[9px] font-bold uppercase tracking-widest text-blue-500 dark:text-blue-400">Процес</p>
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
                            <p class="mb-1.5 text-[9px] font-bold uppercase tracking-widest text-emerald-500 dark:text-emerald-400">Вихід</p>
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
                            <h4 class="mb-1.5 text-[10px] font-bold uppercase tracking-wider text-indigo-600 dark:text-indigo-400">Математика й логіка</h4>
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
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Спільна термінологія</h4>
                                <p><GlossaryRichText text="Глосарій винесений у крок 0, бо всі наступні етапи використовують ці самі терміни в резюме, правилах, формулах і фінальному аудиті." definitions={glossaryDefinitions} /></p>
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
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Вхідні дані</h4>
                                <p><GlossaryRichText text="Система приймає Excel-файл (.xlsx / .xls) з LC-MS піковими даними, автоматично вибирає найкращий аркуш і очікує щонайменше два реплікати плюс одну blank-пробу." definitions={glossaryDefinitions} /></p>
                            </div>

                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Обов'язкові колонки Excel</h4>
                                <div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
                                    <table class="w-full text-[11px]">
                                        <thead class="bg-slate-50 dark:bg-slate-800">
                                            <tr>
                                                <th class="px-3 py-2 text-left font-semibold text-slate-700 dark:text-slate-300">Колонка</th>
                                                <th class="px-3 py-2 text-left font-semibold text-slate-700 dark:text-slate-300">Тип</th>
                                                <th class="px-3 py-2 text-left font-semibold text-slate-700 dark:text-slate-300">Опис</th>
                                                <th class="px-3 py-2 text-left font-semibold text-slate-700 dark:text-slate-300">Приклад</th>
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
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Мітки оператора</h4>
                                <p class="mb-3"><GlossaryRichText text='Кольорові позначки клітинок мають пріоритет над іменем файлу. Без міток тип виводиться з назви файлу: файли з "blank" стають blank; файли на кшталт 1_*.d, 2_*.d стають sample_1, sample_2 тощо.' definitions={glossaryDefinitions} /></p>
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
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Групування</h4>
                                <p><GlossaryRichText text="Після призначення ролей рядки групуються за парою (SampleType, Polarity). Кожна група далі обробляється незалежно через кластеризацію та blank subtraction." definitions={glossaryDefinitions} /></p>
                            </div>

                        <!-- Step 3: Replicate Clustering -->
                        {:else if activeStep.id === 'replicates'}
                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Жадібна кластеризація реплікатів</h4>
                                <p><GlossaryRichText text="Піки сортуються за спаданням площі. Кожен пік по черзі стає зерном кластера. З кожного іншого кошика жадібно вибирається найближчий невикористаний пік до поточного центроїда (RT і m/z усереднюються при додаванні членів). Кластер підтверджується, коли охоплює 2 або більше різні кошики." definitions={glossaryDefinitions} /></p>
                            </div>

                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Правило включення</h4>
                                <div class="rounded-lg border border-blue-100 bg-blue-50/50 p-3 font-mono text-[11px] dark:border-blue-900/40 dark:bg-blue-900/10">
                                    <p>|RT_candidate &minus; RT_centroid| &le; replicate_rt_tol</p>
                                    <p>|mz_candidate &minus; mz_centroid| &le; replicate_mz_tol (Da або ppm)</p>
                                </div>
                                <p class="mt-2"><GlossaryRichText text="При рівності обирається найменша відстань (RT fraction + mz fraction), а потім більша площа." definitions={glossaryDefinitions} /></p>
                            </div>

                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Параметри</h4>
                                <div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
                                    <table class="w-full text-[11px]">
                                        <thead class="bg-slate-50 dark:bg-slate-800">
                                            <tr>
                                                <th class="px-3 py-2 text-left font-semibold">Параметр</th>
                                                <th class="px-3 py-2 text-left font-semibold">Типово</th>
                                                <th class="px-3 py-2 text-left font-semibold">Одиниця</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-slate-100 dark:divide-slate-700">
                                            {#each params.filter(p => p.used === 'Кластеризація реплікатів') as p}
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
                                <p><GlossaryRichText text="Blank проходить ту саму кластеризацію незалежно. Кожен підтверджений sample-пік зіставляється з підтвердженими blank-піками в межах blank_rt_tol / blank_mz_tol. Вибирається найближчий збіг за RT+m/z, а при рівності — більший blank." definitions={glossaryDefinitions} /></p>
                            </div>

                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Правила класифікації</h4>
                                <div class="space-y-1.5">
                                    <div class="flex items-start gap-2 rounded-lg border border-rose-100 bg-rose-50/50 p-2.5 dark:border-rose-900/30 dark:bg-rose-900/10">
                                        <span class="mt-0.5 font-mono font-bold text-rose-600 dark:text-rose-400">Artifact</span>
                                        <span><GlossaryRichText text={"знайдено blank-збіг і S/B < signal_to_blank_min (або площа blank = 0)"} definitions={glossaryDefinitions} /></span>
                                    </div>
                                    <div class="flex items-start gap-2 rounded-lg border border-emerald-100 bg-emerald-50/50 p-2.5 dark:border-emerald-900/30 dark:bg-emerald-900/10">
                                        <span class="mt-0.5 font-mono font-bold text-emerald-600 dark:text-emerald-400">Real Compound</span>
                                        <span><GlossaryRichText text={"blank-збігу немає або S/B >= поріг"} definitions={glossaryDefinitions} /></span>
                                    </div>
                                </div>
                            </div>

                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Параметри</h4>
                                <div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
                                    <table class="w-full text-[11px]">
                                        <thead class="bg-slate-50 dark:bg-slate-800">
                                            <tr>
                                                <th class="px-3 py-2 text-left font-semibold">Параметр</th>
                                                <th class="px-3 py-2 text-left font-semibold">Типово</th>
                                                <th class="px-3 py-2 text-left font-semibold">Одиниця</th>
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
                                <p><GlossaryRichText text="Стартує зі 100. Штрафи застосовуються послідовно, а результат обрізається до [0, 100]." definitions={glossaryDefinitions} /></p>
                            </div>

                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Етап 1 &mdash; Реплікати</h4>
                                <div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
                                    <table class="w-full text-[11px]">
                                        <thead class="bg-slate-50 dark:bg-slate-800">
                                            <tr>
                                                <th class="px-3 py-2 text-left font-semibold">Фактор</th>
                                                <th class="px-3 py-2 text-left font-semibold">Штраф</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-slate-100 dark:divide-slate-700">
                                            <tr><td class="px-3 py-1.5">Близькість RT</td><td class="px-3 py-1.5 font-mono">&minus;(mean_RT_delta / rt_tol) &times; 20, max &minus;20</td></tr>
                                            <tr><td class="px-3 py-1.5">Близькість m/z</td><td class="px-3 py-1.5 font-mono">&minus;(mean_mz_delta / mz_tol) &times; 25, max &minus;25</td></tr>
                                            <tr><td class="px-3 py-1.5">CV% High</td><td class="px-3 py-1.5 font-mono">0</td></tr>
                                            <tr><td class="px-3 py-1.5">CV% Moderate</td><td class="px-3 py-1.5 font-mono">&minus;12</td></tr>
                                            <tr><td class="px-3 py-1.5">CV% Low</td><td class="px-3 py-1.5 font-mono">&minus;(12 + (CV &minus; cv_moderate_max) &times; 0.7), max &minus;35</td></tr>
                                            <tr><td class="px-3 py-1.5">CV% відсутній</td><td class="px-3 py-1.5 font-mono">&minus;10</td></tr>
                                            <tr><td class="px-3 py-1.5">Не зведено по кольору</td><td class="px-3 py-1.5 font-mono">&minus;5</td></tr>
                                        </tbody>
                                    </table>
                                </div>
                            </div>

                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Етап 2 &mdash; Blank</h4>
                                <div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
                                    <table class="w-full text-[11px]">
                                        <thead class="bg-slate-50 dark:bg-slate-800">
                                            <tr>
                                                <th class="px-3 py-2 text-left font-semibold">Умова</th>
                                                <th class="px-3 py-2 text-left font-semibold">Корекція</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-slate-100 dark:divide-slate-700">
                                            <tr><td class="px-3 py-1.5">Blank-збігу немає</td><td class="px-3 py-1.5 font-mono text-emerald-600 dark:text-emerald-400">+3</td></tr>
                                            <tr><td class="px-3 py-1.5">S/B &ge; поріг</td><td class="px-3 py-1.5 font-mono text-emerald-600 dark:text-emerald-400">+min(5, (S/B &minus; threshold) &times; 0.5)</td></tr>
                                            <tr><td class="px-3 py-1.5">S/B &lt; поріг (Artifact)</td><td class="px-3 py-1.5 font-mono text-rose-600 dark:text-rose-400">&minus;(15 + deficit &times; 30), max &minus;45</td></tr>
                                        </tbody>
                                    </table>
                                </div>
                            </div>

                        <!-- Step 6: Output & Audit -->
                        {:else if activeStep.id === 'output'}
                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Вихідні поля</h4>
                                <div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
                                    <table class="w-full text-[11px]">
                                        <thead class="bg-slate-50 dark:bg-slate-800">
                                            <tr>
                                                <th class="px-3 py-2 text-left font-semibold">Поле</th>
                                                <th class="px-3 py-2 text-left font-semibold">Опис</th>
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
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Усі параметри</h4>
                                <div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
                                    <table class="w-full text-[11px]">
                                        <thead class="bg-slate-50 dark:bg-slate-800">
                                            <tr>
                                                <th class="px-3 py-2 text-left font-semibold">Параметр</th>
                                                <th class="px-3 py-2 text-left font-semibold">Типово</th>
                                                <th class="px-3 py-2 text-left font-semibold">Одиниця</th>
                                                <th class="px-3 py-2 text-left font-semibold">Використовується в</th>
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
                                <p class="mt-2 text-[10px] text-slate-400"><GlossaryRichText text="Параметри RT/m/z/S/B можна змінювати на головній формі. cv_high_max і cv_moderate_max наразі фіксовані." definitions={glossaryDefinitions} /></p>
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
                        aria-label="Перейти до кроку {i}"
                    ></button>
                {/each}
            </div>

            <button
                onclick={() => current < steps.length - 1 && current++}
                class="flex items-center gap-1.5 rounded-lg bg-slate-900 px-4 py-1.5 text-xs font-semibold text-white shadow-sm transition-transform hover:scale-105 disabled:pointer-events-none disabled:opacity-30 dark:bg-white dark:text-slate-900"
                disabled={current === steps.length - 1}
            >
                Далі
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none"><path d="M9 18L15 12L9 6" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>
            </button>
        </div>
    </div>
</div>
