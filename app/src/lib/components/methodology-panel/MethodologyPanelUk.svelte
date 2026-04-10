<script lang="ts">
    import TermTooltip from "../TermTooltip.svelte";
    import MethodologyVisualizerUk from "../methodology-page/MethodologyVisualizerUk.svelte";

    const gl: Record<string, string> = {
        RT: "Retention Time — час утримання аналіту в хроматографічній колонці (хвилини). Кожна сполука має характерний RT, що використовується для ідентифікації.",
        "m/z": "Mass-to-charge ratio — відношення маси іона до його заряду. Центральна координата мас-спектрометричного сигналу.",
        "CV%": "Coefficient of Variation — відносна варіабельність площ піків між реплікатами. CV% = (вибіркове std / mean) × 100. Для n=2: std = |v₁−v₂|/√2. Нижчий CV означає кращу відтворюваність.",
        "S/B": "Signal-to-Blank ratio — відношення площі піку зразка до площі найближчого підтвердженого піку в blank.",
        blank: "Зразок-холостий (розчинник без аналіту). Піки, присутні в blank, вважаються фоновими артефактами.",
        replicate: "Повторне незалежне вимірювання того самого зразка. Мінімум 2 реплікати (бакети) потрібні для підтвердження піку та обчислення CV%.",
        "confidence score": "Підсумковий показник довіри 0–100. Починається з 100, знижується за відхиленням RT/m/z, CV% і S/B ratio. Деталі — у розділі 3б.",
        ppm: "Parts per million — відносний допуск по m/z для high-resolution мас-спектрометрії.",
        Da: "Dalton — абсолютний допуск по m/z у дальтонах.",
        "replicate bucket": "Кошик реплікатів — усі піки з одного файлу або з однаковою міткою оператора. Кожен файл/мітка = окремий бакет.",
    };

    const columns = [
        { name: "RT", type: "number", desc: "Retention time — час утримання піку в колонці", example: "2.345" },
        { name: "Base Peak", type: "number", desc: "m/z базового піку (основний іон у спектрі)", example: "195.08" },
        { name: "Polarity", type: "string", desc: "Полярність іонізації: positive / negative", example: "positive" },
        { name: "File", type: "string", desc: "Ім'я вихідного файлу для визначення бакету реплікатів", example: "1_pos.d" },
        { name: "Area", type: "number", desc: "Площа піку, пропорційна концентрації сполуки", example: "1250000" },
        { name: "Label", type: "string", desc: "(Опціонально) мітка або назва піку від оператора", example: "Caffeine" },
    ];

    const outputFields = [
        { name: "RT_mean", desc: "Середнє RT підтвердженого кластера (по всіх реплікатах)." },
        { name: "MZ_mean", desc: "Середнє m/z підтвердженого кластера." },
        { name: "Area_mean", desc: "Середня площа піку (без округлення до int)." },
        { name: "AreaCVPct", desc: "CV% між площами реплікатів кластера (вибіркова std)." },
        { name: "ReplicateQuality", desc: "High (CV% ≤ 15) / Moderate (≤ 30) / Low (> 30) — категорія відтворюваності." },
        { name: "SignalToBlankRatio", desc: "S/B ratio для найближчого підтвердженого blank-піка." },
        { name: "ConfidenceScore", desc: "Підсумковий показник довіри 0–100 після blank subtraction." },
        { name: "Status", desc: "Real Compound або Artifact — результат класифікації." },
        { name: "Why", desc: "JSON-об'єкт з повним аудиторським слідом рішення (RT/mz delta, CV, S/B, поріг)." },
    ];

    const params = [
        { name: "replicate_rt_tol", val: "0.1", unit: "хв", where: "Кластеризація реплікатів" },
        { name: "replicate_mz_tol", val: "0.3", unit: "Da / ppm", where: "Кластеризація реплікатів" },
        { name: "blank_rt_tol", val: "0.1", unit: "хв", where: "Blank subtraction" },
        { name: "blank_mz_tol", val: "0.3", unit: "Da / ppm", where: "Blank subtraction" },
        { name: "signal_to_blank_min", val: "3.0", unit: "ratio", where: "Artifact / Real Compound" },
        { name: "cv_high_max", val: "15", unit: "%", where: "ReplicateQuality = High" },
        { name: "cv_moderate_max", val: "30", unit: "%", where: "ReplicateQuality = Moderate" },
    ];
</script>

<div class="space-y-10 text-sm">
    <MethodologyVisualizerUk />

    <section id="m-input">
        <h3 class="mb-3 border-b border-slate-200 pb-2 text-base font-bold text-slate-900 dark:border-slate-700 dark:text-slate-50">
            1. Вхідні дані
        </h3>
        <p class="leading-relaxed text-slate-600 dark:text-slate-400">
            Система приймає <strong>Excel-файл (.xlsx / .xls)</strong> з результатами LC-MS аналізу,
            автоматично обирає найрелевантніший лист і очікує дані мінімум двох
            <TermTooltip term="реплікатів" def={gl.replicate} /> (окремі файли або мітки оператора) та
            <TermTooltip term="blank" def={gl.blank} />-зразка.
        </p>
    </section>

    <section id="m-columns">
        <h3 class="mb-3 border-b border-slate-200 pb-2 text-base font-bold text-slate-900 dark:border-slate-700 dark:text-slate-50">
            2. Обов'язкові колонки Excel
        </h3>
        <div class="overflow-hidden rounded-xl border border-slate-200 dark:border-slate-700">
            <table class="w-full">
                <thead class="bg-slate-50 dark:bg-slate-700">
                    <tr>
                        <th class="px-4 py-2.5 text-left font-semibold text-slate-700 dark:text-slate-300">Колонка</th>
                        <th class="px-4 py-2.5 text-left font-semibold text-slate-700 dark:text-slate-300">Тип</th>
                        <th class="px-4 py-2.5 text-left font-semibold text-slate-700 dark:text-slate-300">Опис</th>
                        <th class="px-4 py-2.5 text-left font-semibold text-slate-700 dark:text-slate-300">Приклад</th>
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
            <strong>Мітки оператора:</strong> кольорові мітки клітинок Excel (<code class="font-mono text-xs">sample_rep1</code>, <code class="font-mono text-xs">sample_rep2</code>, <code class="font-mono text-xs">blank_positive</code>, <code class="font-mono text-xs">blank_negative</code>) пріоритизуються над іменем файлу. За відсутності міток тип рядка визначається з імені файлу: файли, що містять «blank» → blank; файли виду <code class="font-mono text-xs">1_*.d</code>, <code class="font-mono text-xs">2_*.d</code> → <code class="font-mono text-xs">sample_1</code>, <code class="font-mono text-xs">sample_2</code> тощо.
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
                    <p class="font-semibold text-slate-900 dark:text-slate-100">Попередня обробка</p>
                    <p class="mt-1 leading-relaxed text-slate-600 dark:text-slate-400">
                        Файл зчитується, вибирається найкращий лист, видаляються рядки без
                        <TermTooltip term="RT" def={gl.RT} />, <TermTooltip term="m/z" def={gl["m/z"]} />
                        або Area. Кожному рядку призначається <code class="rounded bg-slate-100 px-1 text-xs font-mono dark:bg-slate-700">SampleType</code> (blank / sample_N) та
                        розподіл по <TermTooltip term="бакетах реплікатів" def={gl["replicate bucket"]} />.
                        Потім рядки групуються за парою (SampleType, Polarity) — кожна група обробляється незалежно.
                    </p>
                </div>
            </div>

            <div class="flex gap-3 rounded-xl border border-slate-200 bg-white p-4 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                <span class="flex h-7 w-7 flex-shrink-0 items-center justify-center rounded-full bg-blue-50 text-xs font-bold text-blue-600 dark:bg-blue-950 dark:text-blue-300">2</span>
                <div class="flex-1">
                    <p class="font-semibold text-slate-900 dark:text-slate-100">Жадібна кластеризація реплікатів</p>
                    <p class="mt-1 leading-relaxed text-slate-600 dark:text-slate-400">
                        Піки сортуються за спаданням площі (Area). Кожен пік по черзі стає насінням кластера.
                        З кожного іншого бакету жадібно обирається найближчий незайнятий пік до поточного
                        <strong>центроїда кластера</strong> (RT, m/z усереднюються при додаванні кожного члена).
                        Кластер підтверджується, якщо містить піки з <strong>≥ 2 різних бакетів</strong>.
                    </p>
                    <div class="mt-3 rounded-lg border border-slate-200 bg-slate-50 px-4 py-3 font-mono text-xs dark:border-slate-700 dark:bg-slate-900">
                        <p class="mb-1 text-[10px] font-semibold uppercase tracking-wider text-slate-400">Умова включення піку до кластера</p>
                        <p class="text-slate-700 dark:text-slate-300">|RT_кандидат − RT_центроїд| ≤ replicate_rt_tol</p>
                        <p class="text-slate-700 dark:text-slate-300">|mz_кандидат − mz_центроїд| ≤ replicate_mz_tol (<TermTooltip term="Da" def={gl.Da} /> або <TermTooltip term="ppm" def={gl.ppm} />)</p>
                        <p class="mt-2 text-[10px] text-slate-400">При кількох збігах — обирається пік з мінімальною відстанню (RT fraction + mz fraction), при рівності — з більшою площею.</p>
                    </div>
                </div>
            </div>

            <div class="flex gap-3 rounded-xl border border-slate-200 bg-white p-4 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                <span class="flex h-7 w-7 flex-shrink-0 items-center justify-center rounded-full bg-blue-50 text-xs font-bold text-blue-600 dark:bg-blue-950 dark:text-blue-300">3</span>
                <div class="flex-1">
                    <p class="font-semibold text-slate-900 dark:text-slate-100">Blank subtraction</p>
                    <p class="mt-1 leading-relaxed text-slate-600 dark:text-slate-400">
                        Blank також проходить кластеризацію (крок 2) незалежно. Потім кожен підтверджений
                        sample-пік зіставляється з підтвердженими піками
                        <TermTooltip term="blank" def={gl.blank} /> у межах blank_rt_tol / blank_mz_tol.
                        Обирається найближчий збіг (за RT+mz відстанню, при рівності — більший blank).
                    </p>
                    <div class="mt-3 grid gap-2 sm:grid-cols-2">
                        <div class="rounded-lg border border-red-200 bg-red-50 px-3 py-2 text-xs text-red-800 dark:border-red-800 dark:bg-red-950 dark:text-red-300">
                            <strong>Artifact</strong> — знайдено збіг із blank і S/B &lt; signal_to_blank_min (або blank_area = 0)
                        </div>
                        <div class="rounded-lg border border-green-200 bg-green-50 px-3 py-2 text-xs text-green-800 dark:border-green-800 dark:bg-green-950 dark:text-green-300">
                            <strong>Real Compound</strong> — збіг із blank відсутній або S/B ≥ поріг
                        </div>
                    </div>
                </div>
            </div>

            <div class="flex gap-3 rounded-xl border border-slate-200 bg-white p-4 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                <span class="flex h-7 w-7 flex-shrink-0 items-center justify-center rounded-full bg-indigo-50 text-xs font-bold text-indigo-600 dark:bg-indigo-950 dark:text-indigo-300">Σ</span>
                <div class="flex-1">
                    <p class="font-semibold text-slate-900 dark:text-slate-100">Confidence Score — формула нарахування</p>
                    <p class="mt-1 mb-3 text-slate-500 dark:text-slate-400">Старт: 100 балів. Штрафи накладаються послідовно, результат затискається у [0, 100].</p>
                    <div class="space-y-1.5 rounded-lg border border-slate-200 bg-slate-50 px-4 py-3 font-mono text-xs dark:border-slate-700 dark:bg-slate-900">
                        <p class="text-[10px] font-semibold uppercase tracking-wider text-slate-400 mb-2">Етап 1 — реплікати</p>
                        <p class="text-slate-700 dark:text-slate-300">RT proximity&nbsp;&nbsp;&nbsp;− (mean_RT_delta / rt_tol) × 20&nbsp;&nbsp;<span class="text-slate-400">max −20</span></p>
                        <p class="text-slate-700 dark:text-slate-300">mz proximity&nbsp;&nbsp;− (mean_mz_delta / mz_tol) × 25&nbsp;&nbsp;<span class="text-slate-400">max −25</span></p>
                        <p class="text-slate-700 dark:text-slate-300">CV% High&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;0</p>
                        <p class="text-slate-700 dark:text-slate-300">CV% Moderate&nbsp;&nbsp;−12</p>
                        <p class="text-slate-700 dark:text-slate-300">CV% Low&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;−(12 + (CV − cv_moderate_max) × 0.7),&nbsp;<span class="text-slate-400">max −35</span></p>
                        <p class="text-slate-700 dark:text-slate-300">CV% відсутній −10</p>
                        <p class="text-slate-700 dark:text-slate-300">Не colour-paired −5</p>
                        <p class="text-[10px] font-semibold uppercase tracking-wider text-slate-400 mt-3 mb-2">Етап 2 — blank</p>
                        <p class="text-slate-700 dark:text-slate-300">Немає blank-збігу&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;+3</p>
                        <p class="text-slate-700 dark:text-slate-300">S/B ≥ threshold&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;+min(5, (S/B − threshold) × 0.5)</p>
                        <p class="text-slate-700 dark:text-slate-300">S/B &lt; threshold (Artifact) −(15 + deficit × 30),&nbsp;<span class="text-slate-400">max −45</span></p>
                    </div>
                </div>
            </div>

            <div class="flex gap-3 rounded-xl border border-slate-200 bg-white p-4 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                <span class="flex h-7 w-7 flex-shrink-0 items-center justify-center rounded-full bg-blue-50 text-xs font-bold text-blue-600 dark:bg-blue-950 dark:text-blue-300">4</span>
                <div>
                    <p class="font-semibold text-slate-900 dark:text-slate-100">Summary — зведена статистика</p>
                    <p class="mt-1 text-slate-600 dark:text-slate-400">
                        Для кожної пари (SampleType, Polarity) система формує Total / Confirmed / Artifacts / Real Compounds,
                        а також середній CV%, середній Confidence Score та середній S/B ratio.
                    </p>
                </div>
            </div>
        </div>
    </section>

    <section id="m-output">
        <h3 class="mb-3 border-b border-slate-200 pb-2 text-base font-bold text-slate-900 dark:border-slate-700 dark:text-slate-50">
            4. Вихідні поля
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
            5. Параметри
        </h3>
        <div class="overflow-hidden rounded-xl border border-slate-200 dark:border-slate-700">
            <table class="w-full">
                <thead class="bg-slate-50 dark:bg-slate-700">
                    <tr>
                        <th class="px-4 py-2.5 text-left font-semibold text-slate-700 dark:text-slate-300">Параметр</th>
                        <th class="px-4 py-2.5 text-left font-semibold text-slate-700 dark:text-slate-300">За замовч.</th>
                        <th class="px-4 py-2.5 text-left font-semibold text-slate-700 dark:text-slate-300">Одиниця</th>
                        <th class="px-4 py-2.5 text-left font-semibold text-slate-700 dark:text-slate-300">Застосування</th>
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
            Параметри rt/mz/S/B можна змінювати у формі на головній сторінці. cv_high_max і cv_moderate_max наразі фіксовані.
        </p>
    </section>

    <section id="m-glossary">
        <h3 class="mb-3 border-b border-slate-200 pb-2 text-base font-bold text-slate-900 dark:border-slate-700 dark:text-slate-50">
            6. Глосарій
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
