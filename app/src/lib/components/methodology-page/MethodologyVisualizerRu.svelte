<!--
  @file MethodologyVisualizerRu.svelte
  @description
  Русскоязычная интерактивная визуализация конвейера LC-MS скрининга.
  Объединяет блок-схему и детальный алгоритм в единый полноэкранный интерактивный компонент.
-->
<script lang="ts">
    import GlossaryTooltip from "./GlossaryTooltip.svelte";
    import { fade } from 'svelte/transition';
    import { cubicOut } from 'svelte/easing';

    // Получаем словарь терминов для подсказок
    let { defs = {} }: { defs: Record<string, string> } = $props();

    // ---------------------------------------------------------------------------
    // Единый массив шагов (Поток + Обход + Детали алгоритма)
    // ---------------------------------------------------------------------------
    const steps = [
        {
            id: "excel",
            title: "1. Чтение и валидация",
            short: "Excel и очистка данных",
            tone: "slate",
            summary: "Система загружает книгу Excel и автоматически выбирает лист с наибольшим количеством нужных колонок (RT, Area, m/z). Сразу после этого отбрасываются все неполные строки, где отсутствуют ключевые значения.",
            deepDive: "Этот этап гарантирует целостность данных. Строки без базовых параметров LC-MS не могут участвовать в расчётах и отклоняются до начала научной обработки.",
            input: ["Книга Excel (один или несколько листов)", "Строки пиков"],
            action: ["Выбрать наиболее подходящий лист", "Удалить неполные строки"],
            output: ["Валидные строки для анализа"],
            formula: "Sheet = MaxMatch(Headers, Req)  AND  Valid = (RT>0 & Area>0)",
            formulaExplanation: "Мы ищем лист, в котором названия колонок больше всего похожи на требуемые. Оставляем только те строки, где есть базовые числовые данные."
        },
        {
            id: "classify",
            title: "2. Назначение ролей",
            short: "Sample / Blank / Rep",
            tone: "violet",
            summary: "Каждой строке присваивается тип образца (Проба или Холостая проба) и она распределяется в конкретную корзину репликатов. Все дальнейшие расчёты идут отдельно для каждой полярности.",
            deepDive: "Если оператор выделил ячейки цветом в Excel (задал цвет для Blank или конкретного Sample), система доверяет этому цвету. Если цвета нет — используется логика разбора имени файла.",
            input: ["Валидные строки", "Цвета ячеек Excel", "Имена файлов"],
            action: ["Считать метку оператора", "Применить логику имени файла (fallback)"],
            output: ["Группировка по (SampleType, Полярность)"],
            formula: "Role = ColorMap[CellColor] || FileNameLogic(Name)",
            formulaExplanation: "Цвет от оператора имеет наивысший приоритет. При его отсутствии система анализирует текстовое имя на предмет паттернов."
        },
        {
            id: "replicates",
            title: "3. Кластеризация репликатов",
            short: "Подтверждение сигнала",
            tone: "blue",
            summary: "Пики из одного образца, но разных измерений (репликатов), объединяются в группы. Мы проверяем, что одно и то же химическое соединение присутствует в нескольких измерениях.",
            deepDive: "Используется жадная кластеризация: пик с наибольшей площадью становится «зерном». Из других корзин репликатов выбирается ближайший по (RT + m/z) пик. Если найдены совпадения в ≥ 2 разных корзинах, кластер подтверждён.",
            input: ["Пики, распределенные по корзинам репликатов"],
            action: ["Сортировка по убыванию площади", "Усреднение центра при добавлении", "Проверка окон допуска"],
            output: ["Подтвержденные кластеры"],
            formula: "|RT_cand − RT_cent| ≤ Tol_RT  AND  Δm/z ≤ Tol_m/z",
            formulaExplanation: "Чтобы объединить пики из разных файлов, их время удерживания (RT) и масса (m/z) должны быть почти идентичны (в пределах настроенных допусков)."
        },
        {
            id: "blank",
            title: "4. Вычитание Blank",
            short: "Сравнение с фоном",
            tone: "cyan",
            summary: "Blank-пробы проходят независимую кластеризацию. Затем каждый подтвержденный образец сравнивается с ближайшим сигналом blank для оценки шума матрицы или прибора.",
            deepDive: "Мы ищем ближайший (по RT + m/z) кластер в холостой пробе. В случае спорных ситуаций выбирается больший blank (пессимистичный подход), чтобы гарантировать чистоту результата.",
            input: ["Кластеры образца", "Кластеры blank"],
            action: ["Найти ближайший blank-кластер той же полярности", "Вычислить отношение S/B"],
            output: ["Пары Образец ↔ Blank с рассчитанным соотношением"],
            formula: "Ratio (S/B) = mean(Area_sample) / mean(Area_blank)",
            formulaExplanation: "Мы делим среднюю площадь в образце на среднюю площадь в холостой пробе. Среднее защищает от случайных всплесков."
        },
        {
            id: "decision",
            title: "5. Классификация (Вердикт)",
            short: "Artifact / Real Compound",
            tone: "green",
            summary: "Если найден подходящий blank и сигнал образца не превышает шум фона в достаточной степени (порог S/B), он отбраковывается как артефакт.",
            deepDive: "Это бинарное разделение — главный бизнес-результат скрининга. Оно позволяет химику фокусироваться только на реальных чистых соединениях, отсеивая загрязнения.",
            input: ["Отношение Signal-to-Blank (S/B)", "Порог решения"],
            action: ["Сравнить отношение S/B с настроенным порогом"],
            output: ["Вердикт: Artifact или Real Compound"],
            formula: "Status = (Ratio < Threshold) ? \"Artifact\" : \"Real Compound\"",
            formulaExplanation: "Если сигнал образца не выделяется достаточно сильно на фоне холостой пробы, мы классифицируем его как Артефакт."
        },
        {
            id: "output",
            title: "6. Итоги и Аудиторский след",
            short: "Регуляторный отчет",
            tone: "rose",
            summary: "Все этапы, приведшие к итоговому выводу, вместе с суммарными статистическими метриками сохраняются для регуляторного контроля.",
            deepDive: "Детальный журнал решений сохраняется в поле 'Why' в формате JSON. Параллельно вычисляются средние RT, m/z, площадь и вариабельность (CV%), а также рассчитывается Confidence Score.",
            input: ["Подтвержденные кластеры со статусом"],
            action: ["Расчет CV% и средних", "Расчет показателя надежности (score)", "Сериализация журнала решений"],
            output: ["Итоговая таблица (Excel для аудита)"],
            formula: "Score = 100 - Σ(Penalties) + Bonus",
            formulaExplanation: "Рейтинг достоверности от 0 до 100. Мы снимаем баллы за девиации массы/RT и разброс репликатов, и добавляем за идеальную чистоту от blank."
        }
    ];

    let current = $state(0);
    let activeStep = $derived(steps[current]);

    // Throttling для прокрутки колесом
    let lastScrollTime = 0;
    const scrollThrottle = 500; // мс

    function handleWheel(e: WheelEvent) {
        // Разрешаем системный скролл, если дельта по X больше (горизонтальный скролл)
        if (Math.abs(e.deltaX) > Math.abs(e.deltaY)) return;

        const now = Date.now();
        if (now - lastScrollTime < scrollThrottle) return;

        if (Math.abs(e.deltaY) > 10) {
            if (e.deltaY > 0 && current < steps.length - 1) {
                current++;
                lastScrollTime = now;
                e.preventDefault();
            } else if (e.deltaY < 0 && current > 0) {
                current--;
                lastScrollTime = now;
                e.preventDefault();
            }
        }
    }

    function toneClasses(tone: string, isCurrent: boolean) {
        if (isCurrent) {
            switch (tone) {
                case "amber": return "border-amber-400 bg-amber-50 shadow-md ring-2 ring-amber-400/20 dark:border-amber-500 dark:bg-amber-900/40 text-amber-900 dark:text-amber-100";
                case "violet": return "border-violet-400 bg-violet-50 shadow-md ring-2 ring-violet-400/20 dark:border-violet-500 dark:bg-violet-900/40 text-violet-900 dark:text-violet-100";
                case "blue": return "border-blue-400 bg-blue-50 shadow-md ring-2 ring-blue-400/20 dark:border-blue-500 dark:bg-blue-900/40 text-blue-900 dark:text-blue-100";
                case "cyan": return "border-cyan-400 bg-cyan-50 shadow-md ring-2 ring-cyan-400/20 dark:border-cyan-500 dark:bg-cyan-900/40 text-cyan-900 dark:text-cyan-100";
                case "green": return "border-green-400 bg-green-50 shadow-md ring-2 ring-green-400/20 dark:border-green-500 dark:bg-green-900/40 text-green-900 dark:text-green-100";
                case "rose": return "border-rose-400 bg-rose-50 shadow-md ring-2 ring-rose-400/20 dark:border-rose-500 dark:bg-rose-900/40 text-rose-900 dark:text-rose-100";
                default: return "border-slate-400 bg-slate-50 shadow-md ring-2 ring-slate-400/20 dark:border-slate-500 dark:bg-slate-800 text-slate-900 dark:text-slate-100";
            }
        } else {
            return "border-slate-200/60 bg-white/40 opacity-70 hover:opacity-100 hover:bg-white/80 dark:border-slate-700/50 dark:bg-slate-800/30 text-slate-600 dark:text-slate-400 backdrop-blur-sm transition-all";
        }
    }
</script>

<!-- ======================================================================= -->
<!-- ОБЗОРНЫЙ ХЕДЕР -->
<!-- ======================================================================= -->
<div 
    onwheel={handleWheel}
    class="mb-6 rounded-3xl border border-blue-200 bg-white p-6 shadow-sm transition-colors hover:border-blue-400 dark:border-slate-800 dark:bg-slate-900/50 dark:hover:border-slate-600"
>
    <div class="flex items-start gap-4">
        <div class="flex h-12 w-12 shrink-0 items-center justify-center rounded-2xl bg-gradient-to-br from-blue-500 to-indigo-600 text-white shadow-lg shadow-blue-500/30">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <path d="M12 2v20M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6" />
            </svg>
        </div>
        <div>
            <h2 class="text-xl font-bold tracking-tight text-slate-900 dark:text-white">Единый алгоритм скрининга</h2>
            <p class="mt-1 text-sm leading-relaxed text-slate-600 dark:text-slate-400">
                Используйте колесо мыши для навигации или кликайте по блокам. 
                Узнайте, как мы проверяем <GlossaryTooltip term="Replicate" definition={defs["Replicate"]} /> и вычитаем фон через <GlossaryTooltip term="Blank" definition={defs["Blank"]} />.
            </p>
        </div>
    </div>
</div>

<!-- ======================================================================= -->
<!-- ГОРИЗОНТАЛЬНЫЙ НАВИГАТОР (БЛОК-СХЕМА) -->
<!-- ======================================================================= -->
<div 
    onwheel={handleWheel}
    class="relative w-full overflow-x-auto pb-4 scrollbar-hide"
>
    <div class="flex min-w-[900px] items-stretch gap-3 px-1">
        {#each steps as st, i}
            <button 
                onclick={() => (current = i)}
                class="group relative flex w-[160px] shrink-0 cursor-pointer flex-col rounded-2xl border p-4 text-left transition-all duration-300 {toneClasses(st.tone, current === i)}"
            >
                <div class="mb-2 flex items-center justify-between">
                    <span class="text-[9px] font-bold uppercase tracking-wider opacity-60">
                        {current === i ? "Активен" : `Шаг ${i + 1}`}
                    </span>
                </div>
                <p class="text-sm font-bold tracking-tight leading-tight">{st.title.replace(/^\d+\.\s*/, '')}</p>
            </button> 
            
            {#if i < steps.length - 1}
                <div class="flex shrink-0 items-center justify-center text-slate-300 dark:text-slate-600">
                    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" aria-hidden="true" class="transition-colors {current === i ? 'text-blue-400 dark:text-blue-500' : ''}">
                        <path d="M5 12h14M12 5l7 7-7 7" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
                    </svg>
                </div>
            {/if}
        {/each}
    </div>
</div>

<!-- ======================================================================= -->
<!-- ДЕТАЛЬНЫЙ FULL-WIDTH ОБЗОР ШАГА -->
<!-- ======================================================================= -->
<div 
    class="relative mb-12 flex flex-col overflow-hidden rounded-3xl border border-slate-200 bg-white shadow-xl shadow-slate-200/40 backdrop-blur-xl dark:border-slate-700/60 dark:bg-slate-900/80 dark:shadow-none"
>
    
    <!-- Top Progress Bar overlay -->
    <div class="absolute left-0 top-0 h-1 w-full bg-slate-100 dark:bg-slate-800">
        <div class="h-full bg-gradient-to-r from-blue-500 to-indigo-500 transition-all duration-500 ease-out" style="width: {((current + 1) / steps.length) * 100}%"></div>
    </div>

    <!-- Container with fixed min-height to prevent page jumping -->
    <div class="min-h-[480px] p-6 lg:p-8">
        {#key current}
            <div in:fade={{ duration: 300, easing: cubicOut, delay: 50 }}>
                <div class="flex flex-col gap-8 lg:flex-row lg:items-start lg:justify-between">
                    
                    <!-- Левая часть: Суть и Описание -->
                    <div class="flex-1 lg:max-w-2xl">
                        <div class="mb-3 inline-flex items-center gap-2 rounded-full border border-blue-200 bg-blue-50 px-3 py-1 font-mono text-[10px] font-bold uppercase tracking-widest text-blue-600 dark:border-blue-900/50 dark:bg-blue-900/20 dark:text-blue-400">
                            <span>{activeStep.id}</span>
                        </div>
                        
                        <h3 class="text-2xl font-extrabold tracking-tight text-slate-900 dark:text-white">
                            {activeStep.title}
                        </h3>
                        
                        <p class="mt-4 text-base leading-relaxed text-slate-600 dark:text-slate-300">
                            {activeStep.summary}
                        </p>

                        <div class="mt-6 rounded-2xl border border-slate-100 bg-slate-50/50 p-5 dark:border-slate-800/60 dark:bg-slate-800/20">
                            <h4 class="flex items-center gap-2 text-xs font-bold uppercase tracking-wider text-slate-800 dark:text-slate-200">
                                <svg class="h-4 w-4 text-indigo-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
                                Детальный разбор
                            </h4>
                            <p class="mt-2 text-sm leading-6 text-slate-600 dark:text-slate-400">
                                {activeStep.deepDive}
                            </p>
                        </div>
                    </div>

                    <!-- Правая часть: I/O Панель -->
                    <div class="w-full shrink-0 lg:w-72">
                        <div class="rounded-2xl border border-slate-100 bg-slate-50 p-5 dark:border-slate-700/50 dark:bg-slate-800/40">
                            
                            <div class="mb-5">
                                <p class="text-[9px] font-bold uppercase tracking-widest text-slate-400 dark:text-slate-500">📥 На входе</p>
                                <ul class="mt-2 space-y-1.5">
                                    {#each activeStep.input as i}
                                        <li class="flex items-start gap-2 text-xs text-slate-700 dark:text-slate-300">
                                            <span class="mt-1.5 flex h-1 w-1 shrink-0 rounded-full bg-slate-300 dark:bg-slate-500"></span>
                                            {i}
                                        </li>
                                    {/each}
                                </ul>
                            </div>
                            
                            <div class="mb-5">
                                <p class="text-[9px] font-bold uppercase tracking-widest text-blue-500 dark:text-blue-400">⚡ Действия системы</p>
                                <ul class="mt-2 space-y-1.5 border-l-2 border-blue-100 pl-3 dark:border-blue-900/50">
                                    {#each activeStep.action as a}
                                        <li class="text-xs font-medium text-slate-800 dark:text-slate-200">{a}</li>
                                    {/each}
                                </ul>
                            </div>

                            <div>
                                <p class="text-[9px] font-bold uppercase tracking-widest text-emerald-500 dark:text-emerald-400">📤 На выходе</p>
                                <ul class="mt-2 space-y-1.5">
                                    {#each activeStep.output as o}
                                        <li class="flex items-start gap-2 text-xs text-slate-700 dark:text-slate-300">
                                            <svg class="mt-0.5 h-3.5 w-3.5 shrink-0 text-emerald-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7"></path></svg>
                                            {o}
                                        </li>
                                    {/each}
                                </ul>
                            </div>

                        </div>
                    </div>
                </div>

                <!-- Математический блок снизу -->
                <div class="mt-8 overflow-hidden rounded-2xl border border-indigo-100 bg-indigo-50/30 dark:border-indigo-900/30 dark:bg-indigo-900/10">
                    <div class="border-b border-indigo-100 bg-indigo-50 px-5 py-2.5 dark:border-indigo-900/50 dark:bg-indigo-900/20">
                        <p class="text-[10px] font-bold uppercase tracking-widest text-indigo-600 dark:text-indigo-400">
                            Математика и Логика
                        </p>
                    </div>
                    <div class="p-5">
                        <div class="font-mono text-base font-bold text-slate-900 dark:text-indigo-100 md:text-lg">
                            {activeStep.formula}
                        </div>
                        <div class="mt-3 border-t border-indigo-100 pt-3 dark:border-indigo-900/40">
                            <p class="text-sm leading-relaxed text-slate-600 dark:text-slate-400">
                                <span class="mr-1 font-semibold text-indigo-700 dark:text-indigo-300">Объяснение:</span> 
                                {activeStep.formulaExplanation}
                            </p>
                        </div>
                    </div>
                </div>

                <!-- Кнопки навигации -->
                <div class="mt-8 flex items-center justify-between border-t border-slate-100 pt-6 dark:border-slate-800">
                    <button
                        onclick={() => current > 0 && current--}
                        class="flex items-center gap-2 rounded-xl px-4 py-2 text-sm font-semibold text-slate-500 transition-colors hover:bg-slate-100 hover:text-slate-900 disabled:opacity-30 dark:text-slate-400 dark:hover:bg-slate-800 dark:hover:text-white"
                        disabled={current === 0}
                    >
                        <svg width="18" height="18" viewBox="0 0 24 24" fill="none"><path d="M15 18L9 12L15 6" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>
                        Назад
                    </button>
                    <div class="text-xs font-bold text-slate-400 dark:text-slate-500">
                        {current + 1} / {steps.length}
                    </div>
                    <button
                        onclick={() => current < steps.length - 1 && current++}
                        class="flex items-center gap-2 rounded-xl bg-slate-900 px-6 py-2.5 text-sm font-semibold text-white shadow-md transition-transform hover:scale-105 disabled:opacity-30 dark:bg-white dark:text-slate-900"
                        disabled={current === steps.length - 1}
                    >
                        Далее
                        <svg width="18" height="18" viewBox="0 0 24 24" fill="none"><path d="M9 18L15 12L9 6" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>
                    </button>
                </div>
            </div>
        {/key}
    </div>
</div>
