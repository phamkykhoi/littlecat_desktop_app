// Ẩn console window trên Windows (release build)
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    hoc_long_tieng_lib::run();
}
