<!--
  @file MethodologyPageUk.svelte
  @description
  Україномовна сторінка документації методології для додатку LC-MS Screening.
  Компонент відображає статичну довідкову сторінку з повним описом конвеєра
  скринінгу — від зчитування Excel до підтвердження реплікатів, віднімання blank
  та формування аудиторського сліду.

  ## Роль компонента

  Це один із трьох локалізованих варіантів сторінки методології (En / Ru / Uk),
  що динамічно обирається маршрутом `src/routes/methodology/+page.svelte`. Кожен
  варіант — самостійний Svelte-компонент із перекладеним вмістом; ключі i18n
  не використовуються, оскільки контент є розгорнутим науковим текстом.

  ## Архітектура даних

  У весь вміст визначено як статичні масиви `const` у блоці `<script>`.
  Кожен масив дотримується кортежного угоди, де позиційні елементи відображаються
  у певні стовпці таблиці або слоти UI у шаблоні. Це розділяє дані та
  представлення без використання CMS чи markdown-конвеєра.

  ## Розділи сторінки

  1. **Вхідні дані**        — очікуваний формат Excel та логіка вибору аркуша
  2. **Колонки Excel**      — схема обов'язкових стовпців (ім'я, тип, опис, приклад)
  3. **Мітки оператора**    — схема кольорового кодування для класифікації рядків
  4. **Алгоритм**           — чотириетапний конвеєр скринінгу з формулами
  5. **Вихідні поля**       — результуючі поля з описами
  6. **Параметри**          — налаштовувані пороги толерантності
  7. **Глосарій**           — визначення термінів предметної області
  8. **Посилання**          — зовнішні посилання для подальшого читання
-->
<script lang="ts">
    import MethodologyVisualizerUk from "./MethodologyVisualizerUk.svelte";

    // ---------------------------------------------------------------------------
    // Масиви даних — таблиці вмісту
    // ---------------------------------------------------------------------------
    // Кожен масив використовує кортежну угоду, де елементи доступні за індексом
    // у шаблоні (наприклад, row[0], row[1]). Це дозволяє не вводити
    // типізований інтерфейс для статичного документаційного вмісту.

    /**
     * Обов'язкові колонки Excel, які скринінговий рушій очікує знайти.
     *
     * Структура кортежу: [ім'я_колонки, тип_даних, опис, приклад_значення]
     *
     * - ім'я_колонки    — точний текст заголовка, який рушій шукає при розборі аркуша
     * - тип_даних       — тип JavaScript/JSON, до якого приводиться значення ("number" | "string")
     * - опис            — людиночитане пояснення того, що представляє колонка
     * - приклад_значення — репрезентативне значення, показане в таблиці документації
     *
     * Збіг колонок виконується без урахування регістру. При наявності кількох
     * аркушів обирається той, у якого найбільша кількість збігів по колонках.
     */
    const columns = [
        ["RT", "number", "Retention time — час утримання піку в колонці", "2.345"],
        ["Base Peak", "number", "m/z базового піку", "195.08"],
        ["Polarity", "string", "Полярність іонізації: positive / negative", "positive"],
        ["File", "string", "Ім'я файлу для ідентифікації зразка і репліки", "1_pos.d"],
        ["Area", "number", "Площа піку", "1250000"],
        ["Label", "string", "Опціональна мітка оператора", "Caffeine"],
    ];

    /**
     * Визначення міток оператора — ручні кольорові коди клітинок, що перекривають
     * евристику за іменем файлу при класифікації рядків.
     *
     * Структура кортежу: [id_мітки, hex_колір, відображувана_мітка]
     *
     * - id_мітки            — внутрішній ідентифікатор, що використовується рушієм для тегування рядків
     * - hex_колір           — точний колір фону, який оператор задає в Excel
     * - відображувана_мітка — людиночитана мітка, показана в UI документації
     *
     * Потік управління: під час попередньої обробки рушій зчитує колір фону кожної
     * клітинки та порівнює його з цими hex-значеннями. Рядок із збігом отримує
     * відповідний SampleType (sample / blank) та індекс репліки, оминаючи
     * евристику за іменем файлу.
     */
    const operatorMarks = [
        ["sample_rep1", "#ff00ff", "Sample, Replicate 1"],
        ["sample_rep2", "#ffff00", "Sample, Replicate 2"],
        ["blank_positive", "#00ffff", "Blank"],
        ["blank_negative", "#00ff00", "Blank"],
    ];

    /**
     * Вихідні поля, що формуються скринінговим рушієм для кожного підтвердженого
     * кластера піків.
     *
     * Структура кортежу: [ім'я_поля, опис]
     *
     * - ім'я_поля — точне ім'я ключа у вихідному JSON/Excel
     * - опис      — що означає поле і як воно обчислюється
     *
     * Ці поля становлять підсумковий результат, придатний для аудиту. Поле `Why`
     * особливо важливе для регуляторної відповідності — воно містить JSON-об'єкт
     * decision trail із документуванням кожного порогового порівняння, що призвело
     * до фінального Status.
     */
    const outputFields = [
        ["RT_mean", "Середній RT підтвердженого кластера."],
        ["MZ_mean", "Середній m/z підтвердженого кластера."],
        ["Area_mean", "Середня площа піку без обрізання до int."],
        ["AreaCVPct", "CV% між площами піків реплікатів."],
        ["ReplicateQuality", "High / Moderate / Low — оцінка якості за CV%."],
        ["SignalToBlankRatio", "S/B ratio для зіставленого blank-піка."],
        ["ConfidenceScore", "Підсумковий показник довіри 0–100."],
        ["Status", "Real Compound або Artifact."],
        ["Why", "JSON-об'єкт із decision trail та деталями порогів."],
    ];

    /**
     * Налаштовувані параметри толерантності, що керують суворістю зіставлення
     * в алгоритмі скринінгу.
     *
     * Структура кортежу: [ім'я_параметра, значення_за_замовч., одиниця, етап_конвеєра]
     *
     * - ім'я_параметра     — точний ключ в об'єкті конфігурації / UI-повзунку
     * - значення_за_замовч. — заводське значення за замовчуванням
     * - одиниця            — фізична або безрозмірна одиниця порогу
     * - етап_конвеєра      — на якому етапі використовується параметр
     *
     * Алгоритмічний вплив:
     *   - replicate_rt_tol / replicate_mz_tol: визначають вікно зіставлення для
     *     підтвердження, що два піки з різних реплікатів представляють одну речовину
     *   - blank_rt_tol / blank_mz_tol: визначають вікно пошуку відповідного
     *     blank-піка
     *   - signal_to_blank_min: мінімальний S/B ratio, нижче якого пік
     *     класифікується як Artifact, а не Real Compound
     */
    const params = [
        ["replicate_rt_tol", "0.1", "хв", "Coarse screening"],
        ["replicate_mz_tol", "0.3", "Da / ppm", "Coarse screening"],
        ["blank_rt_tol", "0.1", "хв", "Blank subtraction"],
        ["blank_mz_tol", "0.3", "Da / ppm", "Blank subtraction"],
        ["signal_to_blank_min", "3.0", "ratio", "Artifact / Real Compound decision"],
    ];

    /**
     * Глосарій термінів предметної області, що використовуються на сторінці методології.
     *
     * Структура кортежу: [термін, визначення]
     *
     * Відображається як двоколонкова сітка карток. Терміни показані
     * моноширинним синім шрифтом для візуального виділення як визначений словник.
     */
    const glossary = [
        ["RT", "Час утримання аналіту в колонці."],
        ["m/z", "Співвідношення маси іона до заряду."],
        ["Replicate", "Незалежне повторне вимірювання того самого зразка."],
        ["Blank", "Холостий зразок для виявлення фонового сигналу."],
        ["CV%", "Відносна варіабельність площ піків між реплікатами."],
        ["S/B ratio", "Signal-to-Blank ratio для зіставленого blank-піка."],
        ["Confidence score", "Узагальнений показник довіри до піка."],
    ];

    /**
     * Зовнішні довідкові посилання для поглибленого вивчення основних концепцій.
     *
     * Структура кортежу: [текст_посилання, url]
     *
     * Кожен запис відображається як вихідне посилання (target="_blank") з
     * rel="noopener noreferrer" для безпеки. Посилання надають науковий
     * та регуляторний контекст, що лежить в основі методології скринінгу.
     */
    const refs = [
        ["Liquid chromatography–mass spectrometry (LC–MS)", "https://en.wikipedia.org/wiki/Liquid_chromatography%E2%80%93mass_spectrometry"],
        ["Mass spectrometry", "https://en.wikipedia.org/wiki/Mass_spectrometry"],
        ["Coefficient of variation", "https://en.wikipedia.org/wiki/Coefficient_of_variation"],
        ["ISO/IEC 17025", "https://en.wikipedia.org/wiki/ISO/IEC_17025"],
    ];
</script>

<!-- Заголовок вкладки браузера для цієї сторінки -->
<svelte:head>
    <title>Методологія — LC-MS Screening</title>
</svelte:head>

<!--
  Основна область вмісту.
  Використовує мінімальну висоту viewport із центрованим контейнером максимальної ширини.
  Класи Tailwind dark: (dark:*) забезпечують коректну тему при перемиканні
    між світлим і темним режимами.
-->
<main class="min-h-screen bg-slate-50 dark:bg-slate-900">
    <div class="mx-auto max-w-5xl px-6 py-12">

        <!-- ----------------------------------------------------------------->
        <!-- Навігація: посилання «Повернутись» на головну панель скринінгу   -->
        <!-- ----------------------------------------------------------------->
        <!--
          href залежить від змінної середовища VITE_STANDALONE:
          - В автономному/PWA режимі → відносний шлях "../" із повним перезавантаженням
          - В режимі SvelteKit SPA   → "/" з клієнтською навігацією (за замовчуванням)
          Атрибут data-sveltekit-reload примусово викликає повне перезавантаження
          сторінки лише в автономному режимі, де клієнтська маршрутизація недоступна.
        -->
        <div class="mb-8">
            <a
                href={import.meta.env.VITE_STANDALONE ? "../" : "/"}
                data-sveltekit-reload={import.meta.env.VITE_STANDALONE ? "" : undefined}
                class="inline-flex items-center gap-2 text-sm text-slate-500 hover:text-slate-900 dark:text-slate-400 dark:hover:text-slate-100"
            >
                <span>←</span>
                <span>Повернутись</span>
            </a>
        </div>

        <!-- ----------------------------------------------------------------->
        <!-- Заголовок сторінки з назвою та вступним описом                   -->
        <!-- ----------------------------------------------------------------->
        <header class="mb-10">
            <h1 class="text-4xl font-bold text-slate-900 dark:text-slate-50">Методологія скринінгу</h1>
            <p class="mt-3 max-w-3xl text-lg text-slate-500 dark:text-slate-400">
                Стисла, але повна схема того, як LC-MS Screening читає Excel, підтверджує піки реплікатів,
                виконує віднімання blank і формує результат, придатний для аудиту.
            </p>
        </header>

        <!-- ----------------------------------------------------------------->
        <!-- Зміст — навігація за якірними посиланнями внутри сторінки        -->
        <!-- ----------------------------------------------------------------->
        <!--
          Кожне посилання веде на id секції, визначений нижче на сторінці.
          Сітка (sm:grid-cols-2) схлопується в одну колонку на мобільних екранах.
        -->
        <nav class="mb-10 rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
            <p class="mb-3 text-xs font-semibold uppercase tracking-[0.2em] text-slate-400 dark:text-slate-500">Зміст</p>
            <div class="grid gap-2 text-sm text-blue-700 dark:text-blue-400 sm:grid-cols-2">
                <a href="#input" class="hover:underline">1. Вхідні дані</a>
                <a href="#columns" class="hover:underline">2. Колонки Excel</a>
                <a href="#marks" class="hover:underline">3. Мітки оператора</a>
                <a href="#algorithm" class="hover:underline">4. Алгоритм</a>
                <a href="#output" class="hover:underline">5. Вихідні поля</a>
                <a href="#params" class="hover:underline">6. Параметри</a>
                <a href="#glossary" class="hover:underline">7. Глосарій</a>
                <a href="#references" class="hover:underline">8. Посилання</a>
            </div>
        </nav>

        <!-- ----------------------------------------------------------------->
        <!-- Інтерактивний візуальний огляд                                   -->
        <!-- ----------------------------------------------------------------->
        <MethodologyVisualizerUk />

        <!-- ----------------------------------------------------------------->
        <!-- Розділ 1: Вхідні дані                                            -->
        <!-- ----------------------------------------------------------------->
        <!--
          Описує очікуваний формат вхідних даних (книга Excel) та евристику
          автоматичного вибору аркуша: при наявності кількох аркушів рушій
          оцінює кожен за кількістю обов'язкових колонок і обирає аркуш
          з найвищим рейтингом.
        -->
        <section id="input" class="mb-10 rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
            <h2 class="mb-3 text-2xl font-semibold text-slate-900 dark:text-slate-50">1. Вхідні дані</h2>
            <div class="space-y-3 text-sm leading-7 text-slate-600 dark:text-slate-400">
                <p>Система очікує Excel-файл із піковими LC-MS даними. Якщо у книзі декілька листів, автоматично вибирається той, де є найбільше обов'язкових колонок.</p>
                <p>Типовий сценарій: два вимірювання реплікатів sample і один blank. Blank використовується як контроль на фон, матрицю та лабораторні артефакти.</p>
            </div>
        </section>

        <!-- ----------------------------------------------------------------->
        <!-- Розділ 2: Обов'язкові колонки Excel                              -->
        <!-- ----------------------------------------------------------------->
        <!--
          Відображає масив `columns` як чотириколонкову HTML-таблицю.
          Блок {#each} ітерує по масиву кортежів, деструктуруючи кожен запис
          за індексом:
            row[0] → ім'я колонки (моноширинний, синій)
            row[1] → тип даних
            row[2] → опис
            row[3] → приклад значення (моноширинний)
        -->
        <section id="columns" class="mb-10">
            <h2 class="mb-4 text-2xl font-semibold text-slate-900 dark:text-slate-50">2. Обов'язкові колонки Excel</h2>
            <div class="overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-sm dark:border-slate-700 dark:bg-slate-800">
                <table class="w-full text-sm">
                    <thead class="bg-slate-50 dark:bg-slate-700">
                        <tr>
                            <th class="px-4 py-3 text-left font-semibold text-slate-700 dark:text-slate-300">Колонка</th>
                            <th class="px-4 py-3 text-left font-semibold text-slate-700 dark:text-slate-300">Тип</th>
                            <th class="px-4 py-3 text-left font-semibold text-slate-700 dark:text-slate-300">Опис</th>
                            <th class="px-4 py-3 text-left font-semibold text-slate-700 dark:text-slate-300">Приклад</th>
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

        <!-- ----------------------------------------------------------------->
        <!-- Розділ 3: Мітки оператора                                        -->
        <!-- ----------------------------------------------------------------->
        <!--
          Відображає масив `operatorMarks` як сітку карток. Кожна картка містить:
            - Кольоровий кружок-зразок (background заданий через inline style = mark[1])
            - Ідентифікатор мітки (mark[0]) моноширинним шрифтом
            - Відображувану мітку (mark[2]) дрібним текстом

          Потік даних: рушій читання Excel зчитує колір фону клітинок і порівнює
          з цими hex-значеннями. При збігу рядку призначається SampleType та
          індекс репліки, перекриваючи евристику за іменем файлу.
        -->
        <section id="marks" class="mb-10 rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
            <h2 class="mb-4 text-2xl font-semibold text-slate-900 dark:text-slate-50">3. Мітки оператора</h2>
            <p class="text-sm leading-7 text-slate-600 dark:text-slate-400">
                Кольорові мітки клітинок у Excel дозволяють явно позначити роль кожного рядка. Якщо вони є, система довіряє їм більше, ніж імені файлу.
            </p>
            <div class="mt-4 grid gap-3 sm:grid-cols-2">
                {#each operatorMarks as mark}
                    <div class="flex items-start gap-3 rounded-xl border border-slate-200 bg-slate-50 p-4 dark:border-slate-700 dark:bg-slate-900">
                        <!-- Кольоровий зразок: inline style встановлює background у hex-колір мітки -->
                        <span class="mt-1 h-4 w-4 rounded-full border border-black/10 dark:border-white/10" style={`background:${mark[1]}`}></span>
                        <div>
                            <p class="font-mono text-sm font-semibold text-slate-900 dark:text-slate-100">{mark[0]}</p>
                            <p class="text-xs text-slate-500 dark:text-slate-400">{mark[2]}</p>
                        </div>
                    </div>
                {/each}
            </div>
        </section>

        <!-- ----------------------------------------------------------------->
        <!-- Розділ 4: Алгоритм — чотириетапний конвеєр скринінгу             -->
        <!-- ----------------------------------------------------------------->
        <!--
          Основний методологічний зміст. Чотири етапи відображаються як
          послідовні картки, кожна описує стадію конвеєра:

          Крок 1 — Попередня обробка:
            Рядки без RT, Base Peak або Area відкидаються (валідація даних).
            Рядки, що пройшли перевірку, отримують SampleType на основі міток
            оператора (найвищий пріоритет) або розбору імені файлу (запасна евристика).

          Крок 2 — Грубий скринінг (кластеризація реплікатів):
            Піки групуються між «кошиками» реплікатів жадібним алгоритмом.
            Правило «не більше одного піку з кошика» запобігає вкладу
            кількох піків одного реплікату в один кластер.
            Підтвердження вимагає збігу І по RT, І по m/z у межах
            відповідних порогів (показано в блоці формул).

          Крок 3 — Blank subtraction:
            Кожен підтверджений sample-кластер зіставляється з blank-піками
            тієї ж полярності іонізації. Обчислюється S/B ratio для пари.
            Логіка класифікації:
              - Artifact:      знайдено збіг із blank І S/B < signal_to_blank_min
              - Real Compound: немає збігу з blank АБО S/B ≥ signal_to_blank_min

          Крок 4 — Summary і аудиторський слід:
            Обчислюється агрегатна статистика по групах SampleType × Polarity.
            JSON-поле `Why` фіксує повний ланцюжок рішень (які пороги
            перевірялися, які значення порівнювалися) для регуляторного аудиту.
        -->
        <section id="algorithm" class="mb-10">
            <h2 class="mb-4 text-2xl font-semibold text-slate-900 dark:text-slate-50">4. Алгоритм</h2>
            <div class="space-y-4">
                <!-- Крок 1: Попередня обробка — валідація даних і класифікація рядків -->
                <div class="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                    <p class="text-sm font-semibold text-slate-900 dark:text-slate-100">Крок 1. Попередня обробка</p>
                    <p class="mt-2 text-sm leading-7 text-slate-600 dark:text-slate-400">Рядки без RT, Base Peak або Area відкидаються. Далі кожному рядку призначається SampleType на основі operator mark або filename fallback.</p>
                </div>

                <!-- Крок 2: Грубий скринінг — кластеризація піків реплікатів у вікнах толерантності -->
                <div class="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                    <p class="text-sm font-semibold text-slate-900 dark:text-slate-100">Крок 2. Грубий скринінг</p>
                    <p class="mt-2 text-sm leading-7 text-slate-600 dark:text-slate-400">Піки кластеризуються між кошиками реплікатів за правилом "не більше одного піку з кошика". Підтвердження вимагає збігу по RT і m/z у межах порогів для реплікатів.</p>
                    <!-- Математична формулювання критеріїв зіставлення -->
                    <div class="mt-3 rounded-xl bg-slate-50 px-4 py-3 font-mono text-xs text-slate-700 dark:bg-slate-900 dark:text-slate-300">
                        <p>|RT₁ − RT₂| ≤ replicate_rt_tol</p>
                        <p>|mz₁ − mz₂| ≤ replicate_mz_tol</p>
                    </div>
                </div>

                <!-- Крок 3: Blank subtraction — виявлення артефактів через S/B ratio -->
                <div class="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                    <p class="text-sm font-semibold text-slate-900 dark:text-slate-100">Крок 3. Blank subtraction</p>
                    <p class="mt-2 text-sm leading-7 text-slate-600 dark:text-slate-400">Підтверджені sample-піки порівнюються з blank-піками тієї ж полярності. Для зіставленої пари обчислюється S/B ratio.</p>
                    <!-- Картки бінарної класифікації -->
                    <div class="mt-3 grid gap-3 sm:grid-cols-2">
                        <div class="rounded-xl border border-red-200 bg-red-50 p-3 text-xs text-red-800 dark:border-red-800 dark:bg-red-950 dark:text-red-300"><strong>Artifact</strong>: знайдено збіг із blank і S/B нижче порога.</div>
                        <div class="rounded-xl border border-green-200 bg-green-50 p-3 text-xs text-green-800 dark:border-green-800 dark:bg-green-950 dark:text-green-300"><strong>Real Compound</strong>: збіг із blank відсутній або S/B достатній.</div>
                    </div>
                </div>

                <!-- Крок 4: Зведена статистика і формування аудиторського сліду -->
                <div class="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                    <p class="text-sm font-semibold text-slate-900 dark:text-slate-100">Крок 4. Summary і аудиторський слід</p>
                    <p class="mt-2 text-sm leading-7 text-slate-600 dark:text-slate-400">Для кожного SampleType / Polarity рахується зведена статистика, а поле <code class="rounded bg-slate-100 px-1 font-mono dark:bg-slate-700">Why</code> зберігає журнал рішення для перевірки та аудиту.</p>
                </div>
            </div>
        </section>

        <!-- ----------------------------------------------------------------->
        <!-- Розділ 5: Вихідні поля                                           -->
        <!-- ----------------------------------------------------------------->
        <!--
          Відображає масив `outputFields` як вертикальний список карток.
          Кожна картка показує ім'я поля (моноширинний синій) та його опис.
          Це колонки, які користувач побачить у фінальному screened Excel-файлі.
        -->
        <section id="output" class="mb-10">
            <h2 class="mb-4 text-2xl font-semibold text-slate-900 dark:text-slate-50">5. Вихідні поля</h2>
            <div class="space-y-3">
                {#each outputFields as field}
                    <div class="rounded-2xl border border-slate-200 bg-white px-5 py-4 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                        <p class="font-mono text-sm font-semibold text-blue-700 dark:text-blue-400">{field[0]}</p>
                        <p class="mt-1 text-sm text-slate-600 dark:text-slate-400">{field[1]}</p>
                    </div>
                {/each}
            </div>
        </section>

        <!-- ----------------------------------------------------------------->
        <!-- Розділ 6: Параметри толерантності                                -->
        <!-- ----------------------------------------------------------------->
        <!--
          Відображає масив `params` як чотириколонкову HTML-таблицю.
          Індекси кортежу відображаються у колонки:
            row[0] → ім'я параметра (моноширинний, синій)
            row[1] → значення за замовчуванням
            row[2] → одиниця виміру
            row[3] → етап конвеєра, що використовує параметр
        -->
        <section id="params" class="mb-10">
            <h2 class="mb-4 text-2xl font-semibold text-slate-900 dark:text-slate-50">6. Параметри толерантності</h2>
            <div class="overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-sm dark:border-slate-700 dark:bg-slate-800">
                <table class="w-full text-sm">
                    <thead class="bg-slate-50 dark:bg-slate-700">
                        <tr>
                            <th class="px-4 py-3 text-left font-semibold text-slate-700 dark:text-slate-300">Параметр</th>
                            <th class="px-4 py-3 text-left font-semibold text-slate-700 dark:text-slate-300">За замовч.</th>
                            <th class="px-4 py-3 text-left font-semibold text-slate-700 dark:text-slate-300">Одиниця</th>
                            <th class="px-4 py-3 text-left font-semibold text-slate-700 dark:text-slate-300">Застосування</th>
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

        <!-- ----------------------------------------------------------------->
        <!-- Розділ 7: Глосарій                                               -->
        <!-- ----------------------------------------------------------------->
        <!--
          Відображає масив `glossary` як двоколонкову сітку карток.
          Кожна картка показує термін (моноширинний синій) та його визначення.
          Забезпечує швидку довідку за термінами предметної області,
          що використовуються в документації методології.
        -->
        <section id="glossary" class="mb-10 rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
            <h2 class="mb-4 text-2xl font-semibold text-slate-900 dark:text-slate-50">7. Глосарій</h2>
            <div class="grid gap-3 sm:grid-cols-2">
                {#each glossary as item}
                    <div class="rounded-xl border border-slate-200 bg-slate-50 p-4 dark:border-slate-700 dark:bg-slate-900">
                        <p class="font-mono text-sm font-semibold text-blue-700 dark:text-blue-400">{item[0]}</p>
                        <p class="mt-1 text-sm text-slate-600 dark:text-slate-400">{item[1]}</p>
                    </div>
                {/each}
            </div>
        </section>

        <!-- ----------------------------------------------------------------->
        <!-- Розділ 8: Посилання                                              -->
        <!-- ----------------------------------------------------------------->
        <!--
          Відображає масив `refs` як вертикальний список вихідних посилань.
          Кожне посилання відкривається в новій вкладці (target="_blank") з
          атрибутами безпеки (rel="noopener noreferrer") для запобігання
          tab-napping-атакам.
        -->
        <section id="references" class="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
            <h2 class="mb-4 text-2xl font-semibold text-slate-900 dark:text-slate-50">8. Посилання</h2>
            <p class="mb-4 text-sm leading-7 text-slate-600 dark:text-slate-400">Базові терміни й регуляторний контекст, на які спирається ця методика.</p>
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
