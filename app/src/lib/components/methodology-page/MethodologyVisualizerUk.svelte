<!--
  @file MethodologyVisualizerUk.svelte
  @description
  Україномовна інтерактивна візуалізація конвеєра LC-MS скринінгу.
  Містить три розділи:
    1. Блок-схема — горизонтальні картки кроків, з'єднані стрілками
    2. Інтерактивний обхід — покроковий навігатор з деталями вхід/дія/вихід
    3. Картки логіки рішень — правила «умова → результат» для Artifact / Real Compound

  Використовує Svelte 5 runes ($state, $derived) та нативні класи Tailwind dark-mode.
  Без зовнішніх залежностей (Mermaid, D3 тощо) — чистий HTML/SVG/CSS.
-->
<script lang="ts">
    // ---------------------------------------------------------------------------
    // Кроки блок-схеми — верхня горизонтальна діаграма
    // ---------------------------------------------------------------------------
    const flowSteps = [
        {
            id: "excel",
            title: "Ввід Excel",
            short: "Робоча книга з піками LC-MS",
            tone: "slate"
        },
        {
            id: "validate",
            title: "Валідація рядків",
            short: "Видалення неповних рядків",
            tone: "amber"
        },
        {
            id: "classify",
            title: "Призначення ролей",
            short: "Sample / Blank / Replicate",
            tone: "violet"
        },
        {
            id: "replicates",
            title: "Підтвердження в реплікатах",
            short: "Перевірка збігу RT та m/z",
            tone: "blue"
        },
        {
            id: "blank",
            title: "Порівняння з blank",
            short: "Пошук фонового сигналу",
            tone: "cyan"
        },
        {
            id: "decision",
            title: "Підсумкове рішення",
            short: "Artifact або Real Compound",
            tone: "green"
        },
        {
            id: "output",
            title: "Результат для аудиту",
            short: "Метрики + журнал Why",
            tone: "rose"
        }
    ];

    // ---------------------------------------------------------------------------
    // Кроки інтерактивного обходу — вміст покрокового навігатора
    // ---------------------------------------------------------------------------
    const walkthroughSteps = [
        {
            title: "1. Вхідні дані Excel",
            summary:
                "Додаток завантажує робочу книгу Excel і автоматично обирає аркуш, що найкраще відповідає необхідним колонкам.",
            input: ["Робоча книга", "Один або кілька аркушів"],
            action: ["Визначити необхідні заголовки", "Обрати найбільш підходящий аркуш"],
            output: ["Структуровані рядки піків LC-MS"],
            highlight: "Користувачу не потрібно вручну вибирати правильний аркуш у типовому випадку."
        },
        {
            title: "2. Валідація рядків",
            summary:
                "Рядки без RT, Base Peak або Area видаляються, оскільки вони не можуть бути надійно перевірені.",
            input: ["Рядки піків з Excel"],
            action: ["Перевірити RT", "Перевірити Base Peak", "Перевірити Area"],
            output: ["Залишаються лише придатні рядки"],
            highlight: "Цей крок видаляє неповні дані до прийняття будь-якого наукового рішення."
        },
        {
            title: "3. Призначення ролей рядкам",
            summary:
                "Кожен рядок класифікується як sample або blank. Кольорові мітки оператора мають пріоритет над евристикою за іменем файлу.",
            input: ["Валідні рядки", "Кольори клітинок Excel", "Імена файлів"],
            action: ["Зчитати мітку оператора", "За відсутності — використати логіку імені файлу"],
            output: ["Призначення Sample / Blank / Replicate"],
            highlight: "Явні мітки оператора перекривають припущення, засновані на іменах файлів."
        },
        {
            title: "4. Підтвердження піків у реплікатах",
            summary:
                "Сигнал стає більш достовірним, якщо він трапляється більш ніж в одному реплікаті в межах налаштованих допусків RT та m/z.",
            input: ["Рядки, згруповані за полярністю та реплікатом"],
            action: ["Порівняти RT", "Порівняти m/z", "Застосувати правило «один пік на кошик»"],
            output: ["Підтверджений кластер реплікатів"],
            highlight: "Система перевіряє, що одну й ту саму сполуку виявлено повторно, а не лише один раз."
        },
        {
            title: "5. Порівняння з blank",
            summary:
                "Кожен підтверджений sample-пік порівнюється з blank-піками тієї ж полярності для виявлення фонового сигналу або забруднення.",
            input: ["Підтверджений sample-кластер", "Blank-піки"],
            action: ["Знайти збіг із blank", "Обчислити відношення signal-to-blank"],
            output: ["Пов'язаний із blank або чистий сигнал"],
            highlight: "Це основний захист від хибнопозитивних результатів, спричинених фоновим сигналом."
        },
        {
            title: "6. Підсумкове рішення",
            summary:
                "Якщо blank-сигнал занадто великий порівняно з sample-сигналом, результат позначається як Artifact. Інакше — Real Compound.",
            input: ["Відношення Signal-to-Blank", "Поріг рішення"],
            action: ["Порівняти відношення з порогом"],
            output: ["Artifact або Real Compound"],
            highlight: "Це бізнес-результат, який найбільше цікавить нетехнічних користувачів."
        },
        {
            title: "7. Результат для аудиту",
            summary:
                "Додаток обчислює підсумкові метрики та зберігає пояснення рішення у полі Why для перевірки й аудиту.",
            input: ["Підтверджене рішення"],
            action: ["Обчислити середні", "Обчислити CV%", "Записати журнал Why"],
            output: ["Підсумкова таблиця результатів"],
            highlight: "Кожен важливий результат можна пояснити згодом."
        }
    ];

    // ---------------------------------------------------------------------------
    // Картки логіки рішень — правила «умова → результат»
    // ---------------------------------------------------------------------------
    const decisionCards = [
        {
            title: "Відсутні обов'язкові значення піку",
            condition: "RT, Base Peak або Area відсутній",
            result: "Рядок відкидається",
            kind: "bad"
        },
        {
            title: "Мітка оператора присутня",
            condition: "Колір клітинки Excel збігається з відомою міткою оператора",
            result: "Використовується явна роль з мітки",
            kind: "info"
        },
        {
            title: "Мітка оператора відсутня",
            condition: "Немає відомої кольорової мітки",
            result: "Використовується евристика за іменем файлу",
            kind: "info"
        },
        {
            title: "Підтвердження в реплікатах не пройшло",
            condition: "RT або m/z поза допусками реплікатів",
            result: "Пік не підтверджений",
            kind: "bad"
        },
        {
            title: "Збіг із blank знайдено і відношення занадто низьке",
            condition: "Збіг із blank існує і S/B нижче порогу",
            result: "Artifact",
            kind: "bad"
        },
        {
            title: "Проблем із blank немає",
            condition: "Немає збігу з blank або прийнятне відношення S/B",
            result: "Real Compound",
            kind: "good"
        }
    ];

    // ---------------------------------------------------------------------------
    // Типовий приклад — конкретний обхід одного піку
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
    // Стан компонента (Svelte 5 runes)
    // ---------------------------------------------------------------------------
    let current = $state(0);
    let step = $derived(walkthroughSteps[current]);

    // ---------------------------------------------------------------------------
    // Навігація
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
    // Стилі
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
<!-- В одному реченні                                                        -->
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
                В одному реченні
            </p>
            <p class="mt-2 text-base leading-8 text-blue-900 dark:text-blue-100">
                Система зчитує вихідні піки LC-MS з Excel, валідує їх, підтверджує кожен сигнал у реплікатах,
                порівнює з blank і позначає результат як <strong>Real Compound</strong> або
                <strong>Artifact</strong> — із повним аудиторським слідом, що пояснює чому.
            </p>
        </div>
    </div>
</section>

<!-- ======================================================================= -->
<!-- Блок-схема — горизонтальні картки кроків, з'єднані стрілками           -->
<!-- ======================================================================= -->
<section class="mb-10 rounded-3xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
    <div class="mb-6">
        <p class="text-xs font-semibold uppercase tracking-[0.2em] text-slate-400 dark:text-slate-500">
            Простой візуальний огляд
        </p>
        <h2 class="mt-2 text-2xl font-semibold text-slate-900 dark:text-slate-50">
            Як працює скринінг
        </h2>
        <p class="mt-3 max-w-3xl text-sm leading-7 text-slate-600 dark:text-slate-400">
            Система бере вихідні піки LC-MS з Excel, видаляє неповні рядки, підтверджує сигнали у реплікатах,
            перевіряє їх за blank і потім видає підсумковий результат з поясненням.
        </p>
    </div>

    <div class="overflow-x-auto pb-2">
        <div class="flex min-w-[980px] items-stretch gap-3">
            {#each flowSteps as item, i}
                <div class="flex w-[170px] shrink-0 flex-col rounded-2xl border p-4 {toneClasses(item.tone)}">
                    <div class="mb-3 flex items-center justify-between">
                        <span class="text-xs font-semibold uppercase tracking-wide opacity-70">
                            Крок {i + 1}
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
<!-- Інтерактивний обхід — покроковий навігатор                              -->
<!-- ======================================================================= -->
<section class="mb-10">
    <div class="mb-4">
        <p class="text-xs font-semibold uppercase tracking-[0.2em] text-slate-400 dark:text-slate-500">
            Інтерактивний обхід
        </p>
        <h2 class="mt-2 text-2xl font-semibold text-slate-900 dark:text-slate-50">
            Покрокове пояснення
        </h2>
    </div>

    <!-- Прогрес-бар -->
    <div class="mb-6 h-2 overflow-hidden rounded-full bg-slate-200 dark:bg-slate-800">
        <div
            class="h-full rounded-full bg-blue-600 transition-all duration-300"
            style="width:{((current + 1) / walkthroughSteps.length) * 100}%"
        ></div>
    </div>

    <div class="grid gap-6 lg:grid-cols-[1.15fr_0.85fr]">
        <!-- Основна панель вмісту -->
        <section class="rounded-3xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
            <div class="flex flex-wrap items-center gap-3">
                <span class="rounded-full bg-blue-100 px-3 py-1 text-xs font-semibold uppercase tracking-wide text-blue-800 dark:bg-blue-950 dark:text-blue-200">
                    Поточний крок
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
                    Чому це важливо
                </p>
                <p class="mt-2 text-sm leading-7 text-slate-600 dark:text-slate-400">
                    {step.highlight}
                </p>
            </div>

            <div class="mt-6 grid gap-4 md:grid-cols-3">
                <!-- Колонка «Вхід» -->
                <div class="rounded-2xl border border-slate-200 bg-slate-50 p-4 dark:border-slate-700 dark:bg-slate-900">
                    <p class="text-xs font-semibold uppercase tracking-wide text-slate-400 dark:text-slate-500">
                        Вхід
                    </p>
                    <div class="mt-3 space-y-2">
                        {#each step.input as item}
                            <div class="rounded-xl bg-white px-3 py-2 text-sm text-slate-700 dark:bg-slate-800 dark:text-slate-300">
                                {item}
                            </div>
                        {/each}
                    </div>
                </div>

                <!-- Колонка «Дія» -->
                <div class="rounded-2xl border border-slate-200 bg-slate-50 p-4 dark:border-slate-700 dark:bg-slate-900">
                    <p class="text-xs font-semibold uppercase tracking-wide text-slate-400 dark:text-slate-500">
                        Що робить система
                    </p>
                    <div class="mt-3 space-y-2">
                        {#each step.action as item}
                            <div class="rounded-xl bg-white px-3 py-2 text-sm text-slate-700 dark:bg-slate-800 dark:text-slate-300">
                                {item}
                            </div>
                        {/each}
                    </div>
                </div>

                <!-- Колонка «Вихід» -->
                <div class="rounded-2xl border border-slate-200 bg-slate-50 p-4 dark:border-slate-700 dark:bg-slate-900">
                    <p class="text-xs font-semibold uppercase tracking-wide text-slate-400 dark:text-slate-500">
                        Вихід
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

        <!-- Бічна навігація -->
        <aside class="rounded-3xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
            <p class="text-xs font-semibold uppercase tracking-[0.2em] text-slate-400 dark:text-slate-500">
                Навігація по кроках
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
                    Далі
                </button>
            </div>
        </aside>
    </div>
</section>

<!-- ======================================================================= -->
<!-- Картки логіки рішень                                                    -->
<!-- ======================================================================= -->
<section class="mb-10 rounded-3xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
    <div class="mb-5">
        <p class="text-xs font-semibold uppercase tracking-[0.2em] text-slate-400 dark:text-slate-500">
            Логіка рішень
        </p>
        <h2 class="mt-2 text-2xl font-semibold text-slate-900 dark:text-slate-50">
            Чому результат стає Artifact або Real Compound
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
                        {card.kind === "good" ? "добрий" : card.kind === "bad" ? "поганий" : "інфо"}
                    </span>
                </div>

                <div class="mt-4 space-y-3">
                    <div>
                        <p class="text-xs font-semibold uppercase tracking-wide text-slate-400 dark:text-slate-500">
                            Умова
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
<!-- Типовий приклад — конкретний обхід одного піку                          -->
<!-- ======================================================================= -->
<section class="mb-10 rounded-3xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
    <div class="mb-5">
        <p class="text-xs font-semibold uppercase tracking-[0.2em] text-slate-400 dark:text-slate-500">
            Типовий приклад
        </p>
        <h2 class="mt-2 text-2xl font-semibold text-slate-900 dark:text-slate-50">
            Один пік — від вихідного сигналу до підсумкового вердикту
        </h2>
        <p class="mt-2 max-w-3xl text-sm leading-7 text-slate-600 dark:text-slate-400">
            Нижче наведено конкретний приклад того, як один пік LC-MS проходить через конвеєр —
            від початкового вимірювання зразка, через підтвердження в реплікатах і порівняння з blank,
            до остаточної класифікації.
        </p>
    </div>

    <div class="grid gap-4 lg:grid-cols-3">
        <!-- Пік зразка -->
        <div class="rounded-2xl border border-violet-200 bg-violet-50 p-5 dark:border-violet-900 dark:bg-violet-950">
            <div class="mb-3 flex items-center gap-2">
                <span class="flex h-7 w-7 items-center justify-center rounded-full bg-violet-200 text-xs font-bold text-violet-800 dark:bg-violet-800 dark:text-violet-100">S</span>
                <p class="text-sm font-semibold text-violet-900 dark:text-violet-200">Пік зразка</p>
            </div>
            <div class="space-y-2 text-sm">
                <div class="flex justify-between rounded-xl bg-white/70 px-3 py-2 dark:bg-black/20">
                    <span class="text-slate-500 dark:text-slate-400">RT</span>
                    <span class="font-mono font-semibold text-slate-800 dark:text-slate-200">{typicalExample.samplePeak.rt} хв</span>
                </div>
                <div class="flex justify-between rounded-xl bg-white/70 px-3 py-2 dark:bg-black/20">
                    <span class="text-slate-500 dark:text-slate-400">m/z</span>
                    <span class="font-mono font-semibold text-slate-800 dark:text-slate-200">{typicalExample.samplePeak.mz}</span>
                </div>
                <div class="flex justify-between rounded-xl bg-white/70 px-3 py-2 dark:bg-black/20">
                    <span class="text-slate-500 dark:text-slate-400">Площа</span>
                    <span class="font-mono font-semibold text-slate-800 dark:text-slate-200">{typicalExample.samplePeak.area.toLocaleString()}</span>
                </div>
                <div class="flex justify-between rounded-xl bg-white/70 px-3 py-2 dark:bg-black/20">
                    <span class="text-slate-500 dark:text-slate-400">Файл</span>
                    <span class="font-mono text-xs text-slate-600 dark:text-slate-300">{typicalExample.samplePeak.file}</span>
                </div>
            </div>
        </div>

        <!-- Збіг у реплікаті -->
        <div class="rounded-2xl border border-blue-200 bg-blue-50 p-5 dark:border-blue-900 dark:bg-blue-950">
            <div class="mb-3 flex items-center gap-2">
                <span class="flex h-7 w-7 items-center justify-center rounded-full bg-blue-200 text-xs font-bold text-blue-800 dark:bg-blue-800 dark:text-blue-100">R</span>
                <p class="text-sm font-semibold text-blue-900 dark:text-blue-200">Збіг у реплікаті</p>
            </div>
            <div class="space-y-2 text-sm">
                <div class="flex justify-between rounded-xl bg-white/70 px-3 py-2 dark:bg-black/20">
                    <span class="text-slate-500 dark:text-slate-400">RT</span>
                    <span class="font-mono font-semibold text-slate-800 dark:text-slate-200">{typicalExample.replicateMatch.rt} хв</span>
                </div>
                <div class="flex justify-between rounded-xl bg-white/70 px-3 py-2 dark:bg-black/20">
                    <span class="text-slate-500 dark:text-slate-400">m/z</span>
                    <span class="font-mono font-semibold text-slate-800 dark:text-slate-200">{typicalExample.replicateMatch.mz}</span>
                </div>
                <div class="flex justify-between rounded-xl bg-white/70 px-3 py-2 dark:bg-black/20">
                    <span class="text-slate-500 dark:text-slate-400">Площа</span>
                    <span class="font-mono font-semibold text-slate-800 dark:text-slate-200">{typicalExample.replicateMatch.area.toLocaleString()}</span>
                </div>
                <div class="flex justify-between rounded-xl bg-white/70 px-3 py-2 dark:bg-black/20">
                    <span class="text-slate-500 dark:text-slate-400">Файл</span>
                    <span class="font-mono text-xs text-slate-600 dark:text-slate-300">{typicalExample.replicateMatch.file}</span>
                </div>
            </div>
            <div class="mt-3 rounded-xl border border-blue-200 bg-blue-100/60 px-3 py-2 text-xs text-blue-700 dark:border-blue-800 dark:bg-blue-900/40 dark:text-blue-300">
                ΔRT = 0.01 хв ✓ &nbsp;·&nbsp; Δm/z = 0.01 Da ✓
            </div>
        </div>

        <!-- Пік blank -->
        <div class="rounded-2xl border border-cyan-200 bg-cyan-50 p-5 dark:border-cyan-900 dark:bg-cyan-950">
            <div class="mb-3 flex items-center gap-2">
                <span class="flex h-7 w-7 items-center justify-center rounded-full bg-cyan-200 text-xs font-bold text-cyan-800 dark:bg-cyan-800 dark:text-cyan-100">B</span>
                <p class="text-sm font-semibold text-cyan-900 dark:text-cyan-200">Пік blank</p>
            </div>
            <div class="space-y-2 text-sm">
                <div class="flex justify-between rounded-xl bg-white/70 px-3 py-2 dark:bg-black/20">
                    <span class="text-slate-500 dark:text-slate-400">RT</span>
                    <span class="font-mono font-semibold text-slate-800 dark:text-slate-200">{typicalExample.blankPeak.rt} хв</span>
                </div>
                <div class="flex justify-between rounded-xl bg-white/70 px-3 py-2 dark:bg-black/20">
                    <span class="text-slate-500 dark:text-slate-400">m/z</span>
                    <span class="font-mono font-semibold text-slate-800 dark:text-slate-200">{typicalExample.blankPeak.mz}</span>
                </div>
                <div class="flex justify-between rounded-xl bg-white/70 px-3 py-2 dark:bg-black/20">
                    <span class="text-slate-500 dark:text-slate-400">Площа</span>
                    <span class="font-mono font-semibold text-slate-800 dark:text-slate-200">{typicalExample.blankPeak.area.toLocaleString()}</span>
                </div>
                <div class="flex justify-between rounded-xl bg-white/70 px-3 py-2 dark:bg-black/20">
                    <span class="text-slate-500 dark:text-slate-400">Файл</span>
                    <span class="font-mono text-xs text-slate-600 dark:text-slate-300">{typicalExample.blankPeak.file}</span>
                </div>
            </div>
            <div class="mt-3 rounded-xl border border-cyan-200 bg-cyan-100/60 px-3 py-2 text-xs text-cyan-700 dark:border-cyan-800 dark:bg-cyan-900/40 dark:text-cyan-300">
                Збіг із blank знайдено · S/B = {typicalExample.results.signalToBlank} ≥ 3.0 ✓
            </div>
        </div>
    </div>

    <!-- Підсумковий вердикт -->
    <div class="mt-5 rounded-2xl border border-green-200 bg-green-50 p-5 dark:border-green-900 dark:bg-green-950">
        <div class="flex flex-wrap items-center gap-4">
            <span class="rounded-full bg-green-100 px-3 py-1 text-xs font-bold uppercase tracking-wide text-green-800 dark:bg-green-900/60 dark:text-green-200">
                {typicalExample.results.status}
            </span>
            <div class="flex flex-wrap gap-4 text-sm text-green-800 dark:text-green-300">
                <span>RT<sub>сер</sub> = {typicalExample.results.rtMean}</span>
                <span>m/z<sub>сер</sub> = {typicalExample.results.mzMean}</span>
                <span>Площа<sub>сер</sub> = {typicalExample.results.areaMean.toLocaleString()}</span>
                <span>CV% = {typicalExample.results.cvPct}</span>
                <span>S/B = {typicalExample.results.signalToBlank}</span>
            </div>
        </div>
    </div>
</section>
