<!--
  @file MethodologyPageRu.svelte
  @description
  Русскоязычная страница документации методологии для приложения LC-MS Screening.
-->
<script lang="ts">
    import MethodologyVisualizerRu from "./MethodologyVisualizerRu.svelte";
    import GlossaryTooltip from "./GlossaryTooltip.svelte";

    // ---------------------------------------------------------------------------
    // Массивы данных — таблицы содержимого
    // ---------------------------------------------------------------------------
    const columns = [
        ["RT", "number", "Время удерживания хроматографического пика (обязательно)", "2.345"],
        ["Base Peak", "number", "m/z доминирующего иона — ОПЦИОНАЛЬНО. Отсутствует ⇒ режим RT-only (GC-FID, LC-UV); штраф за отсутствие m/z не применяется.", "195.08"],
        ["Polarity", "string", "Полярность ионизации — ОПЦИОНАЛЬНО. Отсутствует/пуста для всех строк ⇒ одна группа \"n/a\", без разбиения по полярности.", "positive"],
        ["File", "string", "Имя файла для идентификации образца и репликата (обязательно)", "1_pos.d"],
        ["Area", "number", "Площадь пика (обязательно)", "1250000"],
        ["Label", "string", "Опциональная метка оператора", "Caffeine"],
    ];

    // Принимаемые форматы входных файлов. Текст с разделителями (CSV/TSV/TXT)
    // читается как таблица; XLSX/XLS дополнительно несут цвета заливки ячеек
    // как метки оператора. Новый прибор с другими заголовками подключается через
    // профиль сопоставления колонок.
    const inputFormats = [
        ["CSV / TSV / TXT", "Текст с разделителями. Без цветов ячеек — метки должны быть в колонках operator_mark / operator_color."],
        ["XLSX / XLS", "Книга Excel. Цвета заливки ячеек читаются как метки оператора."],
        ["Профиль колонок", "Сопоставляет заголовки нового прибора (RetentionTime → RT, PeakArea → Area) с каноническими полями; может объявить Base Peak / Polarity отсутствующими."],
    ];

    const operatorMarks = [
        ["sample_rep1", "#ff00ff", "Sample, Replicate 1"],
        ["sample_rep2", "#ffff00", "Sample, Replicate 2"],
        ["blank_positive", "#00ffff", "Blank"],
        ["blank_negative", "#00ff00", "Blank"],
    ];

    const outputFields = [
        ["RT_mean", "Средний RT подтверждённого кластера."],
        ["MZ_mean", "Средний m/z подтверждённого кластера (null для данных RT-only)."],
        ["Area_mean", "Средняя площадь пика без округления до int."],
        ["AreaCVPct", "RSD (относительное стандартное отклонение) площадей пиков репликатов — та же величина, что ранее CV%. В UI: \"RSD (CV%)\"."],
        ["ReplicateCount", "Число репликатов в кластере. 1 репликат = 1 хроматограмма."],
        ["ReplicateQuality", "High / Moderate / Low — категория качества (по RSD)."],
        ["SignalToBlankRatio", "S/B ratio для сопоставленного blank-пика; null, когда вычитать нечего."],
        ["Why.BlankState", "blank_subtracted (blank сопоставлен и вычтен) · blank_clean_here (blank есть, но чист на этом RT) · no_blank_loaded (blank вообще не загружен). Последние два остаются Real Compound."],
        ["Why.BinWidthSuggestion", "Для суррогатов: рекомендательные метрики ширины корзины по разбросу RT — MaxAbsRtShift, KStdevRt (k·σ), RangeRt — каждая с числом репликатов."],
        ["ConfidenceScore", "Итоговый показатель доверия 0–100."],
        ["Status", "Real Compound или Artifact."],
        ["Why", "JSON decision trail с деталями порогов."],
    ];

    const params = [
        ["replicate_rt_tol", "0.1", "мин", "Coarse screening (рекомендация ширины корзины суррогата помогает задать значение)"],
        ["replicate_mz_tol", "0.3", "Da / ppm", "Coarse screening"],
        ["blank_rt_tol", "0.1", "мин", "Blank subtraction"],
        ["blank_mz_tol", "0.3", "Da / ppm", "Blank subtraction"],
        ["signal_to_blank_min", "3.0", "ratio", "Artifact / Real Compound decision"],
        ["mz_available", "true", "флаг", "Авто-false без колонки Base Peak ⇒ RT-only; снимает штраф за m/z"],
        ["polarity_available", "true", "флаг", "Авто-false без Polarity ⇒ одна группа \"n/a\""],
    ];

    const glossary = [
        ["RT", "Время удерживания аналита в LC-колонке."],
        ["m/z", "Отношение массы иона к заряду."],
        ["Replicate", "Независимое повторное измерение образца. 1 репликат = 1 хроматограмма."],
        ["Blank", "Холостой контроль для выявления фонового сигнала."],
        ["RSD (CV%)", "Относительное стандартное отклонение площадей репликатов (σ/среднее·100). RSD и CV% — одно и то же; UI использует RSD."],
        ["S/B ratio", "Signal-to-Blank ratio. Если blank-пика нет, результат — 'blank не обнаружен', а не пусто."],
        ["Ширина корзины", "Допуск replicate RT. Разброс RT суррогата подсказывает значение (рекомендательно) для ввода в Конфигурацию анализатора."],
        ["Режим RT-only", "Работа без m/z (и опционально без полярности) — сопоставление репликатов и blank только по времени удерживания."],
        ["Confidence score", "Обобщённая метрика доверия к screened-пику."],
    ];

    const glossaryMap = Object.fromEntries(glossary);

    const refs = [
        ["Liquid chromatography–mass spectrometry (LC–MS)", "https://en.wikipedia.org/wiki/Liquid_chromatography%E2%80%93mass_spectrometry"],
        ["Mass spectrometry", "https://en.wikipedia.org/wiki/Mass_spectrometry"],
        ["Coefficient of variation", "https://en.wikipedia.org/wiki/Coefficient_of_variation"],
        ["ISO/IEC 17025", "https://en.wikipedia.org/wiki/ISO/IEC_17025"],
    ];
</script>

<svelte:head>
    <title>Методология — LC-MS Screening</title>
</svelte:head>

<main class="min-h-screen bg-slate-50 dark:bg-slate-900">
    <div class="mx-auto max-w-5xl px-6 py-12">

        <div class="mb-8">
            <a
                href={import.meta.env.VITE_STANDALONE ? "../" : "/"}
                data-sveltekit-reload={import.meta.env.VITE_STANDALONE ? "" : undefined}
                class="inline-flex items-center gap-2 text-sm text-slate-500 hover:text-slate-900 dark:text-slate-400 dark:hover:text-slate-100"
            >
                <span>←</span>
                <span>Назад</span>
            </a>
        </div>

        <header class="mb-10">
            <h1 class="text-4xl font-bold text-slate-900 dark:text-slate-50">Методология скрининга</h1>
            <p class="mt-3 max-w-3xl text-lg text-slate-500 dark:text-slate-400">
                Компактное, но полное описание того, как LC-MS Screening читает Excel, подтверждает пики репликатов,
                выполняет вычитание blank и формирует результат, пригодный для аудита.
            </p>
        </header>

        <nav class="mb-10 rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
            <p class="mb-3 text-xs font-semibold uppercase tracking-[0.2em] text-slate-400 dark:text-slate-500">Содержание</p>
            <div class="grid gap-2 text-sm text-blue-700 dark:text-blue-400 sm:grid-cols-2">
                <a href="#input" class="hover:underline">1. Входные данные</a>
                <a href="#flexibility" class="hover:underline">⓿ Гибкость анализатора и терминология (v0.9.0)</a>
                <a href="#columns" class="hover:underline">2. Колонки Excel</a>
                <a href="#marks" class="hover:underline">3. Метки оператора</a>
                <a href="#algorithm" class="hover:underline">4. Алгоритм (Интерактивный гид)</a>
                <a href="#output" class="hover:underline">5. Выходные поля</a>
                <a href="#params" class="hover:underline">6. Параметры</a>
                <a href="#glossary" class="hover:underline">7. Глоссарий</a>
                <a href="#references" class="hover:underline">8. Ссылки</a>
            </div>
        </nav>

        <!-- Интерактивный визуальный обзор -->
        <div id="algorithm">
            <MethodologyVisualizerRu defs={glossaryMap} />
        </div>

        <!-- Раздел 1: Входные данные -->
        <section id="input" class="mb-10 rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
            <h2 class="mb-3 text-2xl font-semibold text-slate-900 dark:text-slate-50">1. Входные данные</h2>
            <div class="space-y-3 text-sm leading-7 text-slate-600 dark:text-slate-400">
                <p>Приложение ожидает Excel-книгу с LC-MS пиковыми данными. Если листов несколько, автоматически выбирается лист с наилучшим совпадением по обязательным колонкам.</p>
                <p>Типичный сценарий: два измерения <GlossaryTooltip term="Replicate" definition={glossaryMap["Replicate"]} /> плюс один <GlossaryTooltip term="Blank" definition={glossaryMap["Blank"]} />. Blank используется как контроль на фон, матрицу и лабораторные артефакты.</p>
            </div>
        </section>

        <!-- Раздел 2: Обязательные колонки Excel -->
        <!-- Раздел: Гибкость анализатора и терминология (v0.9.0) -->
        <section id="flexibility" class="mb-10 rounded-2xl border border-blue-200 bg-blue-50/40 p-6 shadow-sm dark:border-blue-900/60 dark:bg-blue-950/20">
            <h2 class="mb-2 text-2xl font-semibold text-slate-900 dark:text-slate-50">Гибкость анализатора и терминология <span class="align-middle text-xs font-bold text-blue-700 dark:text-blue-400">v0.9.0</span></h2>
            <p class="mb-5 text-sm leading-7 text-slate-600 dark:text-slate-400">Этот релиз делает конвейер устойчивым к реальным входным данным — другие приборы, отсутствующие измерения и чистые blank — и уточняет экранную терминологию.</p>

            <div class="space-y-5 text-sm leading-7 text-slate-600 dark:text-slate-400">
                <div>
                    <h3 class="font-semibold text-slate-900 dark:text-slate-100">1 · Форматы ввода и профили приборов</h3>
                    <p>Принимаемые типы файлов: <strong>CSV, TSV, TXT, XLSX, XLS</strong>. Текст с разделителями читается как таблица; Excel дополнительно несёт цвета заливки ячеек как метки оператора. Чтобы подключить прибор с другими заголовками экспорта, выберите <strong>профиль прибора</strong> в панели импорта — он сопоставляет заголовки прибора (например, <span class="font-mono">RetentionTime → RT</span>, <span class="font-mono">PeakArea → Area</span>) с каноническими полями и может объявить <span class="font-mono">Base Peak</span> / <span class="font-mono">Polarity</span> отсутствующими. Набор образцов собирается в workspace из одного или нескольких файлов.</p>
                </div>
                <div>
                    <h3 class="font-semibold text-slate-900 dark:text-slate-100">2 · Опциональные Polarity / Base Peak (RT-only)</h3>
                    <p>Алгоритм теперь работает без <span class="font-mono">Polarity</span> (все строки образуют одну группу <span class="font-mono">n/a</span> вместо разбиения по полярности) и без <span class="font-mono">Polarity</span> и <span class="font-mono">Base Peak</span> одновременно (RT-only — репликаты и blank сопоставляются только по времени удерживания). Когда m/z отсутствует во всём наборе, штраф доверия за отсутствие m/z не применяется, поскольку это свойство формата данных, а не слабость сопоставления.</p>
                </div>
                <div>
                    <h3 class="font-semibold text-slate-900 dark:text-slate-100">3 · Signal-to-Blank без корзины blank</h3>
                    <p>Когда у пика образца нет сопоставленной корзины blank, поле больше не остаётся пустым. В <span class="font-mono">Why.BlankState</span> сообщаются три состояния: <span class="font-mono">blank_subtracted</span> (blank сопоставлен и вычтен), <span class="font-mono">blank_clean_here</span> (blank есть, но чист на этом RT) и <span class="font-mono">no_blank_loaded</span> (blank вообще не загружен — например, был настолько чист, что оператор ничего не обнаружил). Последние два остаются <strong>Real Compound</strong> — чистый blank читается как сильный результат, а не как отсутствующий.</p>
                </div>
                <div>
                    <h3 class="font-semibold text-slate-900 dark:text-slate-100">4 · Ширина корзины кластера по суррогату</h3>
                    <p>Наблюдаемое поведение RT суррогата подсказывает допуск replicate RT («ширину корзины»). Открыв аудит суррогата, вы увидите <span class="font-mono">Why.BinWidthSuggestion</span> с тремя рекомендательными метриками — <span class="font-mono">MaxAbsRtShift</span>, <span class="font-mono">KStdevRt</span> (k·σ, k=3) и <span class="font-mono">RangeRt</span> — каждая с числом репликатов. Ничего не применяется автоматически; оператор вводит значение в Конфигурацию анализатора вручную.</p>
                </div>
                <div>
                    <h3 class="font-semibold text-slate-900 dark:text-slate-100">5 · Терминология: RSD = CV%, Репликат = хроматограмма</h3>
                    <p><strong>CV% — это RSD</strong> (относительное стандартное отклонение, σ/среднее·100) — та же величина; UI теперь обозначает её <span class="font-mono">RSD (CV%)</span>, значение не меняется. <strong>1 репликат = 1 хроматограмма</strong>, отображается как «Репликаты (хроматограммы)».</p>
                </div>
                <div>
                    <h3 class="font-semibold text-slate-900 dark:text-slate-100">6 · Подсветка происхождения репликатов</h3>
                    <p>Число репликатов визуально выделено; при наведении показывается происхождение каждого репликата — исходные файлы и, если кластер собран из параллельных образцов, какие образцы он охватывает (янтарное кольцо отмечает смешанное происхождение).</p>
                </div>
            </div>

            <div class="mt-6 overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-sm dark:border-slate-700 dark:bg-slate-800">
                <table class="w-full text-sm">
                    <thead class="bg-slate-50 dark:bg-slate-700">
                        <tr>
                            <th class="px-4 py-3 text-left font-semibold text-slate-700 dark:text-slate-300">Формат</th>
                            <th class="px-4 py-3 text-left font-semibold text-slate-700 dark:text-slate-300">Примечания</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-slate-100 dark:divide-slate-700">
                        {#each inputFormats as row}
                            <tr class="hover:bg-slate-50/60 dark:hover:bg-slate-700/40">
                                <td class="px-4 py-3 font-mono font-semibold text-blue-700 dark:text-blue-400">{row[0]}</td>
                                <td class="px-4 py-3 text-slate-600 dark:text-slate-400">{row[1]}</td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
        </section>

        <section id="columns" class="mb-10">
            <h2 class="mb-4 text-2xl font-semibold text-slate-900 dark:text-slate-50">2. Обязательные колонки Excel</h2>
            <div class="overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-sm dark:border-slate-700 dark:bg-slate-800">
                <table class="w-full text-sm">
                    <thead class="bg-slate-50 dark:bg-slate-700">
                        <tr>
                            <th class="px-4 py-3 text-left font-semibold text-slate-700 dark:text-slate-300">Колонка</th>
                            <th class="px-4 py-3 text-left font-semibold text-slate-700 dark:text-slate-300">Тип</th>
                            <th class="px-4 py-3 text-left font-semibold text-slate-700 dark:text-slate-300">Описание</th>
                            <th class="px-4 py-3 text-left font-semibold text-slate-700 dark:text-slate-300">Пример</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-slate-100 dark:divide-slate-700">
                        {#each columns as row}
                            <tr class="hover:bg-slate-50/60 dark:hover:bg-slate-700/40">
                                <td class="px-4 py-3 font-mono font-semibold text-blue-700 dark:text-blue-400">{row[0]}</td>
                                <td class="px-4 py-3 text-slate-500 dark:text-slate-400">{row[1]}</td>
                                <td class="px-4 py-3 text-slate-700 dark:text-slate-300">{row[2]}</td>
                                <td class="px-4 py-3 font-mono text-slate-500 dark:text-slate-400">{row[3]}</td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
        </section>

        <!-- Раздел 3: Метки оператора -->
        <section id="marks" class="mb-10 rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
            <h2 class="mb-4 text-2xl font-semibold text-slate-900 dark:text-slate-50">3. Метки оператора</h2>
            <p class="text-sm leading-7 text-slate-600 dark:text-slate-400">
                Ручные цвета ячеек в Excel позволяют оператору явно задать роль каждой строки. При их наличии эти метки имеют приоритет над эвристикой по имени файла.
            </p>
            <div class="mt-4 grid gap-3 sm:grid-cols-2">
                {#each operatorMarks as mark}
                    <div class="flex items-start gap-3 rounded-xl border border-slate-200 bg-slate-50 p-4 dark:border-slate-700 dark:bg-slate-900">
                        <span class="mt-1 h-4 w-4 rounded-full border border-black/10 dark:border-white/10" style={`background:${mark[1]}`}></span>
                        <div>
                            <p class="font-mono text-sm font-semibold text-slate-900 dark:text-slate-100">{mark[0]}</p>
                            <p class="text-xs text-slate-500 dark:text-slate-400">{mark[2]}</p>
                        </div>
                    </div>
                {/each}
            </div>
        </section>

        <!-- Раздел 5: Выходные поля -->
        <section id="output" class="mb-10">
            <h2 class="mb-4 text-2xl font-semibold text-slate-900 dark:text-slate-50">5. Выходные поля</h2>
            <div class="space-y-3">
                {#each outputFields as field}
                    <div class="rounded-2xl border border-slate-200 bg-white px-5 py-4 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                        <p class="font-mono text-sm font-semibold text-blue-700 dark:text-blue-400">{field[0]}</p>
                        <p class="mt-1 text-sm text-slate-600 dark:text-slate-400">{field[1]}</p>
                    </div>
                {/each}
            </div>
        </section>

        <!-- Раздел 6: Параметры толерантности -->
        <section id="params" class="mb-10">
            <h2 class="mb-4 text-2xl font-semibold text-slate-900 dark:text-slate-50">6. Параметры толерантности</h2>
            <div class="overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-sm dark:border-slate-700 dark:bg-slate-800">
                <table class="w-full text-sm">
                    <thead class="bg-slate-50 dark:bg-slate-700">
                        <tr>
                            <th class="px-4 py-3 text-left font-semibold text-slate-700 dark:text-slate-300">Параметр</th>
                            <th class="px-4 py-3 text-left font-semibold text-slate-700 dark:text-slate-300">По умолч.</th>
                            <th class="px-4 py-3 text-left font-semibold text-slate-700 dark:text-slate-300">Единица</th>
                            <th class="px-4 py-3 text-left font-semibold text-slate-700 dark:text-slate-300">Используется в</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-slate-100 dark:divide-slate-700">
                        {#each params as row}
                            <tr class="hover:bg-slate-50/60 dark:hover:bg-slate-700/40">
                                <td class="px-4 py-3 font-mono font-semibold text-blue-700 dark:text-blue-400">{row[0]}</td>
                                <td class="px-4 py-3 text-slate-700 dark:text-slate-300">{row[1]}</td>
                                <td class="px-4 py-3 text-slate-500 dark:text-slate-400">{row[2]}</td>
                                <td class="px-4 py-3 text-slate-600 dark:text-slate-400">{row[3]}</td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
        </section>

        <!-- Раздел 7: Глоссарий -->
        <section id="glossary" class="mb-10 rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
            <h2 class="mb-4 text-2xl font-semibold text-slate-900 dark:text-slate-50">7. Глоссарий</h2>
            <div class="grid gap-3 sm:grid-cols-2">
                {#each glossary as item}
                    <div class="rounded-xl border border-slate-200 bg-slate-50 p-4 dark:border-slate-700 dark:bg-slate-900">
                        <p class="font-mono text-sm font-semibold text-blue-700 dark:text-blue-400">{item[0]}</p>
                        <p class="mt-1 text-sm text-slate-600 dark:text-slate-400">{item[1]}</p>
                    </div>
                {/each}
            </div>
        </section>

        <!-- Раздел 8: Ссылки -->
        <section id="references" class="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
            <h2 class="mb-4 text-2xl font-semibold text-slate-900 dark:text-slate-50">8. Ссылки</h2>
            <p class="mb-4 text-sm leading-7 text-slate-600 dark:text-slate-400">Базовая терминология и регуляторный контекст, на которые опирается эта методика.</p>
            <div class="space-y-3">
                {#each refs as ref}
                    <a href={ref[1]} target="_blank" rel="noopener noreferrer" class="block rounded-xl border border-slate-200 px-4 py-3 text-sm text-blue-700 hover:bg-slate-50 dark:border-slate-700 dark:text-blue-400 dark:hover:bg-slate-900">
                        {ref[0]}
                    </a>
                {/each}
            </div>
        </section>
    </div>
</main>
