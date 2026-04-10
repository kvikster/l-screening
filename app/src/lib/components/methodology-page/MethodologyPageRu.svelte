<!--
  @file MethodologyPageRu.svelte
  @description
  Русскоязычная страница документации методологии для приложения LC-MS Screening.
  Компонент отображает статичную справочную страницу с полным описанием конвейера
  скрининга — от чтения Excel до подтверждения репликатов, вычитания blank и
  формирования аудиторского следа.

  ## Роль компонента

  Это один из трёх локализованных вариантов страницы методологии (En / Ru / Uk),
  динамически выбираемый маршрутом `src/routes/methodology/+page.svelte`. Каждый
  вариант — самостоятельный Svelte-компонент с переведённым содержимым; ключи
  i18n не используются, так как контент представляет собой развёрнутый научный текст.

  ## Архитектура данных

  Всё содержимое определено как статические массивы `const` в блоке `<script>`.
  Каждый массив следует кортежному соглашению, где позиционные элементы отображаются
  в определённые столбцы таблицы или слоты UI в шаблоне. Это разделяет данные
  и представление без использования CMS или markdown-конвейера.

  ## Разделы страницы

  1. **Входные данные**      — ожидаемый формат Excel и логика выбора листа
  2. **Колонки Excel**       — схема обязательных столбцов (имя, тип, описание, пример)
  3. **Метки оператора**     — схема цветовой кодировки для классификации строк
  4. **Алгоритм**            — четырёхэтапный конвейер скрининга с формулами
  5. **Выходные поля**       — результирующие поля с описаниями
  6. **Параметры**           — настраиваемые пороги толерантности
  7. **Глоссарий**           — определения терминов предметной области
  8. **Ссылки**              — внешние ссылки для дополнительного чтения
-->
<script lang="ts">
    import MethodologyVisualizerRu from "./MethodologyVisualizerRu.svelte";

    // ---------------------------------------------------------------------------
    // Массивы данных — таблицы содержимого
    // ---------------------------------------------------------------------------
    // Каждый массив использует кортежное соглашение, где элементы доступны по
    // индексу в шаблоне (например, row[0], row[1]). Это позволяет не вводить
    // типизированный интерфейс для статического документационного содержимого.

    /**
     * Обязательные колонки Excel, которые скрининговый движок ожидает найти.
     *
     * Структура кортежа: [имя_колонки, тип_данных, описание, пример_значения]
     *
     * - имя_колонки   — точный текст заголовка, который движок ищет при разборе листа
     * - тип_данных    — тип JavaScript/JSON, к которому приводится значение ("number" | "string")
     * - описание      — человекочитаемое объяснение того, что представляет колонка
     * - пример_значения — репрезентативное значение, показанное в таблице документации
     *
     * Совпадение колонок выполняется без учёта регистра. При наличии нескольких
     * листов выбирается тот, у которого наибольшее число совпадений по колонкам.
     */
    const columns = [
        ["RT", "number", "Время удерживания хроматографического пика", "2.345"],
        ["Base Peak", "number", "m/z доминирующего иона", "195.08"],
        ["Polarity", "string", "Полярность ионизации: positive / negative", "positive"],
        ["File", "string", "Имя файла для идентификации образца и репликата", "1_pos.d"],
        ["Area", "number", "Площадь пика", "1250000"],
        ["Label", "string", "Опциональная метка оператора", "Caffeine"],
    ];

    /**
     * Определения меток оператора — ручные цветовые коды ячеек, перекрывающие
     * эвристику по имени файла при классификации строк.
     *
     * Структура кортежа: [id_метки, hex_цвет, отображаемая_метка]
     *
     * - id_метки         — внутренний идентификатор, используемый движком для тегирования строк
     * - hex_цвет         — точный цвет фона, который оператор задаёт в Excel
     * - отображаемая_метка — человекочитаемая метка, показанная в UI документации
     *
     * Поток управления: при предобработке движок читает цвет фона каждой ячейки
     * и сравнивает его с этими hex-значениями. Совпадающая строка получает
     * соответствующий SampleType (sample / blank) и индекс репликата, минуя
     * эвристику по имени файла.
     */
    const operatorMarks = [
        ["sample_rep1", "#ff00ff", "Sample, Replicate 1"],
        ["sample_rep2", "#ffff00", "Sample, Replicate 2"],
        ["blank_positive", "#00ffff", "Blank"],
        ["blank_negative", "#00ff00", "Blank"],
    ];

    /**
     * Выходные поля, формируемые скрининговым движком для каждого подтверждённого
     * кластера пиков.
     *
     * Структура кортежа: [имя_поля, описание]
     *
     * - имя_поля  — точное имя ключа в выходном JSON/Excel
     * - описание  — что означает поле и как оно вычисляется
     *
     * Эти поля составляют итоговый результат, пригодный для аудита. Поле `Why`
     * особенно важно для регуляторного соответствия — оно содержит JSON decision
     * trail с документированием каждого порогового сравнения, приведшего к
     * финальному Status.
     */
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

    /**
     * Настраиваемые параметры толерантности, управляющие строгостью сопоставления
     * в алгоритме скрининга.
     *
     * Структура кортежа: [имя_параметра, значение_по_умолч., единица, этап_конвейера]
     *
     * - имя_параметра     — точный ключ в объекте конфигурации / UI-слайдере
     * - значение_по_умолч. — заводское значение по умолчанию
     * - единица           — физическая или безразмерная единица порога
     * - этап_конвейера    — на каком этапе используется параметр
     *
     * Алгоритмическое влияние:
     *   - replicate_rt_tol / replicate_mz_tol: определяют окно сопоставления для
     *     подтверждения, что два пика из разных репликатов представляют одно вещество
     *   - blank_rt_tol / blank_mz_tol: определяют окно поиска соответствующего
     *     blank-пика
     *   - signal_to_blank_min: минимальный S/B ratio, ниже которого пик
     *     классифицируется как Artifact, а не Real Compound
     */
    const params = [
        ["replicate_rt_tol", "0.1", "мин", "Coarse screening"],
        ["replicate_mz_tol", "0.3", "Da / ppm", "Coarse screening"],
        ["blank_rt_tol", "0.1", "мин", "Blank subtraction"],
        ["blank_mz_tol", "0.3", "Da / ppm", "Blank subtraction"],
        ["signal_to_blank_min", "3.0", "ratio", "Artifact / Real Compound decision"],
    ];

    /**
     * Глоссарий терминов предметной области, используемых на странице методологии.
     *
     * Структура кортежа: [термин, определение]
     *
     * Отображается в виде двухколоночной сетки карточек. Термины показаны
     * моноширинным синим шрифтом для визуального выделения как определённый словарь.
     */
    const glossary = [
        ["RT", "Время удерживания аналита в LC-колонке."],
        ["m/z", "Отношение массы иона к заряду."],
        ["Replicate", "Независимое повторное измерение одного и того же образца."],
        ["Blank", "Холостой контроль для выявления фонового сигнала."],
        ["CV%", "Относительная вариабельность площадей пиков между репликатами."],
        ["S/B ratio", "Signal-to-Blank ratio для сопоставленного blank-пика."],
        ["Confidence score", "Обобщённая метрика доверия к screened-пику."],
    ];

    /**
     * Внешние справочные ссылки для углублённого изучения основных концепций.
     *
     * Структура кортежа: [текст_ссылки, url]
     *
     * Каждая запись отображается как исходящая ссылка (target="_blank") с
     * rel="noopener noreferrer" для безопасности. Ссылки предоставляют научный
     * и регуляторный контекст, лежащий в основе методологии скрининга.
     */
    const refs = [
        ["Liquid chromatography–mass spectrometry (LC–MS)", "https://en.wikipedia.org/wiki/Liquid_chromatography%E2%80%93mass_spectrometry"],
        ["Mass spectrometry", "https://en.wikipedia.org/wiki/Mass_spectrometry"],
        ["Coefficient of variation", "https://en.wikipedia.org/wiki/Coefficient_of_variation"],
        ["ISO/IEC 17025", "https://en.wikipedia.org/wiki/ISO/IEC_17025"],
    ];
</script>

<!-- Заголовок вкладки браузера для данной страницы -->
<svelte:head>
    <title>Методология — LC-MS Screening</title>
</svelte:head>

<!--
  Основная область содержимого.
  Использует минимальную высоту viewport с центрированным контейнером максимальной ширины.
  Классы Tailwind dark: (dark:*) обеспечивают корректную тему при переключении
    между светлым и тёмным режимами.
-->
<main class="min-h-screen bg-slate-50 dark:bg-slate-900">
    <div class="mx-auto max-w-5xl px-6 py-12">

        <!-- ----------------------------------------------------------------->
        <!-- Навигация: ссылка «Назад» на главную панель скрининга            -->
        <!-- ----------------------------------------------------------------->
        <!--
          href зависит от переменной окружения VITE_STANDALONE:
          - В автономном/PWA режиме → относительный путь "../" с полной перезагрузкой
          - В режиме SvelteKit SPA  → "/" с клиентской навигацией (по умолчанию)
          Атрибут data-sveltekit-reload принудительно вызывает полную перезагрузку
          страницы только в автономном режиме, где клиентская маршрутизация недоступна.
        -->
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

        <!-- ----------------------------------------------------------------->
        <!-- Заголовок страницы с названием и вводным описанием               -->
        <!-- ----------------------------------------------------------------->
        <header class="mb-10">
            <h1 class="text-4xl font-bold text-slate-900 dark:text-slate-50">Методология скрининга</h1>
            <p class="mt-3 max-w-3xl text-lg text-slate-500 dark:text-slate-400">
                Компактное, но полное описание того, как LC-MS Screening читает Excel, подтверждает пики репликатов,
                выполняет вычитание blank и формирует результат, пригодный для аудита.
            </p>
        </header>

        <!-- ----------------------------------------------------------------->
        <!-- Содержание — навигация по якорным ссылкам внутри страницы        -->
        <!-- ----------------------------------------------------------------->
        <!--
          Каждая ссылка ведёт на id секции, определённый ниже на странице.
          Сетка (sm:grid-cols-2) схлопывается в одну колонку на мобильных экранах.
        -->
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

        <!-- ----------------------------------------------------------------->
        <!-- Интерактивный визуальный обзор                                   -->
        <!-- ----------------------------------------------------------------->
        <MethodologyVisualizerRu />

        <!-- ----------------------------------------------------------------->
        <!-- Раздел 1: Входные данные                                         -->
        <!-- ----------------------------------------------------------------->
        <!--
          Описывает ожидаемый формат входных данных (книга Excel) и эвристику
          автоматического выбора листа: при наличии нескольких листов движок
          оценивает каждый по числу обязательных колонок и выбирает лист
          с наивысшим рейтингом.
        -->
        <section id="input" class="mb-10 rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
            <h2 class="mb-3 text-2xl font-semibold text-slate-900 dark:text-slate-50">1. Входные данные</h2>
            <div class="space-y-3 text-sm leading-7 text-slate-600 dark:text-slate-400">
                <p>Приложение ожидает Excel-книгу с LC-MS пиковыми данными. Если листов несколько, автоматически выбирается лист с наилучшим совпадением по обязательным колонкам.</p>
                <p>Типичный сценарий: два измерения репликатов sample и один blank. Blank используется как контроль на фон, матрицу и лабораторные артефакты.</p>
            </div>
        </section>

        <!-- ----------------------------------------------------------------->
        <!-- Раздел 2: Обязательные колонки Excel                             -->
        <!-- ----------------------------------------------------------------->
        <!--
          Отображает массив `columns` как четырёхколоночную HTML-таблицу.
          Блок {#each} итерирует по массиву кортежей, извлекая элементы по индексу:
            row[0] → имя колонки (моноширинный, синий)
            row[1] → тип данных
            row[2] → описание
            row[3] → пример значения (моноширинный)
        -->
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

        <!-- ----------------------------------------------------------------->
        <!-- Раздел 3: Метки оператора                                        -->
        <!-- ----------------------------------------------------------------->
        <!--
          Отображает массив `operatorMarks` как сетку карточек. Каждая карточка
          содержит:
            - Цветной кружок-образец (background задан через inline style = mark[1])
            - Идентификатор метки (mark[0]) моноширинным шрифтом
            - Отображаемую метку (mark[2]) мелким текстом

          Поток данных: движок чтения Excel считывает цвет фона ячеек и сравнивает
          с этими hex-значениями. При совпадении строке назначается SampleType и
          индекс репликата, перекрывая эвристику по имени файла.
        -->
        <section id="marks" class="mb-10 rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
            <h2 class="mb-4 text-2xl font-semibold text-slate-900 dark:text-slate-50">3. Метки оператора</h2>
            <p class="text-sm leading-7 text-slate-600 dark:text-slate-400">
                Ручная окраска ячеек в Excel позволяет явно задать роль каждой строки. Если такие метки есть, система доверяет им больше, чем эвристике по имени файла.
            </p>
            <div class="mt-4 grid gap-3 sm:grid-cols-2">
                {#each operatorMarks as mark}
                    <div class="flex items-start gap-3 rounded-xl border border-slate-200 bg-slate-50 p-4 dark:border-slate-700 dark:bg-slate-900">
                        <!-- Цветной образец: inline style устанавливает background в hex-цвет метки -->
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
        <!-- Раздел 4: Алгоритм — четырёхэтапный конвейер скрининга           -->
        <!-- ----------------------------------------------------------------->
        <!--
          Основное методологическое содержание. Четыре этапа отображаются
          как последовательные карточки, каждая описывает стадию конвейера:

          Шаг 1 — Предобработка:
            Строки без RT, Base Peak или Area отбрасываются (валидация данных).
            Оставшиеся строки получают SampleType на основе меток оператора
            (наивысший приоритет) или разбора имени файла (запасная эвристика).

          Шаг 2 — Грубый скрининг (кластеризация репликатов):
            Пики группируются между «корзинами» репликатов жадным алгоритмом.
            Правило «не более одного пика из корзины» предотвращает вклад
            нескольких пиков одного репликата в один кластер.
            Подтверждение требует совпадения И по RT, И по m/z в пределах
            соответствующих порогов (показано в блоке формул).

          Шаг 3 — Blank subtraction:
            Каждый подтверждённый sample-кластер сопоставляется с blank-пиками
            той же полярности ионизации. Вычисляется S/B ratio для пары.
            Логика классификации:
              - Artifact:      найдено совпадение с blank И S/B < signal_to_blank_min
              - Real Compound: нет совпадения с blank ИЛИ S/B ≥ signal_to_blank_min

          Шаг 4 — Summary и аудиторский след:
            Вычисляется агрегатная статистика по группам SampleType × Polarity.
            JSON-поле `Why` фиксирует полную цепочку решений (какие пороги
            проверялись, какие значения сравнивались) для регуляторного аудита.
        -->
        <section id="algorithm" class="mb-10">
            <h2 class="mb-4 text-2xl font-semibold text-slate-900 dark:text-slate-50">4. Алгоритм</h2>
            <div class="space-y-4">
                <!-- Шаг 1: Предобработка — валидация данных и классификация строк -->
                <div class="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                    <p class="text-sm font-semibold text-slate-900 dark:text-slate-100">Шаг 1. Предобработка</p>
                    <p class="mt-2 text-sm leading-7 text-slate-600 dark:text-slate-400">Строки без RT, Base Peak или Area отбрасываются. Каждой оставшейся строке присваивается SampleType на основе operator marks или file-name fallback.</p>
                </div>

                <!-- Шаг 2: Грубый скрининг — кластеризация пиков репликатов в окнах толерантности -->
                <div class="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                    <p class="text-sm font-semibold text-slate-900 dark:text-slate-100">Шаг 2. Грубый скрининг</p>
                    <p class="mt-2 text-sm leading-7 text-slate-600 dark:text-slate-400">Пики кластеризуются между корзинами репликатов по правилу "не более одного пика из корзины". Подтверждение требует совпадения по RT и m/z в пределах порогов для репликатов.</p>
                    <!-- Математическая формулировка критериев сопоставления -->
                    <div class="mt-3 rounded-xl bg-slate-50 px-4 py-3 font-mono text-xs text-slate-700 dark:bg-slate-900 dark:text-slate-300">
                        <p>|RT₁ − RT₂| ≤ replicate_rt_tol</p>
                        <p>|mz₁ − mz₂| ≤ replicate_mz_tol</p>
                    </div>
                </div>

                <!-- Шаг 3: Blank subtraction — обнаружение артефактов через S/B ratio -->
                <div class="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                    <p class="text-sm font-semibold text-slate-900 dark:text-slate-100">Шаг 3. Blank subtraction</p>
                    <p class="mt-2 text-sm leading-7 text-slate-600 dark:text-slate-400">Подтверждённые sample-пики сравниваются с blank-пиками той же полярности. Для сопоставленной пары вычисляется S/B ratio.</p>
                    <!-- Карточки бинарной классификации -->
                    <div class="mt-3 grid gap-3 sm:grid-cols-2">
                        <div class="rounded-xl border border-red-200 bg-red-50 p-3 text-xs text-red-800 dark:border-red-800 dark:bg-red-950 dark:text-red-300"><strong>Artifact</strong>: найдено совпадение с blank и S/B ниже порога.</div>
                        <div class="rounded-xl border border-green-200 bg-green-50 p-3 text-xs text-green-800 dark:border-green-800 dark:bg-green-950 dark:text-green-300"><strong>Real Compound</strong>: совпадение с blank отсутствует или S/B достаточно высок.</div>
                    </div>
                </div>

                <!-- Шаг 4: Сводная статистика и формирование аудиторского следа -->
                <div class="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                    <p class="text-sm font-semibold text-slate-900 dark:text-slate-100">Шаг 4. Summary и аудиторский след</p>
                    <p class="mt-2 text-sm leading-7 text-slate-600 dark:text-slate-400">Приложение считает summary-метрики по SampleType / Polarity и сохраняет логику решения в <code class="rounded bg-slate-100 px-1 font-mono dark:bg-slate-700">Why</code> для проверки и аудита.</p>
                </div>
            </div>
        </section>

        <!-- ----------------------------------------------------------------->
        <!-- Раздел 5: Выходные поля                                          -->
        <!-- ----------------------------------------------------------------->
        <!--
          Отображает массив `outputFields` как вертикальный список карточек.
          Каждая карточка показывает имя поля (моноширинный синий) и его описание.
          Это колонки, которые пользователь увидит в финальном screened Excel-файле.
        -->
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

        <!-- ----------------------------------------------------------------->
        <!-- Раздел 6: Параметры толерантности                                -->
        <!-- ----------------------------------------------------------------->
        <!--
          Отображает массив `params` как четырёхколоночную HTML-таблицу.
          Индексы кортежа отображаются в колонки:
            row[0] → имя параметра (моноширинный, синий)
            row[1] → значение по умолчанию
            row[2] → единица измерения
            row[3] → этап конвейера, использующий параметр
        -->
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

        <!-- ----------------------------------------------------------------->
        <!-- Раздел 7: Глоссарий                                              -->
        <!-- ----------------------------------------------------------------->
        <!--
          Отображает массив `glossary` как двухколоночную сетку карточек.
          Каждая карточка показывает термин (моноширинный синий) и его определение.
          Обеспечивает быструю справку по терминам предметной области,
          используемым в документации методологии.
        -->
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

        <!-- ----------------------------------------------------------------->
        <!-- Раздел 8: Ссылки                                                 -->
        <!-- ----------------------------------------------------------------->
        <!--
          Отображает массив `refs` как вертикальный список исходящих ссылок.
          Каждая ссылка открывается в новой вкладке (target="_blank") с
          атрибутами безопасности (rel="noopener noreferrer") для предотвращения
          tab-napping-атак.
        -->
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
