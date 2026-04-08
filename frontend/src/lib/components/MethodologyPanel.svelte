<script lang="ts">
    import TermTooltip from "./TermTooltip.svelte";

    const gl: Record<string, string> = {
        RT: "Retention Time — час утримання аналіту в хроматографічній колонці (хвилини). Кожна сполука має характерний RT, що використовується для ідентифікації.",
        "m/z": "Mass-to-charge ratio — відношення маси іона до його заряду. Центральна координата мас-спектрометричного сигналу.",
        "CV%": "Coefficient of Variation — відносна варіабельність replicate areas. CV% = (std / mean) × 100. Нижчий CV означає кращу відтворюваність.",
        "S/B": "Signal-to-Blank ratio — відношення площі піку зразка до площі відповідного піку в blank. Використовується замість бінарного blank subtraction.",
        blank: "Зразок-холостий (розчинник без аналіту). Піки, присутні в blank, вважаються фоновими артефактами або забрудненнями, не пов'язаними з аналітом.",
        replicate: "Повторне незалежне вимірювання того самого зразка. Мінімум 2 реплікати потрібні для підтвердження піку та обчислення CV%.",
        "confidence score": "Зведений показник довіри 0–100 на основі узгодженості реплік (CV%), результату blank subtraction та S/B ratio.",
        ppm: "Parts per million — відносний допуск по m/z, типовий для high-resolution мас-спектрометрії. Масштабується з масою іона.",
        Da: "Dalton — абсолютний допуск по m/z у дальтонах. Використовується для low-resolution або legacy даних із грубим округленням m/z.",
        "replicate bucket": "Кошик реплік — група піків з одного файлу (або з однаковою міткою оператора), що представляє одне повторне вимірювання.",
    };

    const columns = [
        { name: "RT", type: "number", desc: "Retention time — час утримання піку в колонці", example: "2.345" },
        { name: "Base Peak", type: "number", desc: "m/z значення базового піку (основний іон у спектрі)", example: "195.08" },
        { name: "Polarity", type: "string", desc: "Полярність іонізації: positive / negative", example: "positive" },
        { name: "File", type: "string", desc: "Ім'я вихідного файлу (ідентифікація зразка та репліки)", example: "1_pos.d" },
        { name: "Area", type: "number", desc: "Площа піку — пропорційна концентрації сполуки", example: "1250000" },
        { name: "Label", type: "string", desc: "(Опціонально) Мітка або назва піку від оператора", example: "Caffeine" },
    ];

    const outputFields = [
        { name: "RT_mean", desc: "Середнє RT підтвердженого кластера (avg(RTᵢ), 4 знаки)." },
        { name: "MZ_mean", desc: "Середнє m/z підтвердженого кластера (вища точність для audit trail)." },
        { name: "Area_mean", desc: "Середня площа піку кластера (не округлюється до int)." },
        { name: "AreaCVPct", desc: "CV% між replicate areas — ключовий показник відтворюваності." },
        { name: "ReplicateQuality", desc: "High / Moderate / Low — категорія якості за CV%." },
        { name: "SignalToBlankRatio", desc: "S/B ratio для зіставленого піку в blank (matched blank peak)." },
        { name: "ConfidenceScore", desc: "Підсумковий показник довіри 0–100." },
        { name: "Status", desc: "Real Compound або Artifact — результат класифікації." },
        { name: "Why", desc: "JSON-об'єкт з повним audit trail рішення (modal «Logic Detail»)." },
    ];

    const params = [
        { name: "replicate_rt_tol", val: "0.1", unit: "хв", where: "Coarse screening" },
        { name: "replicate_mz_tol", val: "0.3", unit: "Da / ppm", where: "Coarse screening" },
        { name: "blank_rt_tol", val: "0.1", unit: "хв", where: "Blank subtraction" },
        { name: "blank_mz_tol", val: "0.3", unit: "Da / ppm", where: "Blank subtraction" },
        { name: "signal_to_blank_min", val: "3.0", unit: "ratio", where: "Real / Artifact decision" },
    ];
</script>

<div class="space-y-10 text-sm">

    <!-- 1. Input -->
    <section id="m-input">
        <h3 class="mb-3 border-b border-slate-200 pb-2 text-base font-bold text-slate-900 dark:border-slate-700 dark:text-slate-50">
            1. Вхідні дані
        </h3>
        <p class="text-slate-600 dark:text-slate-400 leading-relaxed">
            Система приймає <strong>Excel-файл (.xlsx / .xls)</strong> з результатами LC-MS аналізу.
            Автоматично обирається лист з найбільшою кількістю обов'язкових колонок.
            Типовий файл містить дані двох
            <TermTooltip term="реплікатів" def={gl.replicate} />
            та
            <TermTooltip term="blank" def={gl.blank} />-зразка.
        </p>
    </section>

    <!-- 2. Columns -->
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
            <strong>Мітки оператора:</strong> якщо клітинки Excel пофарбовані вручну
            (<code class="font-mono">sample_rep1</code>, <code class="font-mono">blank_positive</code> тощо) —
            вони пріоритизуються над іменем файлу для визначення типу рядка.
        </p>
    </section>

    <!-- 3. Algorithm -->
    <section id="m-algo">
        <h3 class="mb-4 border-b border-slate-200 pb-2 text-base font-bold text-slate-900 dark:border-slate-700 dark:text-slate-50">
            3. Алгоритм
        </h3>
        <div class="space-y-3">

            <!-- Step 1 -->
            <div class="flex gap-3 rounded-xl border border-slate-200 bg-white p-4 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                <span class="flex h-7 w-7 flex-shrink-0 items-center justify-center rounded-full bg-blue-50 text-xs font-bold text-blue-600 dark:bg-blue-950 dark:text-blue-300">1</span>
                <div>
                    <p class="font-semibold text-slate-900 dark:text-slate-100">Попередня обробка</p>
                    <p class="mt-1 text-slate-600 dark:text-slate-400 leading-relaxed">
                        Зчитування файлу, вибір листа, видалення рядків з порожніми
                        <TermTooltip term="RT" def={gl.RT} />,
                        <TermTooltip term="m/z" def={gl["m/z"]} /> або Area.
                        Кожному рядку присвоюється <code class="font-mono text-xs bg-slate-100 dark:bg-slate-700 px-1 rounded">SampleType</code>
                        на основі кольорової мітки або імені файлу.
                    </p>
                </div>
            </div>

            <!-- Step 2 -->
            <div class="flex gap-3 rounded-xl border border-slate-200 bg-white p-4 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                <span class="flex h-7 w-7 flex-shrink-0 items-center justify-center rounded-full bg-blue-50 text-xs font-bold text-blue-600 dark:bg-blue-950 dark:text-blue-300">2</span>
                <div class="flex-1">
                    <p class="font-semibold text-slate-900 dark:text-slate-100">Coarse screening — підтвердження реплік</p>
                    <p class="mt-1 text-slate-600 dark:text-slate-400 leading-relaxed">
                        Жадібна кластеризація піків між
                        <TermTooltip term="replicate buckets" def={gl["replicate bucket"]} />
                        (мін. 2 кошики) за
                        <TermTooltip term="RT" def={gl.RT} /> та
                        <TermTooltip term="m/z" def={gl["m/z"]} />.
                        Для підтвердженого кластера обчислюється
                        <TermTooltip term="CV%" def={gl["CV%"]} /> та
                        <TermTooltip term="confidence score" def={gl["confidence score"]} />.
                    </p>
                    <div class="mt-3 rounded-lg border border-slate-200 bg-slate-50 px-4 py-3 font-mono text-xs dark:border-slate-700 dark:bg-slate-900">
                        <p class="mb-1 text-[10px] font-semibold uppercase tracking-wider text-slate-400">Умова підтвердження</p>
                        <p class="text-slate-700 dark:text-slate-300">|RT₁ − RT₂| ≤ replicate_rt_tol</p>
                        <p class="text-slate-700 dark:text-slate-300">|mz₁ − mz₂| ≤ replicate_mz_tol <span class="font-sans text-slate-400">(<TermTooltip term="Da" def={gl.Da} /> або <TermTooltip term="ppm" def={gl.ppm} />)</span></p>
                    </div>
                </div>
            </div>

            <!-- Step 3 -->
            <div class="flex gap-3 rounded-xl border border-slate-200 bg-white p-4 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                <span class="flex h-7 w-7 flex-shrink-0 items-center justify-center rounded-full bg-blue-50 text-xs font-bold text-blue-600 dark:bg-blue-950 dark:text-blue-300">3</span>
                <div class="flex-1">
                    <p class="font-semibold text-slate-900 dark:text-slate-100">Blank subtraction</p>
                    <p class="mt-1 text-slate-600 dark:text-slate-400 leading-relaxed">
                        Зіставлення підтверджених піків sample із підтвердженими піками
                        <TermTooltip term="blank" def={gl.blank} />.
                        Обчислюється
                        <TermTooltip term="S/B" def={gl["S/B"]} /> ratio.
                    </p>
                    <div class="mt-3 grid gap-2 sm:grid-cols-2">
                        <div class="rounded-lg border border-red-200 bg-red-50 px-3 py-2 text-xs text-red-800 dark:border-red-800 dark:bg-red-950 dark:text-red-300">
                            <strong>Artifact</strong> — blank match знайдено і S/B &lt; signal_to_blank_min
                        </div>
                        <div class="rounded-lg border border-green-200 bg-green-50 px-3 py-2 text-xs text-green-800 dark:border-green-800 dark:bg-green-950 dark:text-green-300">
                            <strong>Real Compound</strong> — blank match відсутній або S/B ≥ поріг
                        </div>
                    </div>
                </div>
            </div>

            <!-- Step 4 -->
            <div class="flex gap-3 rounded-xl border border-slate-200 bg-white p-4 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                <span class="flex h-7 w-7 flex-shrink-0 items-center justify-center rounded-full bg-blue-50 text-xs font-bold text-blue-600 dark:bg-blue-950 dark:text-blue-300">4</span>
                <div>
                    <p class="font-semibold text-slate-900 dark:text-slate-100">Summary — зведена статистика</p>
                    <p class="mt-1 text-slate-600 dark:text-slate-400">
                        Для кожної пари (SampleType, Polarity): Total / Confirmed / Artifacts / Real Compounds,
                        середній <TermTooltip term="CV%" def={gl["CV%"]} />,
                        середній <TermTooltip term="confidence score" def={gl["confidence score"]} />.
                    </p>
                </div>
            </div>
        </div>
    </section>

    <!-- 4. Output -->
    <section id="m-output">
        <h3 class="mb-3 border-b border-slate-200 pb-2 text-base font-bold text-slate-900 dark:border-slate-700 dark:text-slate-50">
            4. Вихідні поля
        </h3>
        <div class="space-y-2">
            {#each outputFields as f}
                <div class="flex items-start gap-3 rounded-xl border border-slate-200 bg-white px-4 py-3 dark:border-slate-700 dark:bg-slate-800">
                    <code class="min-w-[140px] flex-shrink-0 font-mono font-semibold text-blue-700 dark:text-blue-400">{f.name}</code>
                    <p class="text-slate-600 dark:text-slate-400">{f.desc}</p>
                </div>
            {/each}
        </div>
    </section>

    <!-- 5. Parameters -->
    <section id="m-params">
        <h3 class="mb-3 border-b border-slate-200 pb-2 text-base font-bold text-slate-900 dark:border-slate-700 dark:text-slate-50">
            5. Параметри толерантності
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
            Параметри можна змінити у формі на головній сторінці. Replicate matching і blank subtraction налаштовуються незалежно.
        </p>
    </section>

    <!-- 6. Glossary -->
    <section id="m-glossary">
        <h3 class="mb-3 border-b border-slate-200 pb-2 text-base font-bold text-slate-900 dark:border-slate-700 dark:text-slate-50">
            6. Глосарій
        </h3>
        <div class="grid gap-2 sm:grid-cols-2">
            {#each Object.entries(gl) as [term, def]}
                <div class="rounded-xl border border-slate-200 bg-white px-4 py-3 dark:border-slate-700 dark:bg-slate-800">
                    <p class="font-mono font-semibold text-blue-700 dark:text-blue-400">{term}</p>
                    <p class="mt-1 text-xs text-slate-600 leading-relaxed dark:text-slate-400">{def}</p>
                </div>
            {/each}
        </div>
    </section>

</div>
