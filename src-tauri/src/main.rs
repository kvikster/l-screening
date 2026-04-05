#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{convert::Infallible, error::Error, path::PathBuf};

use pyo3::wrap_pymodule;
use pytauri::standalone::{
    dunce::simplified, PythonInterpreterBuilder, PythonInterpreterEnv, PythonScript,
};
use tauri::utils::platform::resource_dir;

use lc_ms_screening_lib::{ext_mod, tauri_generate_context};

fn main() -> Result<Infallible, Box<dyn Error>> {
    let py_env = if cfg!(dev) {
        // Development mode: use local virtual environment
        let mut venv_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        venv_dir.pop(); // src-tauri -> project root
        venv_dir.push(".venv");
        assert!(
            venv_dir.is_dir(),
            "Python virtual environment not found at: {}",
            venv_dir.display()
        );
        PythonInterpreterEnv::Venv(venv_dir.into())
    } else {
        // Production mode: use embedded Python bundled with the app
        let context = tauri_generate_context();
        let resource_dir = resource_dir(context.package_info(), &tauri::Env::default())
            .map_err(|err| format!("failed to get resource dir: {err}"))?;
        // Remove UNC prefix `\\?\` — Python ecosystems don't like it
        let resource_dir = simplified(&resource_dir).to_owned();
        PythonInterpreterEnv::Standalone(resource_dir.into())
    };

    // Equivalent to `python -m lc_ms_screening`
    let py_script = PythonScript::Module("lc_ms_screening".into());

    // Export ext_mod from memory (no separate .so needed in standalone mode)
    let builder =
        PythonInterpreterBuilder::new(py_env, py_script, |py| wrap_pymodule!(ext_mod)(py));
    let interpreter = builder.build()?;

    let exit_code = interpreter.run();
    std::process::exit(exit_code);
}
