<!--
  @file MethodologyVisualizerUk.svelte
  @description
  Україномовна інтерактивна візуалізація конвеєра LC-MS скринінгу.
  Об'єднує блок-схему та детальний алгоритм у єдиний інтерактивний "Full-width" компонент. 
-->
<script lang="ts">
    import GlossaryTooltip from "./GlossaryTooltip.svelte";
    import { fade } from 'svelte/transition';
    import { cubicOut } from 'svelte/easing';

    // Отримуємо словник термінів для підказок
    let { defs = {} }: { defs: Record<string, string> } = $props();

    // ---------------------------------------------------------------------------
    // Об'єднаний масив кроків (Flow + Walkthrough + Algorithm Details)
    // ---------------------------------------------------------------------------
    const steps = [
        {
            id: "excel",
            title: "1. Вхідні дані та валідація",
            short: "Читання Excel та обрізка",
            tone: "slate",
            summary: "Робоча книга зчитується, і система автоматично вибирає аркуш, що містить найбільше обов'язкових колонок (RT, Area, m/z). Відразу після цього всі неповні рядки (без ключових значень) видаляються.",
            deepDive: "Цей крок гарантує чистоту даних. Рядки без базових LC-MS параметрів не підлягають подальшому розрахунку, і відхиляються до прийняття будь-яких наукових рішень.",
            input: ["Робоча книга (один або кілька аркушів)", "Рядки піків"],
            action: ["Обрати найбільш підходящий аркуш", "Видалити рядки без значень"],
            output: ["Валідні рядки для аналізу"],
            formula: "Sheet = MaxMatch(Headers, Req)  AND  Valid = (RT>0 & Area>0)",
            formulaExplanation: "Шукаємо аркуш, де назви колонок найбільше схожі на потрібні. Залишаємо лише ті рядки, де є базові числа для аналізу."
        },
        {
            id: "classify",
            title: "2. Призначення ролей",
            short: "Sample / Blank / Rep",
            tone: "violet",
            summary: "Кожному рядку призначається тип зразка (Sample або Blank) і він групується у кошик певної репліки. Далі всі обчислення відбуваються ізольовано для кожної полярності.",
            deepDive: "Якщо оператор розфарбував клітинки в Excel (встановив колір для Blank чи певного Sample), система довіряє цьому кольору. Якщо кольору немає — розбирає ім'я файлу рядка.",
            input: ["Валідні рядки", "Кольори клітинок Excel", "Імена файлів"],
            action: ["Зчитати мітку оператора", "Застосувати логіку імені файлу (fallback)"],
            output: ["Групування за (SampleType, Polarity)"],
            formula: "Role = ColorMap[CellColor] || FileNameLogic(Name)",
            formulaExplanation: "Кольорові мітки мають найвищий пріоритет. Немає кольору — аналізуємо текстову назву."
        },
        {
            id: "replicates",
            title: "3. Кластеризація реплікатів",
            short: "Підтвердження піків",
            tone: "blue",
            summary: "Піки з одного зразка, але різних вимірювань (реплікатів), групуються разом. Ми перевіряємо, що та сама хімічна сполука присутня в кількох вимірюваннях поспіль.",
            deepDive: "Використовується жадібна кластеризація: пік з найбільшою площею стає центроїдом. З іншого кошика реплікатів обирається найближчий за (RT + m/z) пік. Якщо знайдено збіги в ≥ 2 різних кошиках, кластер вважається підтвердженим.",
            input: ["Піки, розподілені по кошиках реплікатів"],
            action: ["Сортування за спаданням площі", "Усереднення центроїда при додаванні", "Перевірка допусків"],
            output: ["Підтверджені кластери"],
            formula: "|RTкан − RTцен| ≤ Tol_RT  AND  Δm/z ≤ Tol_m/z",
            formulaExplanation: "Щоб об'єднати піки з різних файлів, їхній час утримання (RT) та маса (m/z) мають бути майже однаковими (в межах допусків)."
        },
        {
            id: "blank",
            title: "4. Blank Subtraction",
            short: "Порівняння з фоном",
            tone: "cyan",
            summary: "Blank проходять кластеризацію незалежно. Далі кожен підтверджений зразок співставляється з найближчим blank-сигналом для оцінки фонового шуму хімічної матриці чи приладу.",
            deepDive: "Шукаємо найближчий за відстанню (RT + m/z) кластер у blank. При рівності відстаней завжди вибираємо більший blank (песимістичний підхід) для гарантованого відсікання помилкових сигналів.",
            input: ["Sample-кластери", "Blank-кластери"],
            action: ["Пошук найближчого blank-кластера тієї ж полярності", "Обчислення S/B ratio"],
            output: ["Пара [Sample ↔ Blank] із розрахованим відношенням"],
            formula: "Ratio (S/B) = mean(Area_sample) / mean(Area_blank)",
            formulaExplanation: "Ділимо середню площу в зразку на середню площу в холостій пробі. Середнє захищає від випадкових сплесків."
        },
        {
            id: "decision",
            title: "5. Класифікація (Підсумкове рішення)",
            short: "Artifact / Real Compound",
            tone: "green",
            summary: "Якщо знайдено відповідний blank, і сигнал зразка не перевищує фоновий шум у достатній мірі (поріг S/B), він відхиляється як артефакт.",
            deepDive: "Цей бінарний поділ є ключовим бізнес-результатом скринінгу. Дозволяє хіміку сфокусуватись лише на реальних, чистих сполуках, відкинувши все забруднення.",
            input: ["Відношення Signal-to-Blank (S/B)", "Поріг рішення"],
            action: ["Порівняти відношення S/B з налаштованим порогом"],
            output: ["Рішення: Artifact або Real Compound"],
            formula: "Status = (Ratio < Threshold) ? \"Artifact\" : \"Real Compound\"",
            formulaExplanation: "Якщо сигнал зразка недостатньо сильно виділяється на фоні холостої проби (blank), ми називаємо його Artifact."
        },
        {
            id: "output",
            title: "6. Зведена статистика та аудит",
            short: "Аудиторський слід",
            tone: "rose",
            summary: "Кожен крок, що привів до підсумкового висновку, разом із підсумковими статистичними метриками зберігається для регуляторного контролю.",
            deepDive: "Глибокий лог рішення зберігається у полі 'Why' у форматі JSON. Разом з цим обчислюються середній RT, m/z, площа та показники варіабельності (CV%), а також бонус/штраф системи оцінки довіри.",
            input: ["Підтверджені кластери зі статусом"],
            action: ["Обчислення CV% та середніх", "Розрахунок показника довіри", "Серіалізація рішення"],
            output: ["Підсумкова таблиця (готовий звіт Excel)"],
            formula: "Score = 100 - Σ(Penalties) + Bonus",
            formulaExplanation: "Рейтинг надійності від 0 до 100. Віднімаємо бали за відхилення маси/часу та розбіжності між реплікатами, додаємо за ідеальну чистоту від фону."
        }
    ];

    let current = $state(0);
    let activeStep = $derived(steps[current]);

    // Throttle для скролу мишею
    let lastScrollTime = 0;
    const scrollThrottle = 500; // мс

    function handleWheel(e: WheelEvent) {
        // Дозволяємо системний скрол, якщо дельта по X більша (горизонтальний скрол мишею)
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
            // Unselected state (faded, glass-like)
            return "border-slate-200/60 bg-white/40 opacity-70 hover:opacity-100 hover:bg-white/80 dark:border-slate-700/50 dark:bg-slate-800/30 text-slate-600 dark:text-slate-400 backdrop-blur-sm transition-all";
        }
    }

    // Допоміжна функція для безпечного парсингу термінів і заміни їх на тултипи
    // Так як Svelte не підтримує динамічні компоненти всередині рядків @html легко, 
    // ми зробили це захардкодженим на рівні шаблону (або за допомогою ручного парсингу).
    // Оскільки ми хочемо працювати з GlossaryTooltip компонентом:
</script>

<!-- ======================================================================= -->
<!-- ОГЛЯДОВИЙ ХЕДЕР -->
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
            <h2 class="text-xl font-bold tracking-tight text-slate-900 dark:text-white">Єдиний алгоритм скринінгу</h2>
            <p class="mt-1 text-sm leading-relaxed text-slate-600 dark:text-slate-400">
                Гортайте колесом миші або натискайте на блоки нижче.
                Дізнайтеся, як ми перевіряємо <GlossaryTooltip term="Replicate" definition={defs["Replicate"]} /> та відсікаємо фон по <GlossaryTooltip term="Blank" definition={defs["Blank"]} />.
            </p>
        </div>
    </div>
</div>

<!-- ======================================================================= -->
<!-- ГОРИЗОНТАЛЬНИЙ НАВІГАТОР (БЛОК-СХЕМА) -->
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
                        {current === i ? "Активовано" : `Крок ${i + 1}`}
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
<!-- ДЕТАЛЬНИЙ FULL-WIDTH ОГЛЯД КРОКУ -->
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
                    
                    <!-- Ліва частина: Суть і Опис -->
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
                                Детальний розгляд
                            </h4>
                            <p class="mt-2 text-sm leading-6 text-slate-600 dark:text-slate-400">
                                {activeStep.deepDive}
                            </p>
                        </div>
                    </div>

                    <!-- Права частина: I/O Панель -->
                    <div class="w-full shrink-0 lg:w-72">
                        <div class="rounded-2xl border border-slate-100 bg-slate-50 p-5 dark:border-slate-700/50 dark:bg-slate-800/40">
                            
                            <div class="mb-5">
                                <p class="text-[9px] font-bold uppercase tracking-widest text-slate-400 dark:text-slate-500">📥 На вході</p>
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
                                <p class="text-[9px] font-bold uppercase tracking-widest text-blue-500 dark:text-blue-400">⚡ Дії системи</p>
                                <ul class="mt-2 space-y-1.5 border-l-2 border-blue-100 pl-3 dark:border-blue-900/50">
                                    {#each activeStep.action as a}
                                        <li class="text-xs font-medium text-slate-800 dark:text-slate-200">{a}</li>
                                    {/each}
                                </ul>
                            </div>

                            <div>
                                <p class="text-[9px] font-bold uppercase tracking-widest text-emerald-500 dark:text-emerald-400">📤 На виході</p>
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

                <!-- Математичний блок знизу - Вертикальний формат -->
                <div class="mt-8 overflow-hidden rounded-2xl border border-indigo-100 bg-indigo-50/30 dark:border-indigo-900/30 dark:bg-indigo-900/10">
                    <div class="border-b border-indigo-100 bg-indigo-50 px-5 py-2.5 dark:border-indigo-900/50 dark:bg-indigo-900/20">
                        <p class="text-[10px] font-bold uppercase tracking-widest text-indigo-600 dark:text-indigo-400">
                            Математика та Логіка
                        </p>
                    </div>
                    <div class="p-5">
                        <div class="font-mono text-base font-bold text-slate-900 dark:text-indigo-100 md:text-lg">
                            {activeStep.formula}
                        </div>
                        <div class="mt-3 border-t border-indigo-100 pt-3 dark:border-indigo-900/40">
                            <p class="text-sm leading-relaxed text-slate-600 dark:text-slate-400">
                                <span class="mr-1 font-semibold text-indigo-700 dark:text-indigo-300">Пояснення:</span> 
                                {activeStep.formulaExplanation}
                            </p>
                        </div>
                    </div>
                </div>

                <!-- Кнопки навігації Всередині Картки -->
                <div class="mt-8 flex items-center justify-between border-t border-slate-100 pt-6 dark:border-slate-800">
                    <button
                        onclick={() => current > 0 && current--}
                        class="flex items-center gap-2 rounded-xl px-4 py-2 text-sm font-semibold text-slate-500 transition-colors hover:bg-slate-100 hover:text-slate-900 disabled:opacity-30 dark:text-slate-400 dark:hover:bg-slate-800 dark:hover:text-white"
                        disabled={current === 0}
                    >
                        <svg width="18" height="18" viewBox="0 0 24 24" fill="none"><path d="M15 18L9 12L15 6" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>
                        Попередній
                    </button>
                    <div class="text-xs font-bold text-slate-400 dark:text-slate-500">
                        {current + 1} / {steps.length}
                    </div>
                    <button
                        onclick={() => current < steps.length - 1 && current++}
                        class="flex items-center gap-2 rounded-xl bg-slate-900 px-6 py-2.5 text-sm font-semibold text-white shadow-md transition-transform hover:scale-105 disabled:opacity-30 dark:bg-white dark:text-slate-900"
                        disabled={current === steps.length - 1}
                    >
                        Далі
                        <svg width="18" height="18" viewBox="0 0 24 24" fill="none"><path d="M9 18L15 12L9 6" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>
                    </button>
                </div>
            </div>
        {/key}
    </div>
</div>
