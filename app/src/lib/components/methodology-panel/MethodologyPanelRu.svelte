<script lang="ts">
    import TermTooltip from "../TermTooltip.svelte";

    const gl: Record<string, string> = {
        RT: "Retention Time — время удерживания аналита в хроматографической колонке (минуты).",
        "m/z": "Mass-to-charge ratio — центральная координата масс-спектрометрического сигнала.",
        "CV%": "Coefficient of Variation — относительная вариабельность площадей пиков репликатов. CV% = (выборочное std / mean) × 100. Для n=2: std = |v₁−v₂|/√2. Ниже CV — лучше воспроизводимость.",
        "S/B": "Signal-to-Blank ratio — площадь sample-пика делённая на площадь ближайшего подтверждённого blank-пика.",
        blank: "Холостой образец (только растворитель, без аналита), используемый для выявления фоновых артефактов.",
        replicate: "Независимое повторное измерение одного образца. Минимум 2 корзины репликатов нужны для подтверждения пика.",
        "confidence score": "Итоговый показатель доверия 0–100. Начинается с 100, снижается за отклонения RT/m/z, CV% и S/B. Подробности — в разделе 3б.",
        ppm: "Parts per million — относительный допуск по m/z для high-resolution приборов.",
        Da: "Dalton — абсолютный допуск по m/z в дальтонах.",
        "replicate bucket": "Корзина репликатов — все пики из одного файла или с одной операторской меткой.",
    };

    const columns = [
        { name: "RT", type: "number", desc: "Время удерживания хроматографического пика", example: "2.345" },
        { name: "Base Peak", type: "number", desc: "m/z доминирующего иона в спектре", example: "195.08" },
        { name: "Polarity", type: "string", desc: "Полярность ионизации: positive / negative", example: "positive" },
        { name: "File", type: "string", desc: "Имя исходного файла для определения корзины репликатов", example: "1_pos.d" },
        { name: "Area", type: "number", desc: "Площадь пика, пропорциональная концентрации аналита", example: "1250000" },
        { name: "Label", type: "string", desc: "(Опционально) метка оператора или имя соединения", example: "Caffeine" },
    ];

    const outputFields = [
        { name: "RT_mean", desc: "Среднее RT подтверждённого кластера (по всем репликатам)." },
        { name: "MZ_mean", desc: "Среднее m/z подтверждённого кластера." },
        { name: "Area_mean", desc: "Средняя площадь пика (без округления до int)." },
        { name: "AreaCVPct", desc: "CV% между площадями репликатов кластера (выборочное std)." },
        { name: "ReplicateQuality", desc: "High (CV% ≤ 15) / Moderate (≤ 30) / Low (> 30) — категория воспроизводимости." },
        { name: "SignalToBlankRatio", desc: "S/B ratio для ближайшего подтверждённого blank-пика." },
        { name: "ConfidenceScore", desc: "Итоговый показатель доверия 0–100 после blank subtraction." },
        { name: "Status", desc: "Real Compound или Artifact — результат классификации." },
        { name: "Why", desc: "JSON-объект с полным аудиторским следом решения (RT/mz delta, CV, S/B, пороги)." },
    ];

    const params = [
        { name: "replicate_rt_tol", val: "0.1", unit: "мин", where: "Кластеризация репликатов" },
        { name: "replicate_mz_tol", val: "0.3", unit: "Da / ppm", where: "Кластеризация репликатов" },
        { name: "blank_rt_tol", val: "0.1", unit: "мин", where: "Blank subtraction" },
        { name: "blank_mz_tol", val: "0.3", unit: "Da / ppm", where: "Blank subtraction" },
        { name: "signal_to_blank_min", val: "3.0", unit: "ratio", where: "Artifact / Real Compound" },
        { name: "cv_high_max", val: "15", unit: "%", where: "ReplicateQuality = High" },
        { name: "cv_moderate_max", val: "30", unit: "%", where: "ReplicateQuality = Moderate" },
    ];
</script>

<div class="space-y-10 text-sm">
    <section id="m-input">
        <h3 class="mb-3 border-b border-slate-200 pb-2 text-base font-bold text-slate-900 dark:border-slate-700 dark:text-slate-50">
            1. Входные данные
        </h3>
        <p class="leading-relaxed text-slate-600 dark:text-slate-400">
            Система принимает <strong>Excel-файл (.xlsx / .xls)</strong> с результатами LC-MS анализа,
            автоматически выбирает наиболее подходящий лист и ожидает минимум два
            <TermTooltip term="репликата" def={gl.replicate} /> (отдельные файлы или метки оператора) и
            <TermTooltip term="blank" def={gl.blank} />-образец.
        </p>
    </section>

    <section id="m-columns">
        <h3 class="mb-3 border-b border-slate-200 pb-2 text-base font-bold text-slate-900 dark:border-slate-700 dark:text-slate-50">
            2. Обязательные колонки Excel
        </h3>
        <div class="overflow-hidden rounded-xl border border-slate-200 dark:border-slate-700">
            <table class="w-full">
                <thead class="bg-slate-50 dark:bg-slate-700">
                    <tr>
                        <th class="px-4 py-2.5 text-left font-semibold text-slate-700 dark:text-slate-300">Колонка</th>
                        <th class="px-4 py-2.5 text-left font-semibold text-slate-700 dark:text-slate-300">Тип</th>
                        <th class="px-4 py-2.5 text-left font-semibold text-slate-700 dark:text-slate-300">Описание</th>
                        <th class="px-4 py-2.5 text-left font-semibold text-slate-700 dark:text-slate-300">Пример</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-slate-100 dark:divide-slate-700">
                    {#each columns as col}
                        <tr class="hover:bg-slate-50/60 dark:hover:bg-slate-700/40">
                            <td class="px-4 py-2.5 font-mono font-semibold text-blue-700 dark:text-blue-400">{col.name}</td>
                            <td class="px-4 py-2.5 text-slate-500 dark:text-slate-400">{col.type}</td>
                            <td class="px-4 py-2.5 text-slate-700 dark:text-slate-300">{col.desc}</td>
                            <td class="px-4 py-2.5 font-mono text-slate-500 dark:text-slate-400">{col.example}</td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>
        <p class="mt-3 rounded-lg border border-amber-200 bg-amber-50 px-4 py-2.5 text-amber-800 dark:border-amber-700 dark:bg-amber-950 dark:text-amber-200">
            <strong>Метки оператора:</strong> цветовые метки ячеек Excel (<code class="font-mono text-xs">sample_rep1</code>, <code class="font-mono text-xs">sample_rep2</code>, <code class="font-mono text-xs">blank_positive</code>, <code class="font-mono text-xs">blank_negative</code>) имеют приоритет над именем файла. При их отсутствии тип строки определяется из имени файла: содержащие «blank» → blank; файлы вида <code class="font-mono text-xs">1_*.d</code>, <code class="font-mono text-xs">2_*.d</code> → <code class="font-mono text-xs">sample_1</code>, <code class="font-mono text-xs">sample_2</code> и т.д.
        </p>
    </section>

    <section id="m-algo">
        <h3 class="mb-4 border-b border-slate-200 pb-2 text-base font-bold text-slate-900 dark:border-slate-700 dark:text-slate-50">
            3. Алгоритм
        </h3>
        <div class="space-y-3">
            <div class="flex gap-3 rounded-xl border border-slate-200 bg-white p-4 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                <span class="flex h-7 w-7 flex-shrink-0 items-center justify-center rounded-full bg-blue-50 text-xs font-bold text-blue-600 dark:bg-blue-950 dark:text-blue-300">1</span>
                <div>
                    <p class="font-semibold text-slate-900 dark:text-slate-100">Предобработка</p>
                    <p class="mt-1 leading-relaxed text-slate-600 dark:text-slate-400">
                        Файл считывается, выбирается лучший лист, удаляются строки без
                        <TermTooltip term="RT" def={gl.RT} />, <TermTooltip term="m/z" def={gl["m/z"]} />
                        или Area. Каждой строке назначается <code class="rounded bg-slate-100 px-1 text-xs font-mono dark:bg-slate-700">SampleType</code> (blank / sample_N) и
                        <TermTooltip term="корзина репликатов" def={gl["replicate bucket"]} />.
                        Затем строки группируются по паре (SampleType, Polarity) — каждая группа обрабатывается независимо.
                    </p>
                </div>
            </div>

            <div class="flex gap-3 rounded-xl border border-slate-200 bg-white p-4 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                <span class="flex h-7 w-7 flex-shrink-0 items-center justify-center rounded-full bg-blue-50 text-xs font-bold text-blue-600 dark:bg-blue-950 dark:text-blue-300">2</span>
                <div class="flex-1">
                    <p class="font-semibold text-slate-900 dark:text-slate-100">Жадная кластеризация репликатов</p>
                    <p class="mt-1 leading-relaxed text-slate-600 dark:text-slate-400">
                        Пики сортируются по убыванию площади (Area). Каждый пик по очереди становится зерном кластера.
                        Из каждой другой корзины жадно выбирается ближайший свободный пик к текущему
                        <strong>центроиду кластера</strong> (RT, m/z усредняются при добавлении каждого члена).
                        Кластер подтверждается, если содержит пики из <strong>≥ 2 разных корзин</strong>.
                    </p>
                    <div class="mt-3 rounded-lg border border-slate-200 bg-slate-50 px-4 py-3 font-mono text-xs dark:border-slate-700 dark:bg-slate-900">
                        <p class="mb-1 text-[10px] font-semibold uppercase tracking-wider text-slate-400">Условие включения пика в кластер</p>
                        <p class="text-slate-700 dark:text-slate-300">|RT_кандидат − RT_центроид| ≤ replicate_rt_tol</p>
                        <p class="text-slate-700 dark:text-slate-300">|mz_кандидат − mz_центроид| ≤ replicate_mz_tol (<TermTooltip term="Da" def={gl.Da} /> или <TermTooltip term="ppm" def={gl.ppm} />)</p>
                        <p class="mt-2 text-[10px] text-slate-400">При нескольких совпадениях — выбирается пик с минимальным расстоянием (RT fraction + mz fraction), при равенстве — с большей площадью.</p>
                    </div>
                </div>
            </div>

            <div class="flex gap-3 rounded-xl border border-slate-200 bg-white p-4 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                <span class="flex h-7 w-7 flex-shrink-0 items-center justify-center rounded-full bg-blue-50 text-xs font-bold text-blue-600 dark:bg-blue-950 dark:text-blue-300">3</span>
                <div class="flex-1">
                    <p class="font-semibold text-slate-900 dark:text-slate-100">Blank subtraction</p>
                    <p class="mt-1 leading-relaxed text-slate-600 dark:text-slate-400">
                        Blank также проходит кластеризацию (шаг 2) независимо. Затем каждый подтверждённый
                        sample-пик сопоставляется с подтверждёнными пиками
                        <TermTooltip term="blank" def={gl.blank} /> в пределах blank_rt_tol / blank_mz_tol.
                        Выбирается ближайшее совпадение (по RT+mz расстоянию, при равенстве — больший blank).
                    </p>
                    <div class="mt-3 grid gap-2 sm:grid-cols-2">
                        <div class="rounded-lg border border-red-200 bg-red-50 px-3 py-2 text-xs text-red-800 dark:border-red-800 dark:bg-red-950 dark:text-red-300">
                            <strong>Artifact</strong> — найдено совпадение с blank и S/B &lt; signal_to_blank_min (или blank_area = 0)
                        </div>
                        <div class="rounded-lg border border-green-200 bg-green-50 px-3 py-2 text-xs text-green-800 dark:border-green-800 dark:bg-green-950 dark:text-green-300">
                            <strong>Real Compound</strong> — совпадение с blank отсутствует или S/B ≥ порог
                        </div>
                    </div>
                </div>
            </div>

            <div class="flex gap-3 rounded-xl border border-slate-200 bg-white p-4 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                <span class="flex h-7 w-7 flex-shrink-0 items-center justify-center rounded-full bg-indigo-50 text-xs font-bold text-indigo-600 dark:bg-indigo-950 dark:text-indigo-300">Σ</span>
                <div class="flex-1">
                    <p class="font-semibold text-slate-900 dark:text-slate-100">Confidence Score — формула начисления</p>
                    <p class="mt-1 mb-3 text-slate-500 dark:text-slate-400">Старт: 100 баллов. Штрафы применяются последовательно; результат зажимается в [0, 100].</p>
                    <div class="space-y-1.5 rounded-lg border border-slate-200 bg-slate-50 px-4 py-3 font-mono text-xs dark:border-slate-700 dark:bg-slate-900">
                        <p class="text-[10px] font-semibold uppercase tracking-wider text-slate-400 mb-2">Этап 1 — репликаты</p>
                        <p class="text-slate-700 dark:text-slate-300">RT proximity&nbsp;&nbsp;&nbsp;− (mean_RT_delta / rt_tol) × 20&nbsp;&nbsp;<span class="text-slate-400">макс −20</span></p>
                        <p class="text-slate-700 dark:text-slate-300">mz proximity&nbsp;&nbsp;− (mean_mz_delta / mz_tol) × 25&nbsp;&nbsp;<span class="text-slate-400">макс −25</span></p>
                        <p class="text-slate-700 dark:text-slate-300">CV% High&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;0</p>
                        <p class="text-slate-700 dark:text-slate-300">CV% Moderate&nbsp;&nbsp;−12</p>
                        <p class="text-slate-700 dark:text-slate-300">CV% Low&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;−(12 + (CV − cv_moderate_max) × 0.7),&nbsp;<span class="text-slate-400">макс −35</span></p>
                        <p class="text-slate-700 dark:text-slate-300">CV% отсутствует −10</p>
                        <p class="text-slate-700 dark:text-slate-300">Не colour-paired −5</p>
                        <p class="text-[10px] font-semibold uppercase tracking-wider text-slate-400 mt-3 mb-2">Этап 2 — blank</p>
                        <p class="text-slate-700 dark:text-slate-300">Нет blank-совпадения&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;+3</p>
                        <p class="text-slate-700 dark:text-slate-300">S/B ≥ threshold&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;+min(5, (S/B − threshold) × 0.5)</p>
                        <p class="text-slate-700 dark:text-slate-300">S/B &lt; threshold (Artifact) −(15 + deficit × 30),&nbsp;<span class="text-slate-400">макс −45</span></p>
                    </div>
                </div>
            </div>

            <div class="flex gap-3 rounded-xl border border-slate-200 bg-white p-4 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                <span class="flex h-7 w-7 flex-shrink-0 items-center justify-center rounded-full bg-blue-50 text-xs font-bold text-blue-600 dark:bg-blue-950 dark:text-blue-300">4</span>
                <div>
                    <p class="font-semibold text-slate-900 dark:text-slate-100">Summary — сводная статистика</p>
                    <p class="mt-1 text-slate-600 dark:text-slate-400">
                        Для каждой пары (SampleType, Polarity) система формирует Total / Confirmed / Artifacts / Real Compounds,
                        а также средний CV%, средний Confidence Score и средний S/B ratio.
                    </p>
                </div>
            </div>
        </div>
    </section>

    <section id="m-output">
        <h3 class="mb-3 border-b border-slate-200 pb-2 text-base font-bold text-slate-900 dark:border-slate-700 dark:text-slate-50">
            4. Выходные поля
        </h3>
        <div class="space-y-2">
            {#each outputFields as f}
                <div class="flex items-start gap-3 rounded-xl border border-slate-200 bg-white px-4 py-3 dark:border-slate-700 dark:bg-slate-800">
                    <code class="min-w-[150px] flex-shrink-0 font-mono font-semibold text-blue-700 dark:text-blue-400">{f.name}</code>
                    <p class="text-slate-600 dark:text-slate-400">{f.desc}</p>
                </div>
            {/each}
        </div>
    </section>

    <section id="m-params">
        <h3 class="mb-3 border-b border-slate-200 pb-2 text-base font-bold text-slate-900 dark:border-slate-700 dark:text-slate-50">
            5. Параметры
        </h3>
        <div class="overflow-hidden rounded-xl border border-slate-200 dark:border-slate-700">
            <table class="w-full">
                <thead class="bg-slate-50 dark:bg-slate-700">
                    <tr>
                        <th class="px-4 py-2.5 text-left font-semibold text-slate-700 dark:text-slate-300">Параметр</th>
                        <th class="px-4 py-2.5 text-left font-semibold text-slate-700 dark:text-slate-300">По умолч.</th>
                        <th class="px-4 py-2.5 text-left font-semibold text-slate-700 dark:text-slate-300">Единица</th>
                        <th class="px-4 py-2.5 text-left font-semibold text-slate-700 dark:text-slate-300">Применение</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-slate-100 dark:divide-slate-700">
                    {#each params as p}
                        <tr class="hover:bg-slate-50/60 dark:hover:bg-slate-700/40">
                            <td class="px-4 py-2.5 font-mono font-semibold text-blue-700 dark:text-blue-400">{p.name}</td>
                            <td class="px-4 py-2.5 font-mono text-slate-700 dark:text-slate-300">{p.val}</td>
                            <td class="px-4 py-2.5 text-slate-500 dark:text-slate-400">{p.unit}</td>
                            <td class="px-4 py-2.5 text-slate-600 dark:text-slate-400">{p.where}</td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>
        <p class="mt-2 text-xs text-slate-400 dark:text-slate-500">
            Параметры rt/mz/S/B можно изменять в форме на главной странице. cv_high_max и cv_moderate_max в настоящее время фиксированы.
        </p>
    </section>

    <section id="m-glossary">
        <h3 class="mb-3 border-b border-slate-200 pb-2 text-base font-bold text-slate-900 dark:border-slate-700 dark:text-slate-50">
            6. Глоссарий
        </h3>
        <div class="grid gap-2 sm:grid-cols-2">
            {#each Object.entries(gl) as [term, def]}
                <div class="rounded-xl border border-slate-200 bg-white px-4 py-3 dark:border-slate-700 dark:bg-slate-800">
                    <p class="font-mono font-semibold text-blue-700 dark:text-blue-400">{term}</p>
                    <p class="mt-1 text-xs leading-relaxed text-slate-600 dark:text-slate-400">{def}</p>
                </div>
            {/each}
        </div>
    </section>
</div>
