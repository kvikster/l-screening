<!--
  @file MethodologyVisualizerRu.svelte
  @description
  Русскоязычная интерактивная визуализация конвейера LC-MS скрининга.
  Содержит три раздела:
    1. Блок-схема — горизонтальные карточки этапов, соединённые стрелками
    2. Интерактивный обход — пошаговый навигатор с деталями вход/действие/выход
    3. Карточки логики решений — правила «условие → результат» для Artifact / Real Compound

  Использует Svelte 5 runes ($state, $derived) и нативные классы Tailwind dark-mode.
  Без внешних зависимостей (Mermaid, D3 и т.д.) — чистый HTML/SVG/CSS.
-->
<script lang="ts">
    // ---------------------------------------------------------------------------
    // Шаги блок-схемы — верхняя горизонтальная диаграмма
    // ---------------------------------------------------------------------------
    const flowSteps = [
        {
            id: "excel",
            title: "Ввод Excel",
            short: "Рабочая книга с пиками LC-MS",
            tone: "slate"
        },
        {
            id: "validate",
            title: "Валидация строк",
            short: "Удаление неполных строк",
            tone: "amber"
        },
        {
            id: "classify",
            title: "Назначение ролей",
            short: "Sample / Blank / Replicate",
            tone: "violet"
        },
        {
            id: "replicates",
            title: "Подтверждение в репликатах",
            short: "Проверка совпадения RT и m/z",
            tone: "blue"
        },
        {
            id: "blank",
            title: "Сравнение с blank",
            short: "Поиск фонового сигнала",
            tone: "cyan"
        },
        {
            id: "decision",
            title: "Итоговое решение",
            short: "Artifact или Real Compound",
            tone: "green"
        },
        {
            id: "output",
            title: "Результат для аудита",
            short: "Метрики + журнал Why",
            tone: "rose"
        }
    ];

    // ---------------------------------------------------------------------------
    // Шаги интерактивного обхода — содержимое пошагового навигатора
    // ---------------------------------------------------------------------------
    const walkthroughSteps = [
        {
            title: "1. Входные данные Excel",
            summary:
                "Приложение загружает рабочую книгу Excel и автоматически выбирает лист, наиболее соответствующий требуемым колонкам.",
            input: ["Рабочая книга", "Один или несколько листов"],
            action: ["Определить требуемые заголовки", "Выбрать наиболее подходящий лист"],
            output: ["Структурированные строки пиков LC-MS"],
            highlight: "Пользователю не нужно вручную выбирать правильный лист в типичном случае."
        },
        {
            title: "2. Валидация строк",
            summary:
                "Строки без RT, Base Peak или Area удаляются, поскольку они не могут быть надёжно проверены.",
            input: ["Строки пиков из Excel"],
            action: ["Проверить RT", "Проверить Base Peak", "Проверить Area"],
            output: ["Остаются только пригодные строки"],
            highlight: "Этот шаг удаляет неполные данные до принятия любого научного решения."
        },
        {
            title: "3. Назначение ролей строкам",
            summary:
                "Каждая строка классифицируется как sample или blank. Цветовые метки оператора имеют приоритет над эвристикой по имени файла.",
            input: ["Валидные строки", "Цвета ячеек Excel", "Имена файлов"],
            action: ["Считать метку оператора", "При отсутствии — использовать логику имени файла"],
            output: ["Назначение Sample / Blank / Replicate"],
            highlight: "Явные метки оператора перекрывают предположения, основанные на именах файлов."
        },
        {
            title: "4. Подтверждение пиков в репликатах",
            summary:
                "Сигнал становится более достоверным, если он встречается более чем в одном репликате в пределах настроенных допусков RT и m/z.",
            input: ["Строки, сгруппированные по полярности и репликату"],
            action: ["Сравнить RT", "Сравнить m/z", "Применить правило «один пик на корзину»"],
            output: ["Подтверждённый кластер репликатов"],
            highlight: "Система проверяет, что одно и то же соединение обнаружено повторно, а не только один раз."
        },
        {
            title: "5. Сравнение с blank",
            summary:
                "Каждый подтверждённый sample-пик сравнивается с blank-пиками той же полярности для выявления фонового сигнала или загрязнения.",
            input: ["Подтверждённый sample-кластер", "Blank-пики"],
            action: ["Найти совпадение с blank", "Вычислить отношение signal-to-blank"],
            output: ["Связанный с blank или чистый сигнал"],
            highlight: "Это основная защита от ложноположительных результатов, вызванных фоновым сигналом."
        },
        {
            title: "6. Итоговое решение",
            summary:
                "Если blank-сигнал слишком велик по сравнению с sample-сигналом, результат помечается как Artifact. В противном случае — Real Compound.",
            input: ["Отношение Signal-to-Blank", "Порог решения"],
            action: ["Сравнить отношение с порогом"],
            output: ["Artifact или Real Compound"],
            highlight: "Это бизнес-результат, который больше всего интересует нетехнических пользователей."
        },
        {
            title: "7. Результат для аудита",
            summary:
                "Приложение вычисляет итоговые метрики и сохраняет объяснение решения в поле Why для проверки и аудита.",
            input: ["Подтверждённое решение"],
            action: ["Вычислить средние", "Вычислить CV%", "Записать журнал Why"],
            output: ["Итоговая таблица результатов"],
            highlight: "Каждый важный результат можно объяснить позже."
        }
    ];

    // ---------------------------------------------------------------------------
    // Карточки логики решений — правила «условие → результат»
    // ---------------------------------------------------------------------------
    const decisionCards = [
        {
            title: "Отсутствуют обязательные значения пика",
            condition: "RT, Base Peak или Area отсутствует",
            result: "Строка отбрасывается",
            kind: "bad"
        },
        {
            title: "Метка оператора присутствует",
            condition: "Цвет ячейки Excel совпадает с известной меткой оператора",
            result: "Используется явная роль из метки",
            kind: "info"
        },
        {
            title: "Метка оператора отсутствует",
            condition: "Нет известной цветовой метки",
            result: "Используется эвристика по имени файла",
            kind: "info"
        },
        {
            title: "Подтверждение в репликатах не прошло",
            condition: "RT или m/z вне допуска репликатов",
            result: "Пик не подтверждён",
            kind: "bad"
        },
        {
            title: "Совпадение с blank найдено и отношение слишком низкое",
            condition: "Совпадение с blank существует и S/B ниже порога",
            result: "Artifact",
            kind: "bad"
        },
        {
            title: "Проблемы с blank нет",
            condition: "Нет совпадения с blank или допустимое отношение S/B",
            result: "Real Compound",
            kind: "good"
        }
    ];

    // ---------------------------------------------------------------------------
    // Типичный пример — конкретный обход одного пика
    // ---------------------------------------------------------------------------
    const typicalExample = {
        samplePeak: {
            rt: 2.35,
            mz: 195.08,
            area: 1250000,
            polarity: "positive",
            file: "1_pos.d"
        },
        replicateMatch: {
            rt: 2.36,
            mz: 195.09,
            area: 1180000,
            polarity: "positive",
            file: "2_pos.d"
        },
        blankPeak: {
            rt: 2.34,
            mz: 195.07,
            area: 85000,
            polarity: "positive",
            file: "blank_pos.d"
        },
        results: {
            rtMean: 2.355,
            mzMean: 195.085,
            areaMean: 1215000,
            cvPct: 4.1,
            signalToBlank: 14.3,
            status: "Real Compound" as const
        }
    };

    // ---------------------------------------------------------------------------
    // Состояние компонента (Svelte 5 runes)
    // ---------------------------------------------------------------------------
    let current = $state(0);
    let step = $derived(walkthroughSteps[current]);

    // ---------------------------------------------------------------------------
    // Навигация
    // ---------------------------------------------------------------------------
    function next() {
        if (current < walkthroughSteps.length - 1) current += 1;
    }

    function prev() {
        if (current > 0) current -= 1;
    }

    function jumpTo(index: number) {
        current = index;
    }

    // ---------------------------------------------------------------------------
    // Стили
    // ---------------------------------------------------------------------------
    function toneClasses(tone: string) {
        switch (tone) {
            case "amber":
                return "border-amber-200 bg-amber-50 text-amber-900 dark:border-amber-900 dark:bg-amber-950 dark:text-amber-200";
            case "violet":
                return "border-violet-200 bg-violet-50 text-violet-900 dark:border-violet-900 dark:bg-violet-950 dark:text-violet-200";
            case "blue":
                return "border-blue-200 bg-blue-50 text-blue-900 dark:border-blue-900 dark:bg-blue-950 dark:text-blue-200";
            case "cyan":
                return "border-cyan-200 bg-cyan-50 text-cyan-900 dark:border-cyan-900 dark:bg-cyan-950 dark:text-cyan-200";
            case "green":
                return "border-green-200 bg-green-50 text-green-900 dark:border-green-900 dark:bg-green-950 dark:text-green-200";
            case "rose":
                return "border-rose-200 bg-rose-50 text-rose-900 dark:border-rose-900 dark:bg-rose-950 dark:text-rose-200";
            default:
                return "border-slate-200 bg-slate-50 text-slate-900 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-200";
        }
    }

    function cardClasses(kind: string) {
        switch (kind) {
            case "good":
                return "border-green-200 bg-green-50 dark:border-green-900 dark:bg-green-950";
            case "bad":
                return "border-red-200 bg-red-50 dark:border-red-900 dark:bg-red-950";
            default:
                return "border-blue-200 bg-blue-50 dark:border-blue-900 dark:bg-blue-950";
        }
    }

    function badgeClasses(kind: string) {
        switch (kind) {
            case "good":
                return "bg-green-100 text-green-800 dark:bg-green-900/50 dark:text-green-200";
            case "bad":
                return "bg-red-100 text-red-800 dark:bg-red-900/50 dark:text-red-200";
            default:
                return "bg-blue-100 text-blue-800 dark:bg-blue-900/50 dark:text-blue-200";
        }
    }
</script>

<!-- ======================================================================= -->
<!-- В одном предложении                                                     -->
<!-- ======================================================================= -->
<section class="mb-8 rounded-3xl border border-blue-200 bg-gradient-to-r from-blue-50 to-indigo-50 p-6 shadow-sm dark:border-blue-900 dark:from-blue-950 dark:to-indigo-950">
    <div class="flex items-start gap-4">
        <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-2xl bg-blue-100 text-blue-600 dark:bg-blue-900/60 dark:text-blue-300">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <circle cx="12" cy="12" r="10" />
                <path d="M12 16v-4" />
                <path d="M12 8h.01" />
            </svg>
        </div>
        <div>
            <p class="text-xs font-semibold uppercase tracking-[0.2em] text-blue-500 dark:text-blue-400">
                В одном предложении
            </p>
            <p class="mt-2 text-base leading-8 text-blue-900 dark:text-blue-100">
                Система считывает исходные пики LC-MS из Excel, валидирует их, подтверждает каждый сигнал в репликатах,
                сравнивает с blank и помечает результат как <strong>Real Compound</strong> или
                <strong>Artifact</strong> — с полным аудиторским следом, объясняющим почему.
            </p>
        </div>
    </div>
</section>

<!-- ======================================================================= -->
<!-- Блок-схема — горизонтальные карточки этапов, соединённые стрелками      -->
<!-- ======================================================================= -->
<section class="mb-10 rounded-3xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
    <div class="mb-6">
        <p class="text-xs font-semibold uppercase tracking-[0.2em] text-slate-400 dark:text-slate-500">
            Простой визуальный обзор
        </p>
        <h2 class="mt-2 text-2xl font-semibold text-slate-900 dark:text-slate-50">
            Как работает скрининг
        </h2>
        <p class="mt-3 max-w-3xl text-sm leading-7 text-slate-600 dark:text-slate-400">
            Система берёт исходные пики LC-MS из Excel, удаляет неполные строки, подтверждает сигналы в репликатах,
            проверяет их по blank и затем выдаёт итоговый результат с объяснением.
        </p>
    </div>

    <div class="overflow-x-auto pb-2">
        <div class="flex min-w-[980px] items-stretch gap-3">
            {#each flowSteps as item, i}
                <div class="flex w-[170px] shrink-0 flex-col rounded-2xl border p-4 {toneClasses(item.tone)}">
                    <div class="mb-3 flex items-center justify-between">
                        <span class="text-xs font-semibold uppercase tracking-wide opacity-70">
                            Шаг {i + 1}
                        </span>
                        <span class="rounded-full bg-white/70 px-2 py-0.5 text-[10px] font-semibold dark:bg-black/20">
                            {item.id}
                        </span>
                    </div>

                    <p class="text-sm font-semibold">{item.title}</p>
                    <p class="mt-2 text-xs leading-6 opacity-80">{item.short}</p>
                </div>

                {#if i < flowSteps.length - 1}
                    <div class="flex shrink-0 items-center justify-center px-1 text-slate-300 dark:text-slate-600">
                        <svg width="32" height="24" viewBox="0 0 32 24" fill="none" aria-hidden="true">
                            <path d="M2 12H26" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" />
                            <path d="M20 6L26 12L20 18" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" />
                        </svg>
                    </div>
                {/if}
            {/each}
        </div>
    </div>
</section>

<!-- ======================================================================= -->
<!-- Интерактивный обход — пошаговый навигатор                               -->
<!-- ======================================================================= -->
<section class="mb-10">
    <div class="mb-4">
        <p class="text-xs font-semibold uppercase tracking-[0.2em] text-slate-400 dark:text-slate-500">
            Интерактивный обход
        </p>
        <h2 class="mt-2 text-2xl font-semibold text-slate-900 dark:text-slate-50">
            Пошаговое объяснение
        </h2>
    </div>

    <!-- Прогресс-бар -->
    <div class="mb-6 h-2 overflow-hidden rounded-full bg-slate-200 dark:bg-slate-800">
        <div
            class="h-full rounded-full bg-blue-600 transition-all duration-300"
            style="width:{((current + 1) / walkthroughSteps.length) * 100}%"
        ></div>
    </div>

    <div class="grid gap-6 lg:grid-cols-[1.15fr_0.85fr]">
        <!-- Основная панель содержимого -->
        <section class="rounded-3xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
            <div class="flex flex-wrap items-center gap-3">
                <span class="rounded-full bg-blue-100 px-3 py-1 text-xs font-semibold uppercase tracking-wide text-blue-800 dark:bg-blue-950 dark:text-blue-200">
                    Текущий шаг
                </span>
                <p class="text-sm font-semibold text-slate-500 dark:text-slate-400">
                    {step.title}
                </p>
            </div>

            <p class="mt-5 text-base leading-8 text-slate-700 dark:text-slate-300">
                {step.summary}
            </p>

            <div class="mt-6 rounded-2xl border border-slate-200 bg-slate-50 p-4 dark:border-slate-700 dark:bg-slate-900">
                <p class="text-xs font-semibold uppercase tracking-[0.15em] text-slate-400 dark:text-slate-500">
                    Почему это важно
                </p>
                <p class="mt-2 text-sm leading-7 text-slate-600 dark:text-slate-400">
                    {step.highlight}
                </p>
            </div>

            <div class="mt-6 grid gap-4 md:grid-cols-3">
                <!-- Колонка «Вход» -->
                <div class="rounded-2xl border border-slate-200 bg-slate-50 p-4 dark:border-slate-700 dark:bg-slate-900">
                    <p class="text-xs font-semibold uppercase tracking-wide text-slate-400 dark:text-slate-500">
                        Вход
                    </p>
                    <div class="mt-3 space-y-2">
                        {#each step.input as item}
                            <div class="rounded-xl bg-white px-3 py-2 text-sm text-slate-700 dark:bg-slate-800 dark:text-slate-300">
                                {item}
                            </div>
                        {/each}
                    </div>
                </div>

                <!-- Колонка «Действие» -->
                <div class="rounded-2xl border border-slate-200 bg-slate-50 p-4 dark:border-slate-700 dark:bg-slate-900">
                    <p class="text-xs font-semibold uppercase tracking-wide text-slate-400 dark:text-slate-500">
                        Что делает система
                    </p>
                    <div class="mt-3 space-y-2">
                        {#each step.action as item}
                            <div class="rounded-xl bg-white px-3 py-2 text-sm text-slate-700 dark:bg-slate-800 dark:text-slate-300">
                                {item}
                            </div>
                        {/each}
                    </div>
                </div>

                <!-- Колонка «Выход» -->
                <div class="rounded-2xl border border-slate-200 bg-slate-50 p-4 dark:border-slate-700 dark:bg-slate-900">
                    <p class="text-xs font-semibold uppercase tracking-wide text-slate-400 dark:text-slate-500">
                        Выход
                    </p>
                    <div class="mt-3 space-y-2">
                        {#each step.output as item}
                            <div class="rounded-xl bg-white px-3 py-2 text-sm text-slate-700 dark:bg-slate-800 dark:text-slate-300">
                                {item}
                            </div>
                        {/each}
                    </div>
                </div>
            </div>
        </section>

        <!-- Боковая навигация -->
        <aside class="rounded-3xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
            <p class="text-xs font-semibold uppercase tracking-[0.2em] text-slate-400 dark:text-slate-500">
                Навигация по шагам
            </p>

            <div class="mt-4 space-y-3">
                {#each walkthroughSteps as s, i}
                    <button
                        class="w-full rounded-2xl border px-4 py-3 text-left transition {i === current
                            ? 'border-blue-500 bg-blue-50 text-blue-900 dark:border-blue-500 dark:bg-blue-950 dark:text-blue-200'
                            : 'border-slate-200 bg-slate-50 text-slate-600 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-400'}"
                        onclick={() => jumpTo(i)}
                    >
                        <span class="block text-sm font-semibold">{s.title}</span>
                    </button>
                {/each}
            </div>

            <div class="mt-6 flex gap-3">
                <button
                    onclick={prev}
                    disabled={current === 0}
                    class="rounded-2xl border border-slate-300 px-4 py-2 text-sm text-slate-700 disabled:opacity-40 dark:border-slate-600 dark:text-slate-300"
                >
                    Назад
                </button>

                <button
                    onclick={next}
                    disabled={current === walkthroughSteps.length - 1}
                    class="rounded-2xl bg-blue-600 px-4 py-2 text-sm text-white disabled:opacity-40"
                >
                    Далее
                </button>
            </div>
        </aside>
    </div>
</section>

<!-- ======================================================================= -->
<!-- Карточки логики решений                                                 -->
<!-- ======================================================================= -->
<section class="mb-10 rounded-3xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
    <div class="mb-5">
        <p class="text-xs font-semibold uppercase tracking-[0.2em] text-slate-400 dark:text-slate-500">
            Логика решений
        </p>
        <h2 class="mt-2 text-2xl font-semibold text-slate-900 dark:text-slate-50">
            Почему результат становится Artifact или Real Compound
        </h2>
    </div>

    <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
        {#each decisionCards as card}
            <div class="rounded-2xl border p-4 {cardClasses(card.kind)}">
                <div class="flex items-center justify-between gap-3">
                    <p class="text-sm font-semibold text-slate-900 dark:text-slate-100">
                        {card.title}
                    </p>
                    <span class="rounded-full px-2.5 py-1 text-[10px] font-semibold uppercase tracking-wide {badgeClasses(card.kind)}">
                        {card.kind === "good" ? "хороший" : card.kind === "bad" ? "плохой" : "инфо"}
                    </span>
                </div>

                <div class="mt-4 space-y-3">
                    <div>
                        <p class="text-xs font-semibold uppercase tracking-wide text-slate-400 dark:text-slate-500">
                            Условие
                        </p>
                        <p class="mt-1 text-sm leading-7 text-slate-700 dark:text-slate-300">
                            {card.condition}
                        </p>
                    </div>

                    <div>
                        <p class="text-xs font-semibold uppercase tracking-wide text-slate-400 dark:text-slate-500">
                            Результат
                        </p>
                        <p class="mt-1 text-sm font-semibold text-slate-900 dark:text-slate-100">
                            {card.result}
                        </p>
                    </div>
                </div>
            </div>
        {/each}
    </div>
</section>

<!-- ======================================================================= -->
<!-- Типичный пример — конкретный обход одного пика                          -->
<!-- ======================================================================= -->
<section class="mb-10 rounded-3xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
    <div class="mb-5">
        <p class="text-xs font-semibold uppercase tracking-[0.2em] text-slate-400 dark:text-slate-500">
            Типичный пример
        </p>
        <h2 class="mt-2 text-2xl font-semibold text-slate-900 dark:text-slate-50">
            Один пик — от исходного сигнала до итогового вердикта
        </h2>
        <p class="mt-2 max-w-3xl text-sm leading-7 text-slate-600 dark:text-slate-400">
            Ниже приведён конкретный пример того, как один пик LC-MS проходит через конвейер —
            от начального измерения образца, через подтверждение в репликатах и сравнение с blank,
            до окончательной классификации.
        </p>
    </div>

    <div class="grid gap-4 lg:grid-cols-3">
        <!-- Пик образца -->
        <div class="rounded-2xl border border-violet-200 bg-violet-50 p-5 dark:border-violet-900 dark:bg-violet-950">
            <div class="mb-3 flex items-center gap-2">
                <span class="flex h-7 w-7 items-center justify-center rounded-full bg-violet-200 text-xs font-bold text-violet-800 dark:bg-violet-800 dark:text-violet-100">S</span>
                <p class="text-sm font-semibold text-violet-900 dark:text-violet-200">Пик образца</p>
            </div>
            <div class="space-y-2 text-sm">
                <div class="flex justify-between rounded-xl bg-white/70 px-3 py-2 dark:bg-black/20">
                    <span class="text-slate-500 dark:text-slate-400">RT</span>
                    <span class="font-mono font-semibold text-slate-800 dark:text-slate-200">{typicalExample.samplePeak.rt} мин</span>
                </div>
                <div class="flex justify-between rounded-xl bg-white/70 px-3 py-2 dark:bg-black/20">
                    <span class="text-slate-500 dark:text-slate-400">m/z</span>
                    <span class="font-mono font-semibold text-slate-800 dark:text-slate-200">{typicalExample.samplePeak.mz}</span>
                </div>
                <div class="flex justify-between rounded-xl bg-white/70 px-3 py-2 dark:bg-black/20">
                    <span class="text-slate-500 dark:text-slate-400">Площадь</span>
                    <span class="font-mono font-semibold text-slate-800 dark:text-slate-200">{typicalExample.samplePeak.area.toLocaleString()}</span>
                </div>
                <div class="flex justify-between rounded-xl bg-white/70 px-3 py-2 dark:bg-black/20">
                    <span class="text-slate-500 dark:text-slate-400">Файл</span>
                    <span class="font-mono text-xs text-slate-600 dark:text-slate-300">{typicalExample.samplePeak.file}</span>
                </div>
            </div>
        </div>

        <!-- Совпадение в репликате -->
        <div class="rounded-2xl border border-blue-200 bg-blue-50 p-5 dark:border-blue-900 dark:bg-blue-950">
            <div class="mb-3 flex items-center gap-2">
                <span class="flex h-7 w-7 items-center justify-center rounded-full bg-blue-200 text-xs font-bold text-blue-800 dark:bg-blue-800 dark:text-blue-100">R</span>
                <p class="text-sm font-semibold text-blue-900 dark:text-blue-200">Совпадение в репликате</p>
            </div>
            <div class="space-y-2 text-sm">
                <div class="flex justify-between rounded-xl bg-white/70 px-3 py-2 dark:bg-black/20">
                    <span class="text-slate-500 dark:text-slate-400">RT</span>
                    <span class="font-mono font-semibold text-slate-800 dark:text-slate-200">{typicalExample.replicateMatch.rt} мин</span>
                </div>
                <div class="flex justify-between rounded-xl bg-white/70 px-3 py-2 dark:bg-black/20">
                    <span class="text-slate-500 dark:text-slate-400">m/z</span>
                    <span class="font-mono font-semibold text-slate-800 dark:text-slate-200">{typicalExample.replicateMatch.mz}</span>
                </div>
                <div class="flex justify-between rounded-xl bg-white/70 px-3 py-2 dark:bg-black/20">
                    <span class="text-slate-500 dark:text-slate-400">Площадь</span>
                    <span class="font-mono font-semibold text-slate-800 dark:text-slate-200">{typicalExample.replicateMatch.area.toLocaleString()}</span>
                </div>
                <div class="flex justify-between rounded-xl bg-white/70 px-3 py-2 dark:bg-black/20">
                    <span class="text-slate-500 dark:text-slate-400">Файл</span>
                    <span class="font-mono text-xs text-slate-600 dark:text-slate-300">{typicalExample.replicateMatch.file}</span>
                </div>
            </div>
            <div class="mt-3 rounded-xl border border-blue-200 bg-blue-100/60 px-3 py-2 text-xs text-blue-700 dark:border-blue-800 dark:bg-blue-900/40 dark:text-blue-300">
                ΔRT = 0.01 мин ✓ &nbsp;·&nbsp; Δm/z = 0.01 Da ✓
            </div>
        </div>

        <!-- Пик blank -->
        <div class="rounded-2xl border border-cyan-200 bg-cyan-50 p-5 dark:border-cyan-900 dark:bg-cyan-950">
            <div class="mb-3 flex items-center gap-2">
                <span class="flex h-7 w-7 items-center justify-center rounded-full bg-cyan-200 text-xs font-bold text-cyan-800 dark:bg-cyan-800 dark:text-cyan-100">B</span>
                <p class="text-sm font-semibold text-cyan-900 dark:text-cyan-200">Пик blank</p>
            </div>
            <div class="space-y-2 text-sm">
                <div class="flex justify-between rounded-xl bg-white/70 px-3 py-2 dark:bg-black/20">
                    <span class="text-slate-500 dark:text-slate-400">RT</span>
                    <span class="font-mono font-semibold text-slate-800 dark:text-slate-200">{typicalExample.blankPeak.rt} мин</span>
                </div>
                <div class="flex justify-between rounded-xl bg-white/70 px-3 py-2 dark:bg-black/20">
                    <span class="text-slate-500 dark:text-slate-400">m/z</span>
                    <span class="font-mono font-semibold text-slate-800 dark:text-slate-200">{typicalExample.blankPeak.mz}</span>
                </div>
                <div class="flex justify-between rounded-xl bg-white/70 px-3 py-2 dark:bg-black/20">
                    <span class="text-slate-500 dark:text-slate-400">Площадь</span>
                    <span class="font-mono font-semibold text-slate-800 dark:text-slate-200">{typicalExample.blankPeak.area.toLocaleString()}</span>
                </div>
                <div class="flex justify-between rounded-xl bg-white/70 px-3 py-2 dark:bg-black/20">
                    <span class="text-slate-500 dark:text-slate-400">Файл</span>
                    <span class="font-mono text-xs text-slate-600 dark:text-slate-300">{typicalExample.blankPeak.file}</span>
                </div>
            </div>
            <div class="mt-3 rounded-xl border border-cyan-200 bg-cyan-100/60 px-3 py-2 text-xs text-cyan-700 dark:border-cyan-800 dark:bg-cyan-900/40 dark:text-cyan-300">
                Совпадение с blank найдено · S/B = {typicalExample.results.signalToBlank} ≥ 3.0 ✓
            </div>
        </div>
    </div>

    <!-- Итоговый вердикт -->
    <div class="mt-5 rounded-2xl border border-green-200 bg-green-50 p-5 dark:border-green-900 dark:bg-green-950">
        <div class="flex flex-wrap items-center gap-4">
            <span class="rounded-full bg-green-100 px-3 py-1 text-xs font-bold uppercase tracking-wide text-green-800 dark:bg-green-900/60 dark:text-green-200">
                {typicalExample.results.status}
            </span>
            <div class="flex flex-wrap gap-4 text-sm text-green-800 dark:text-green-300">
                <span>RT<sub>ср</sub> = {typicalExample.results.rtMean}</span>
                <span>m/z<sub>ср</sub> = {typicalExample.results.mzMean}</span>
                <span>Площадь<sub>ср</sub> = {typicalExample.results.areaMean.toLocaleString()}</span>
                <span>CV% = {typicalExample.results.cvPct}</span>
                <span>S/B = {typicalExample.results.signalToBlank}</span>
            </div>
        </div>
    </div>
</section>
