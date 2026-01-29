//! Aster Tauri Desktop Application Entry Point

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    aster_tauri_lib::run()
}
