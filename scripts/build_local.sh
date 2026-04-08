#!/bin/bash
set -e

# --- CONFIG ---
APP_NAME="lc-ms-screening"
PYTHON_VERSION="3.14.3"
# --------------

echo "🚀 Починаємо локальну збірку $APP_NAME..."

# 1. Очищення
rm -rf src-tauri/pyembed
mkdir -p src-tauri/pyembed

# 2. Збірка Frontend
echo "📦 Збірка Frontend (SvelteKit)..."
cd frontend
bun install
bun run build
cd ..

# 3. Завантаження портативного Python
echo "🐍 Завантаження портативного Python..."
ARCH=$(uname -m)
OS=$(uname -s | tr '[:upper:]' '[:lower:]')

if [[ "$OS" == "darwin" ]]; then
    if [[ "$ARCH" == "arm64" ]]; then
        URL="https://github.com/indygreg/python-build-standalone/releases/download/20260203/cpython-${PYTHON_VERSION}+20260203-aarch64-apple-darwin-install_only.tar.gz"
    else
        URL="https://github.com/indygreg/python-build-standalone/releases/download/20260203/cpython-${PYTHON_VERSION}+20260203-x86_64-apple-darwin-install_only.tar.gz"
    fi
elif [[ "$OS" == "linux" ]]; then
    URL="https://github.com/indygreg/python-build-standalone/releases/download/20260203/cpython-${PYTHON_VERSION}+20260203-x86_64-unknown-linux-gnu-install_only.tar.gz"
else
    echo "⚠️ OS $OS не підтримується цим скриптом безпосередньо."
    exit 1
fi

# Extract directly as 'python' (do NOT rename — tauri.conf.json maps pyembed/python -> Resources/)
curl -L $URL | tar -xz -C src-tauri/pyembed

# 4. Налаштування оточення для PyO3
echo "🛠 Налаштування оточення..."
PY_DIST_DIR="$(pwd)/src-tauri/pyembed/python"

if [[ "$OS" == "darwin" ]]; then
    export PY_BIN="$PY_DIST_DIR/bin/python3"
    export PYO3_PYTHON="$PY_BIN"

    # Patch libpython so dyld can find it via @rpath at runtime.
    # python-build-standalone ships the dylib with an absolute install name;
    # we must replace it with the @rpath-relative name the binary expects.
    LIBPYTHON=$(ls "$PY_DIST_DIR/lib/libpython"*.dylib | head -1)
    LIBPYTHON_BASENAME=$(basename "$LIBPYTHON")
    echo "🔧 Patching $LIBPYTHON_BASENAME install name..."
    install_name_tool -id "@rpath/$LIBPYTHON_BASENAME" "$LIBPYTHON"

    export RUSTFLAGS="-C link-arg=-Wl,-rpath,@executable_path/../Resources/lib -L $PY_DIST_DIR/lib"
else
    export PY_BIN="$PY_DIST_DIR/bin/python3"
    export PYO3_PYTHON="$PY_BIN"
    export RUSTFLAGS="-C link-arg=-Wl,-rpath,\$ORIGIN/../lib/$APP_NAME/lib -L $PY_DIST_DIR/lib"
fi

# 5. Встановлення Python-залежностей через uv
echo "📦 Встановлення Python-залежностей..."
# Use SYSTEM uv (not embedded Python) so Rust/maturin build tools are available,
# but install INTO the embedded Python via --python flag.
uv pip install \
    --python="$PY_BIN" \
    --reinstall-package=lc_ms_screening \
    .

# 6. Збірка Tauri-додатка
echo "🏗 Фінальна збірка Tauri..."
export PYTAURI_STANDALONE="1"

cd src-tauri
npx -y @tauri-apps/cli build --features pytauri-standalone
cd ..

echo "✅ Збірка завершена! Результат у src-tauri/target/release/bundle/"
