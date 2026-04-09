<script lang="ts">
    const columns = [
        ["RT", "number", "Время удерживания хроматографического пика", "2.345"],
        ["Base Peak", "number", "m/z доминирующего иона", "195.08"],
        ["Polarity", "string", "Полярность ионизации: positive / negative", "positive"],
        ["File", "string", "Имя файла для идентификации образца и репликата", "1_pos.d"],
        ["Area", "number", "Площадь пика", "1250000"],
        ["Label", "string", "Опциональная метка оператора", "Caffeine"],
    ];

    const operatorMarks = [
        ["sample_rep1", "#ff00ff", "Sample, Replicate 1"],
        ["sample_rep2", "#ffff00", "Sample, Replicate 2"],
        ["blank_positive", "#00ffff", "Blank"],
        ["blank_negative", "#00ff00", "Blank"],
    ];

    const outputFields = [
        ["RT_mean", "Средний RT подтверждённого кластера."],
        ["MZ_mean", "Средний m/z подтверждённого кластера."],
        ["Area_mean", "Средняя площадь пика без округления до int."],
        ["AreaCVPct", "CV% между площадями пиков репликатов."],
        ["ReplicateQuality", "High / Moderate / Low — категория качества."],
        ["SignalToBlankRatio", "S/B ratio для сопоставленного blank-пика."],
        ["ConfidenceScore", "Итоговый показатель доверия 0–100."],
        ["Status", "Real Compound или Artifact."],
        ["Why", "JSON decision trail с деталями порогов."],
    ];

    const params = [
        ["replicate_rt_tol", "0.1", "мин", "Coarse screening"],
        ["replicate_mz_tol", "0.3", "Da / ppm", "Coarse screening"],
        ["blank_rt_tol", "0.1", "мин", "Blank subtraction"],
        ["blank_mz_tol", "0.3", "Da / ppm", "Blank subtraction"],
        ["signal_to_blank_min", "3.0", "ratio", "Artifact / Real Compound decision"],
    ];

    const glossary = [
        ["RT", "Время удерживания аналита в LC-колонке."],
        ["m/z", "Отношение массы иона к заряду."],
        ["Replicate", "Независимое повторное измерение одного и того же образца."],
        ["Blank", "Холостой контроль для выявления фонового сигнала."],
        ["CV%", "Относительная вариабельность площадей пиков между репликатами."],
        ["S/B ratio", "Signal-to-Blank ratio для сопоставленного blank-пика."],
        ["Confidence score", "Обобщённая метрика доверия к screened-пику."],
    ];

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
                <a href="#columns" class="hover:underline">2. Колонки Excel</a>
                <a href="#marks" class="hover:underline">3. Метки оператора</a>
                <a href="#algorithm" class="hover:underline">4. Алгоритм</a>
                <a href="#output" class="hover:underline">5. Выходные поля</a>
                <a href="#params" class="hover:underline">6. Параметры</a>
                <a href="#glossary" class="hover:underline">7. Глоссарий</a>
                <a href="#references" class="hover:underline">8. Ссылки</a>
            </div>
        </nav>

        <section id="input" class="mb-10 rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
            <h2 class="mb-3 text-2xl font-semibold text-slate-900 dark:text-slate-50">1. Входные данные</h2>
            <div class="space-y-3 text-sm leading-7 text-slate-600 dark:text-slate-400">
                <p>Приложение ожидает Excel-книгу с LC-MS пиковыми данными. Если листов несколько, автоматически выбирается лист с наилучшим совпадением по обязательным колонкам.</p>
                <p>Типичный сценарий: два измерения репликатов sample и один blank. Blank используется как контроль на фон, матрицу и лабораторные артефакты.</p>
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

        <section id="marks" class="mb-10 rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
            <h2 class="mb-4 text-2xl font-semibold text-slate-900 dark:text-slate-50">3. Метки оператора</h2>
            <p class="text-sm leading-7 text-slate-600 dark:text-slate-400">
                Ручная окраска ячеек в Excel позволяет явно задать роль каждой строки. Если такие метки есть, система доверяет им больше, чем эвристике по имени файла.
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

        <section id="algorithm" class="mb-10">
            <h2 class="mb-4 text-2xl font-semibold text-slate-900 dark:text-slate-50">4. Алгоритм</h2>
            <div class="space-y-4">
                <div class="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                    <p class="text-sm font-semibold text-slate-900 dark:text-slate-100">Шаг 1. Предобработка</p>
                    <p class="mt-2 text-sm leading-7 text-slate-600 dark:text-slate-400">Строки без RT, Base Peak или Area отбрасываются. Каждой оставшейся строке присваивается SampleType на основе operator marks или file-name fallback.</p>
                </div>
                <div class="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                    <p class="text-sm font-semibold text-slate-900 dark:text-slate-100">Шаг 2. Грубый скрининг</p>
                    <p class="mt-2 text-sm leading-7 text-slate-600 dark:text-slate-400">Пики кластеризуются между корзинами репликатов по правилу “не более одного пика из корзины”. Подтверждение требует совпадения по RT и m/z в пределах порогов для репликатов.</p>
                    <div class="mt-3 rounded-xl bg-slate-50 px-4 py-3 font-mono text-xs text-slate-700 dark:bg-slate-900 dark:text-slate-300">
                        <p>|RT₁ − RT₂| ≤ replicate_rt_tol</p>
                        <p>|mz₁ − mz₂| ≤ replicate_mz_tol</p>
                    </div>
                </div>
                <div class="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                    <p class="text-sm font-semibold text-slate-900 dark:text-slate-100">Шаг 3. Blank subtraction</p>
                    <p class="mt-2 text-sm leading-7 text-slate-600 dark:text-slate-400">Подтверждённые sample-пики сравниваются с blank-пиками той же полярности. Для сопоставленной пары вычисляется S/B ratio.</p>
                    <div class="mt-3 grid gap-3 sm:grid-cols-2">
                        <div class="rounded-xl border border-red-200 bg-red-50 p-3 text-xs text-red-800 dark:border-red-800 dark:bg-red-950 dark:text-red-300"><strong>Artifact</strong>: найдено совпадение с blank и S/B ниже порога.</div>
                        <div class="rounded-xl border border-green-200 bg-green-50 p-3 text-xs text-green-800 dark:border-green-800 dark:bg-green-950 dark:text-green-300"><strong>Real Compound</strong>: совпадение с blank отсутствует или S/B достаточно высок.</div>
                    </div>
                </div>
                <div class="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                    <p class="text-sm font-semibold text-slate-900 dark:text-slate-100">Шаг 4. Summary и аудиторский след</p>
                    <p class="mt-2 text-sm leading-7 text-slate-600 dark:text-slate-400">Приложение считает summary-метрики по SampleType / Polarity и сохраняет логику решения в <code class="rounded bg-slate-100 px-1 font-mono dark:bg-slate-700">Why</code> для проверки и аудита.</p>
                </div>
            </div>
        </section>

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

        <section id="params" class="mb-10">
            <h2 class="mb-4 text-2xl font-semibold text-slate-900 dark:text-slate-50">6. Параметры толерантности</h2>
            <div class="overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-sm dark:border-slate-700 dark:bg-slate-800">
                <table class="w-full text-sm">
                    <thead class="bg-slate-50 dark:bg-slate-700">
                        <tr>
                            <th class="px-4 py-3 text-left font-semibold text-slate-700 dark:text-slate-300">Параметр</th>
                            <th class="px-4 py-3 text-left font-semibold text-slate-700 dark:text-slate-300">По умолч.</th>
                            <th class="px-4 py-3 text-left font-semibold text-slate-700 dark:text-slate-300">Единица</th>
                            <th class="px-4 py-3 text-left font-semibold text-slate-700 dark:text-slate-300">Использование</th>
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
