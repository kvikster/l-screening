# --- CONFIG ---
$AppName = "lc-ms-screening"
$PythonVersion = "3.12.3"
# --------------

Write-Host "🚀 Починаємо локальну збірку $AppName для Windows..." -ForegroundColor Cyan

# 1. Очищення
if (Test-Path "src-tauri/pyembed") { Remove-Item -Recurse -Force "src-tauri/pyembed" }
New-Item -ItemType Directory -Path "src-tauri/pyembed"

# 2. Збірка Frontend
Write-Host "📦 Збірка Frontend (SvelteKit)..."
Set-Location frontend
npm install
npm run build
Set-Location ..

# 3. Завантаження портативного Python (Windows x64)
Write-Host "🐍 Завантаження портативного Python..."
$Url = "https://github.com/indygreg/python-build-standalone/releases/download/20240415/cpython-$PythonVersion+20240415-x86_64-pc-windows-msvc-shared-install_only.tar.gz"
$Dest = "src-tauri/pyembed/python.tar.gz"

Invoke-WebRequest -Uri $Url -OutFile $Dest
# Використання tar для розпакування (вбудований у Windows 10+)
tar -xzf $Dest -C src-tauri/pyembed
Remove-Item $Dest

# 4. Встановлення Python-залежностей
Write-Host "🛠 Встановлення залежностей..."
$PyExe = "src-tauri/pyembed/python/python.exe"
& $PyExe -m pip install uv
& $PyExe -m uv pip install -e .

# 5. Збірка Tauri-додатка
Write-Host "🏗 Фінальна збірка Tauri..."
$env:PYTAURI_STANDALONE = "1"
Set-Location src-tauri
cargo tauri build
Set-Location ..

Write-Host "✅ Збірка завершена! Результат у src-tauri/target/release/bundle/" -ForegroundColor Green
