mod core;
use core::panel;
use tauri::Manager;

#[cfg(target_os = "macos")]
extern crate core_foundation;
#[cfg(target_os = "macos")]
extern crate core_graphics;

use std::thread;
use std::time::Duration;

#[cfg(target_os = "macos")]
extern "C" {
    fn CGSessionCopyCurrentDictionary() -> core_foundation::dictionary::CFDictionaryRef;
}

#[cfg(target_os = "macos")]
use core_foundation::{base::TCFType, base::ToVoid, dictionary::CFDictionary, string::CFString};

// use tauri_plugin_autostart::MacosLauncher;
// use tauri_plugin_eco_window::{show_main_window, MAIN_WINDOW_LABEL, PREFERENCE_WINDOW_LABEL};
// use tauri_plugin_log::{Target, TargetKind};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(target_os = "macos")]
            {
                let is_running = Arc::new(AtomicBool::new(true));
                let is_running_clone = is_running.clone();

                // 计时器线程
                thread::spawn(move || loop {
                    if is_running_clone.load(Ordering::SeqCst) {
                        let mut start_time = Instant::now();
                        while is_running_clone.load(Ordering::SeqCst) {
                            let elapsed = start_time.elapsed();
                            println!("计时：{:?}", elapsed);
                            thread::sleep(Duration::from_secs(1));
                        }
                    }
                    thread::sleep(Duration::from_millis(100));
                });

                // 锁屏监听线程
                thread::spawn(move || {
                    let mut flg = false;
                    let lock_key = CFString::new("CGSSessionScreenIsLocked");
                    loop {
                        unsafe {
                            let session_dictionary_ref = CGSessionCopyCurrentDictionary();
                            let session_dictionary: CFDictionary =
                                CFDictionary::wrap_under_create_rule(session_dictionary_ref);
                            let current_session_property =
                                session_dictionary.find(lock_key.to_void()).is_some();

                            if flg != current_session_property {
                                flg = current_session_property;
                                is_running.store(!current_session_property, Ordering::SeqCst);
                                println!(
                                    "系统{}，{}计时",
                                    if current_session_property {
                                        "锁屏"
                                    } else {
                                        "解锁"
                                    },
                                    if current_session_property {
                                        "停止"
                                    } else {
                                        "开始"
                                    }
                                );
                            }
                            thread::sleep(Duration::from_millis(1000));
                        }
                    }
                });
            }

            let main_window = app.get_webview_window("main").unwrap();
            #[cfg(target_os = "macos")]
            panel::platform(app, main_window.clone());

            Ok(())
        })
        // .plugin()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
