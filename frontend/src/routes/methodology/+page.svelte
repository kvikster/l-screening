<svelte:head>
  <title>Методологія — LC-MS Screening</title>
</svelte:head>

<main class="min-h-screen bg-slate-50">
  <div class="max-w-4xl mx-auto px-6 py-12">

    <!-- Header -->
    <div class="mb-10 flex items-center gap-4">
      <a
        href="/"
        class="inline-flex items-center gap-1.5 text-sm text-slate-500 hover:text-slate-800 transition-colors"
      >
        <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M15 18l-6-6 6-6"/>
        </svg>
        Повернутись
      </a>
    </div>

    <h1 class="text-4xl font-bold text-slate-900 mb-2">Методологія скрінінгу</h1>
    <p class="text-slate-500 text-lg mb-12">
      Детальний опис вхідних даних, результатів, QC-метрик та audit-oriented алгоритму обрахунку LC-MS Screening.
    </p>

    <!-- TOC -->
    <nav class="mb-12 rounded-xl border border-slate-200 bg-white p-6 shadow-sm">
      <p class="text-xs font-semibold uppercase tracking-widest text-slate-400 mb-3">Зміст</p>
      <ol class="space-y-1.5 text-sm text-blue-600">
        <li><a href="#input" class="hover:underline">1. Вхідні дані</a></li>
        <li><a href="#columns" class="hover:underline">2. Обов'язкові колонки Excel</a></li>
        <li><a href="#marks" class="hover:underline">3. Operator marks (мітки оператора)</a></li>
        <li><a href="#algo" class="hover:underline">4. Алгоритм: кроки обрахунку</a></li>
        <li><a href="#output" class="hover:underline">5. Вихідні дані</a></li>
        <li><a href="#params" class="hover:underline">6. Параметри толерантності</a></li>
        <li><a href="#glossary" class="hover:underline">7. Глосарій</a></li>
      </ol>
    </nav>

    <!-- 1. Input -->
    <section id="input" class="mb-12">
      <h2 class="section-title">1. Вхідні дані</h2>
      <div class="prose-block">
        <p>
          Система приймає <strong>Excel-файл (.xlsx / .xls)</strong> з результатами LC-MS аналізу.
          Файл може містити декілька листів — система автоматично обирає той лист,
          у якому присутня найбільша кількість обов'язкових колонок.
        </p>
        <p class="mt-3">
          Типовий файл містить детектовані піки з двох <strong>replicates</strong> (повторних вимірювань)
          одного зразка, а також дані <strong>blank</strong>-зразка (холостого розчинника).
          Blank використовується для виявлення артефактів — піків, що мають хімічне походження
          не від аналіту, а від фонового шуму або розчинника.
        </p>
      </div>
    </section>

    <!-- 2. Columns -->
    <section id="columns" class="mb-12">
      <h2 class="section-title">2. Обов'язкові колонки Excel</h2>

      <div class="overflow-hidden rounded-xl border border-slate-200 bg-white shadow-sm">
        <table class="w-full text-sm">
          <thead>
            <tr class="border-b border-slate-100 bg-slate-50">
              <th class="px-5 py-3 text-left font-semibold text-slate-700">Колонка</th>
              <th class="px-5 py-3 text-left font-semibold text-slate-700">Тип</th>
              <th class="px-5 py-3 text-left font-semibold text-slate-700">Опис</th>
              <th class="px-5 py-3 text-left font-semibold text-slate-700">Приклад</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-slate-100">
            {#each columns as col}
              <tr class="hover:bg-slate-50/60 transition-colors">
                <td class="px-5 py-3 font-mono font-semibold text-blue-700">{col.name}</td>
                <td class="px-5 py-3 text-slate-500">{col.type}</td>
                <td class="px-5 py-3 text-slate-700">{col.desc}</td>
                <td class="px-5 py-3 font-mono text-slate-500">{col.example}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>

      <div class="mt-4 rounded-lg border border-amber-200 bg-amber-50 p-4 text-sm text-amber-800">
        <strong>Опціонально:</strong> якщо у файлі присутня колонка <code class="font-mono">Label</code>,
        вона зберігається у результатах для зручності ідентифікації піку (наприклад, назва сполуки
        або внутрішній ідентифікатор). Якщо присутня колонка <code class="font-mono">operator_mark</code>
        — вона пріоритизується над іменем файлу для визначення типу зразка.
      </div>
    </section>

    <!-- 3. Marks -->
    <section id="marks" class="mb-12">
      <h2 class="section-title">3. Operator marks (мітки оператора)</h2>
      <div class="prose-block">
        <p>
          Оператор може вручну <strong>виділити клітинки кольором</strong> у Excel-файлі,
          щоб явно вказати роль кожного рядка. Це більш надійний спосіб, ніж покладатися
          лише на ім'я файлу. Якщо система виявляє кольорове маркування — воно
          <em>пріоритизується</em> над іменем файлу.
        </p>
      </div>

      <div class="mt-5 grid grid-cols-1 sm:grid-cols-2 gap-4">
        {#each operatorMarks as m}
          <div class="flex items-start gap-3 rounded-xl border border-slate-200 bg-white p-4 shadow-sm">
            <span
              class="mt-0.5 inline-block h-4 w-4 flex-shrink-0 rounded-full border border-black/10 shadow"
              style="background-color: {m.color}"
            ></span>
            <div>
              <p class="font-mono font-semibold text-slate-800 text-sm">{m.mark}</p>
              <p class="text-xs text-slate-500 mt-0.5">{m.desc}</p>
            </div>
          </div>
        {/each}
      </div>

      <p class="mt-4 text-sm text-slate-500">
        Якщо кольорове маркування відсутнє — система класифікує рядки за
        <strong>іменем файлу</strong>: рядки з файлів, що містять слово <code class="font-mono">blank</code>,
        відносяться до blank; файли що починаються з цифри відносяться до sample.
        Перший та другий унікальні файли в групі вважаються Rep 1 та Rep 2.
      </p>
    </section>

    <!-- 4. Algorithm -->
    <section id="algo" class="mb-12">
      <h2 class="section-title">4. Алгоритм: кроки обрахунку</h2>

      <!-- Step 1 -->
      <div class="step-card">
        <div class="step-number">1</div>
        <div class="flex-1">
          <h3 class="step-title">Попередня обробка</h3>
          <ul class="step-list">
            <li>Зчитується Excel-файл; вибирається лист з найбільшою кількістю обов'язкових колонок.</li>
            <li>Рядки з порожнім <code class="code">RT</code> видаляються.</li>
            <li>Кожному рядку присвоюється <code class="code">SampleType</code>
              (<code class="code">blank</code> або <code class="code">sample</code>) на основі
              operator mark або імені файлу.</li>
            <li>Зчитується колір клітинки (якщо наявний) — для визначення Rep 1 / Rep 2.</li>
          </ul>
        </div>
      </div>

      <!-- Step 2 -->
      <div class="step-card">
        <div class="step-number">2</div>
        <div class="flex-1">
          <h3 class="step-title">Coarse Screening — підтвердження реплік</h3>
          <p class="text-slate-600 text-sm mb-3">
            Групуємо рядки за <code class="code">(SampleType, Polarity)</code>.
            У кожній групі система формує <strong>replicate buckets</strong>:
            або за operator marks, або за окремими файлами. Далі виконується
            <strong>greedy clustering</strong> з правилом "не більше одного піку з кожного replicate bucket".
          </p>
          <div class="formula-block">
            <p class="formula-label">Умова підтвердження піку:</p>
            <p class="font-mono">|RT₁ − RT₂| ≤ replicate_rt_tol</p>
            <p class="font-mono mt-1">|mz₁ − mz₂| ≤ replicate_mz_tol</p>
            <p class="mt-2 text-xs text-slate-500">
              Допуск по <code class="code">m/z</code> працює у вибраному режимі:
              <code class="code">Da</code> або <code class="code">ppm</code>.
            </p>
            <p class="mt-2 text-xs text-slate-500">
              Якщо обидві умови виконані — пік <strong>підтверджений</strong>.
              RT та m/z усереднюються: <code class="code">RT_mean = (RT₁ + RT₂) / 2</code>,
              аналогічно для <code class="code">MZ_mean</code> та <code class="code">Area_mean</code>.
              Додатково система обчислює <code class="code">AreaCVPct</code>,
              <code class="code">ReplicateQuality</code> і початковий <code class="code">ReplicateConfidenceScore</code>.
            </p>
          </div>
          <p class="mt-3 text-sm text-slate-500">
            Це зменшує combinatorial overmatching, який характерний для cross-join, і дозволяє
            підтримувати <strong>n &gt; 2 replicate files</strong>. Піки, що не знайшли кластер з
            принаймні двома replicate buckets, відкидаються як ненадійні.
          </p>
        </div>
      </div>

      <!-- Step 3 -->
      <div class="step-card">
        <div class="step-number">3</div>
        <div class="flex-1">
          <h3 class="step-title">Out-Target Screening — blank subtraction</h3>
          <p class="text-slate-600 text-sm mb-3">
            Беремо підтверджені піки <strong>sample</strong> та зіставляємо їх із підтвердженими
            піками <strong>blank</strong> тієї ж полярності.
          </p>
          <div class="formula-block">
            <p class="formula-label">Умова blank match:</p>
            <p class="font-mono">Polarity_sample == Polarity_blank</p>
            <p class="font-mono mt-1">|RT_mean_sample − RT_mean_blank| ≤ blank_rt_tol</p>
            <p class="font-mono mt-1">|MZ_mean_sample − MZ_mean_blank| ≤ blank_mz_tol</p>
            <p class="font-mono mt-3">S/B = Area_mean_sample / Area_mean_blank</p>
            <div class="mt-3 grid grid-cols-1 sm:grid-cols-2 gap-2 text-xs">
              <div class="rounded-md bg-red-50 border border-red-200 p-2 text-red-800">
                <strong>Artifact</strong> — blank match знайдено і
                <code class="code">S/B &lt; signal_to_blank_min</code>.
                Пік, ймовірно, походить від розчинника або фону, а не від аналіту.
              </div>
              <div class="rounded-md bg-green-50 border border-green-200 p-2 text-green-800">
                <strong>Real Compound</strong> — blank match відсутній або
                <code class="code">S/B ≥ signal_to_blank_min</code>.
                Пік специфічний для зразка — ймовірно є сполукою аналіту.
              </div>
            </div>
            <p class="mt-3 text-xs text-slate-500">
              Це відокремлює replicate matching від blank subtraction: для них можуть бути різні
              RT/m/z пороги та різні m/z режими.
            </p>
          </div>
        </div>
      </div>

      <!-- Step 4 -->
      <div class="step-card">
        <div class="step-number">4</div>
        <div class="flex-1">
          <h3 class="step-title">Summary — зведена статистика</h3>
          <p class="text-slate-600 text-sm">
            Для кожної комбінації <code class="code">(SampleType, Polarity)</code> обраховуються:
          </p>
          <ul class="step-list mt-2">
            <li><strong>Total Peaks</strong> — загальна кількість рядків у вхідних даних.</li>
            <li><strong>Confirmed</strong> — кількість підтверджених пар після Coarse Screening.</li>
            <li><strong>Artifacts</strong> — підтверджені піки sample, що збіглися з blank.</li>
            <li><strong>Real Compounds</strong> — підтверджені піки sample без збігу в blank.</li>
            <li><strong>Mean CV%</strong> — середній CV між підтвердженими реплікатами.</li>
            <li><strong>Quality H/M/L</strong> — розподіл піків за <code class="code">ReplicateQuality</code>.</li>
            <li><strong>Mean Confidence</strong> — середній підсумковий confidence score.</li>
          </ul>
        </div>
      </div>
    </section>

    <!-- 5. Output -->
    <section id="output" class="mb-12">
      <h2 class="section-title">5. Вихідні дані</h2>

      <div class="space-y-4">
        {#each outputFields as f}
          <div class="flex items-start gap-3 rounded-xl border border-slate-200 bg-white p-4 shadow-sm text-sm">
            <code class="font-mono font-semibold text-blue-700 min-w-[140px] flex-shrink-0">{f.name}</code>
            <div>
              <p class="text-slate-700">{f.desc}</p>
              {#if f.note}
                <p class="mt-1 text-xs text-slate-400">{f.note}</p>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    </section>

    <!-- 6. Parameters -->
    <section id="params" class="mb-12">
      <h2 class="section-title">6. Параметри толерантності</h2>
      <div class="overflow-hidden rounded-xl border border-slate-200 bg-white shadow-sm">
        <table class="w-full text-sm">
          <thead>
            <tr class="border-b border-slate-100 bg-slate-50">
              <th class="px-5 py-3 text-left font-semibold text-slate-700">Параметр</th>
              <th class="px-5 py-3 text-left font-semibold text-slate-700">Значення</th>
              <th class="px-5 py-3 text-left font-semibold text-slate-700">Одиниця</th>
              <th class="px-5 py-3 text-left font-semibold text-slate-700">Де застосовується</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-slate-100">
            <tr class="hover:bg-slate-50/60">
              <td class="px-5 py-3 font-mono font-semibold text-blue-700">replicate_rt_tol</td>
              <td class="px-5 py-3 font-mono">0.1</td>
              <td class="px-5 py-3 text-slate-500">хвилини</td>
              <td class="px-5 py-3 text-slate-700">Coarse Screening</td>
            </tr>
            <tr class="hover:bg-slate-50/60">
              <td class="px-5 py-3 font-mono font-semibold text-blue-700">replicate_mz_tol</td>
              <td class="px-5 py-3 font-mono">0.3</td>
              <td class="px-5 py-3 text-slate-500">Da або ppm</td>
              <td class="px-5 py-3 text-slate-700">Coarse Screening</td>
            </tr>
            <tr class="hover:bg-slate-50/60">
              <td class="px-5 py-3 font-mono font-semibold text-blue-700">blank_rt_tol</td>
              <td class="px-5 py-3 font-mono">0.1</td>
              <td class="px-5 py-3 text-slate-500">хвилини</td>
              <td class="px-5 py-3 text-slate-700">Out-Target Screening</td>
            </tr>
            <tr class="hover:bg-slate-50/60">
              <td class="px-5 py-3 font-mono font-semibold text-blue-700">blank_mz_tol</td>
              <td class="px-5 py-3 font-mono">0.3</td>
              <td class="px-5 py-3 text-slate-500">Da або ppm</td>
              <td class="px-5 py-3 text-slate-700">Out-Target Screening</td>
            </tr>
            <tr class="hover:bg-slate-50/60">
              <td class="px-5 py-3 font-mono font-semibold text-blue-700">signal_to_blank_min</td>
              <td class="px-5 py-3 font-mono">3.0</td>
              <td class="px-5 py-3 text-slate-500">ratio</td>
              <td class="px-5 py-3 text-slate-700">Artifact / Real Compound decision</td>
            </tr>
          </tbody>
        </table>
      </div>
      <p class="mt-3 text-xs text-slate-400">
        Значення мають дефолт у backend-конфігурації, але можуть бути змінені у screening form на головній сторінці.
        Replicate matching і blank subtraction налаштовуються незалежно.
      </p>
    </section>

    <!-- 7. Glossary -->
    <section id="glossary" class="mb-12">
      <h2 class="section-title">7. Глосарій</h2>
      <div class="space-y-3">
        {#each glossary as g}
          <div class="flex gap-3 text-sm">
            <dt class="font-mono font-semibold text-blue-700 min-w-[150px] flex-shrink-0">{g.term}</dt>
            <dd class="text-slate-700">{g.def}</dd>
          </div>
        {/each}
      </div>
    </section>

  </div>
</main>

<script lang="ts">
  const columns = [
    { name: "RT",         type: "number",  desc: "Retention time — час утримання піку в колонці",       example: "2.345" },
    { name: "Base Peak",  type: "number",  desc: "m/z значення базового піку (основний іон у спектрі)", example: "195.08" },
    { name: "Polarity",   type: "string",  desc: "Полярність іонізації",                               example: "positive / negative" },
    { name: "File",       type: "string",  desc: "Ім'я вихідного файлу даних (використовується для ідентифікації зразка та репліки)", example: "1_pos.d" },
    { name: "Area",       type: "number",  desc: "Площа піку — пропорційна концентрації сполуки",       example: "1250000" },
    { name: "Label",      type: "string",  desc: "(Опціонально) Мітка або назва піку від оператора",   example: "Caffeine" },
  ];

  const operatorMarks = [
    { mark: "sample_rep1",    color: "#ff00ff", desc: "Sample, Replicate 1" },
    { mark: "sample_rep2",    color: "#ffff00", desc: "Sample, Replicate 2" },
    { mark: "blank_positive", color: "#00ffff", desc: "Blank" },
    { mark: "blank_negative", color: "#00ff00", desc: "Blank" },
  ];

  const outputFields = [
    { name: "RT_mean",    desc: "Середнє значення RT між Rep 1 та Rep 2.",              note: "RT_mean = (RT₁ + RT₂) / 2, округлено до 4 знаків" },
    { name: "MZ_mean",    desc: "Середнє значення m/z між Rep 1 та Rep 2.",             note: "MZ_mean = (mz₁ + mz₂) / 2, збережено з вищою точністю для audit trail" },
    { name: "Area_mean",  desc: "Середня площа піку між Rep 1 та Rep 2.",               note: "Не обрізається до int, щоб не втрачати precision" },
    { name: "AreaCVPct",  desc: "CV% між replicate areas — ключовий показник відтворюваності.", note: null },
    { name: "ReplicateQuality", desc: "Категорія якості реплікатів: High / Moderate / Low.", note: "Визначається за CV%" },
    { name: "SignalToBlankRatio", desc: "Відношення sample signal до blank signal для matched blank peak.", note: "Artifact зазвичай означає S/B < threshold" },
    { name: "ConfidenceScore", desc: "Підсумковий confidence score з урахуванням replicate agreement та blank subtraction.", note: "0–100" },
    { name: "SampleType", desc: "Тип зразка: sample або blank.",                        note: null },
    { name: "Polarity",   desc: "Полярність: positive або negative.",                   note: null },
    { name: "Status",     desc: "Результат класифікації: Real Compound або Artifact.",  note: "Real Compound — специфічний для зразка пік; Artifact — наявний і в blank" },
    { name: "Rep1_Mark",  desc: "Operator mark першої репліки (якщо є кольорове маркування).", note: null },
    { name: "Rep2_Mark",  desc: "Operator mark другої репліки (якщо є кольорове маркування).", note: null },
    { name: "Why",        desc: "JSON-об'єкт з деталями логіки рішення: replicate deltas, tolerance mode, CV%, blank match, S/B ratio та decision trail.", note: "Відображається у modal «Logic Detail» у таблиці результатів" },
  ];

  const glossary = [
    { term: "RT",            def: "Retention Time — час утримання аналіту в хроматографічній колонці (хвилини)." },
    { term: "m/z",           def: "Mass-to-charge ratio — відношення маси іона до його заряду; ідентифікатор сполуки в мас-спектрометрії." },
    { term: "Base Peak",     def: "Іон з найбільшою інтенсивністю у мас-спектрі для даного піку." },
    { term: "Polarity",      def: "Режим іонізації: positive (ESI+) або negative (ESI−). Визначає тип іонів, що детектуються." },
    { term: "Replicate",     def: "Повторне незалежне вимірювання того самого зразка для підтвердження відтворюваності." },
    { term: "Blank",         def: "Зразок-холостий (розчинник без аналіту). Піки, присутні в blank, вважаються фоновими артефактами." },
    { term: "CV%",           def: "Coefficient of Variation — відносна варіабельність replicate areas. Нижчий CV означає кращу відтворюваність." },
    { term: "S/B ratio",     def: "Signal-to-Blank ratio — співвідношення sample signal до blank signal для matched піку. Використовується замість бінарного blank subtraction." },
    { term: "ppm",           def: "Parts per million — відносний допуск по m/z, типовий для high-resolution mass spectrometry." },
    { term: "Confidence Score", def: "Зведений показник довіри до піку на основі replicate agreement, CV та blank subtraction outcome." },
    { term: "Coarse Screening", def: "Перший етап фільтрації: підтвердження піку через збіг між двома репліками за RT та m/z." },
    { term: "Out-Target Screening", def: "Другий етап: blank subtraction — видалення піків, що присутні в blank-зразку." },
    { term: "Artifact",      def: "Пік, що знайдений у blank — ймовірно, артефакт матриці або забруднення, не пов'язане з аналітом." },
    { term: "Real Compound", def: "Пік, відсутній у blank — специфічний для зразка, ймовірно є досліджуваною сполукою." },
    { term: "Operator mark", def: "Ручне кольорове маркування клітинок у Excel-файлі оператором для явного позначення типу рядка." },
    { term: "replicate_rt_tol / blank_rt_tol", def: "Окремі допуски по RT для replicate matching та blank subtraction." },
    { term: "replicate_mz_tol / blank_mz_tol", def: "Окремі допуски по m/z для replicate matching та blank subtraction; можуть працювати в Da або ppm." },
  ];
</script>

<style>
  :global(.section-title) {
    font-size: 1.375rem;
    font-weight: 700;
    color: #0f172a;
    margin-bottom: 1rem;
    padding-bottom: 0.5rem;
    border-bottom: 2px solid #e2e8f0;
  }
  :global(.prose-block) {
    color: #475569;
    font-size: 0.9375rem;
    line-height: 1.7;
  }
  :global(.step-card) {
    display: flex;
    gap: 1rem;
    margin-bottom: 1.25rem;
    background: white;
    border: 1px solid #e2e8f0;
    border-radius: 0.875rem;
    padding: 1.25rem;
    box-shadow: 0 1px 3px 0 rgb(0 0 0 / 0.04);
  }
  :global(.step-number) {
    flex-shrink: 0;
    width: 2rem;
    height: 2rem;
    border-radius: 9999px;
    background: #eff6ff;
    color: #2563eb;
    font-weight: 700;
    font-size: 0.875rem;
    display: flex;
    align-items: center;
    justify-content: center;
    margin-top: 0.125rem;
  }
  :global(.step-title) {
    font-weight: 600;
    font-size: 1rem;
    color: #0f172a;
    margin-bottom: 0.5rem;
  }
  :global(.step-list) {
    list-style: disc;
    padding-left: 1.25rem;
    font-size: 0.875rem;
    color: #475569;
    line-height: 1.6;
  }
  :global(.formula-block) {
    background: #f8fafc;
    border: 1px solid #e2e8f0;
    border-left: 4px solid #3b82f6;
    border-radius: 0.5rem;
    padding: 1rem;
    font-size: 0.875rem;
  }
  :global(.formula-label) {
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: #94a3b8;
    margin-bottom: 0.5rem;
  }
  :global(.code) {
    font-family: monospace;
    font-size: 0.85em;
    background: #f1f5f9;
    border-radius: 0.25rem;
    padding: 0.1em 0.35em;
  }
  :global(body) {
    font-family: 'Inter', system-ui, -apple-system, sans-serif;
  }
</style>
