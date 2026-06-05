use std::{
    process::exit,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::{
    path,
    usecase::{view::Event, DesktopServiceState},
    BUFFER,
};
use async_std::task::sleep;
use crossbeam_channel::{Receiver, Sender};
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    Emitter, Listener, Manager, WindowEvent, Wry,
};

const FILE_ITEMS: [&str; 5] = ["hbbs.out", "hbbs.err", "hbbr.out", "hbbr.err", ".env"];

#[derive(Clone)]
struct DesktopMenuHandles {
    restart: MenuItem<Wry>,
    start: MenuItem<Wry>,
    stop: MenuItem<Wry>,
    tray_restart: MenuItem<Wry>,
    tray_start: MenuItem<Wry>,
    tray_stop: MenuItem<Wry>,
    file_items: Vec<CheckMenuItem<Wry>>,
    _tray: TrayIcon<Wry>,
}

impl DesktopMenuHandles {
    fn set_file_checked(&self, id: &str) {
        for item in &self.file_items {
            item.set_checked(item.id().as_ref() == id)
                .unwrap_or_default();
        }
    }

    fn set_service_enabled(&self, id: &str, enabled: bool) {
        match id {
            "restart" => {
                self.restart.set_enabled(enabled).unwrap_or_default();
                self.tray_restart.set_enabled(enabled).unwrap_or_default();
            }
            "start" => {
                self.start.set_enabled(enabled).unwrap_or_default();
                self.tray_start.set_enabled(enabled).unwrap_or_default();
            }
            "stop" => {
                self.stop.set_enabled(enabled).unwrap_or_default();
                self.tray_stop.set_enabled(enabled).unwrap_or_default();
            }
            _ => {}
        }
    }
}

pub async fn run(sender: Sender<Event>, receiver: Receiver<Event>) {
    let setup_sender = sender.clone();
    let menu_sender = sender.clone();
    let handles: Arc<Mutex<Option<DesktopMenuHandles>>> = Arc::new(Mutex::new(None));
    let setup_handles = handles.clone();
    let mut app = tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .on_window_event(|window, event| match event {
            // WindowEvent::Resized(size) => {
            //     if size.width == 0 && size.height == 0 {
            //         window.hide().unwrap();
            //     }
            // }
            WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                window.minimize().unwrap();
                window.hide().unwrap();
            }
            _ => {}
        })
        .setup(move |app| {
            let desktop_handles = build_desktop_ui(app, menu_sender.clone())?;
            *setup_handles.lock().unwrap() = Some(desktop_handles);
            setup_sender.send(Event::ViewInit).unwrap_or_default();
            app.listen_any("__action__", move |msg| match msg.payload() {
                r#""__init__""# => setup_sender.send(Event::BrowserInit).unwrap_or_default(),
                r#""restart""# => setup_sender
                    .send(Event::BrowserAction("restart".to_owned()))
                    .unwrap_or_default(),
                _ => (),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![root])
        .build(tauri::generate_context!())
        .expect("error while running tauri application");
    let mut now = Instant::now();
    let mut blink = false;
    let mut span = 0;
    let mut title = "".to_owned();
    let product = "RustAdmin Server";
    let buffer = BUFFER.get().unwrap().to_owned();
    loop {
        for _ in 1..buffer {
            match receiver.recv_timeout(Duration::from_nanos(1)) {
                Ok(event) => {
                    let Some(main) = app.get_webview_window("main") else {
                        continue;
                    };
                    match event {
                        Event::BrowserUpdate((action, data)) => match action.as_str() {
                            "file" => {
                                let id = data.as_str();
                                if FILE_ITEMS.contains(&id) {
                                    if let Some(handles) = handles.lock().unwrap().as_ref() {
                                        handles.set_file_checked(id);
                                    }
                                    // println!(
                                    //     "emit {}: {}",
                                    //     std::time::SystemTime::now()
                                    //         .duration_since(std::time::UNIX_EPOCH)
                                    //         .unwrap_or_default()
                                    //         .as_millis(),
                                    //     data
                                    // );
                                    app.emit("__update__", (action, data)).unwrap_or_default();
                                }
                            }
                            _ => (),
                        },
                        Event::ViewRenderAppExit => exit(0),
                        Event::ViewRenderServiceState(state) => {
                            title = format!("{} {:?}", product, state);
                            main.set_title(title.as_str()).unwrap_or_default();
                            if let Some(handles) = handles.lock().unwrap().as_ref() {
                                match state {
                                    DesktopServiceState::Started => {
                                        handles.set_service_enabled("start", false);
                                        handles.set_service_enabled("stop", true);
                                        handles.set_service_enabled("restart", true);
                                        blink = false;
                                    }
                                    DesktopServiceState::Stopped => {
                                        handles.set_service_enabled("start", true);
                                        handles.set_service_enabled("stop", false);
                                        handles.set_service_enabled("restart", false);
                                        blink = true;
                                    }
                                    _ => {
                                        handles.set_service_enabled("start", false);
                                        handles.set_service_enabled("stop", false);
                                        handles.set_service_enabled("restart", false);
                                        blink = true;
                                    }
                                }
                            }
                        }
                        _ => (),
                    }
                }
                Err(_) => break,
            }
        }
        let elapsed = now.elapsed().as_micros();
        if elapsed > 16666 {
            now = Instant::now();
            // println!("{}ms", elapsed as f64 * 0.001);
            app.run_iteration(|_, _| {});
            if app.webview_windows().is_empty() {
                app.cleanup_before_exit();
                break;
            }
            if blink {
                if span > 1000000 {
                    span = 0;
                    app.get_webview_window("main")
                        .unwrap()
                        .set_title(title.as_str())
                        .unwrap_or_default();
                } else {
                    span += elapsed;
                    if span > 500000 {
                        app.get_webview_window("main")
                            .unwrap()
                            .set_title(product)
                            .unwrap_or_default();
                    }
                }
            }
        } else {
            sleep(Duration::from_micros(999)).await;
        }
    }
}

fn build_desktop_ui(
    app: &mut tauri::App,
    menu_sender: Sender<Event>,
) -> tauri::Result<DesktopMenuHandles> {
    let restart = MenuItem::with_id(app, "restart", "Restart", true, None::<&str>)?;
    let start = MenuItem::with_id(app, "start", "Start", true, None::<&str>)?;
    let stop = MenuItem::with_id(app, "stop", "Stop", true, None::<&str>)?;

    let hbbs_out = CheckMenuItem::with_id(app, "hbbs.out", "hbbs.out", true, false, None::<&str>)?;
    let hbbs_err = CheckMenuItem::with_id(app, "hbbs.err", "hbbs.err", true, false, None::<&str>)?;
    let hbbr_out = CheckMenuItem::with_id(app, "hbbr.out", "hbbr.out", true, false, None::<&str>)?;
    let hbbr_err = CheckMenuItem::with_id(app, "hbbr.err", "hbbr.err", true, false, None::<&str>)?;
    let env = CheckMenuItem::with_id(app, ".env", ".env", true, false, None::<&str>)?;

    let service_menu = Submenu::with_items(
        app,
        "Service",
        true,
        &[
            &restart,
            &PredefinedMenuItem::separator(app)?,
            &start,
            &stop,
        ],
    )?;
    let logs_menu = Submenu::with_items(
        app,
        "Logs",
        true,
        &[
            &hbbs_out,
            &hbbs_err,
            &PredefinedMenuItem::separator(app)?,
            &hbbr_out,
            &hbbr_err,
        ],
    )?;
    let configuration_menu = Submenu::with_items(app, "Configuration", true, &[&env])?;
    let menu = Menu::with_items(app, &[&service_menu, &logs_menu, &configuration_menu])?;
    app.set_menu(menu)?;
    app.on_menu_event(move |_app, event| {
        // println!(
        //     "send {}: {}",
        //     std::time::SystemTime::now()
        //         .duration_since(std::time::UNIX_EPOCH)
        //         .unwrap_or_default()
        //         .as_millis(),
        //     event.id().as_ref()
        // );
        menu_sender
            .send(Event::ViewAction(menu_action(event.id().as_ref())))
            .unwrap_or_default()
    });

    let tray_restart = MenuItem::with_id(app, "tray_restart", "Restart", true, None::<&str>)?;
    let tray_start = MenuItem::with_id(app, "tray_start", "Start", true, None::<&str>)?;
    let tray_stop = MenuItem::with_id(app, "tray_stop", "Stop", true, None::<&str>)?;
    let tray_exit = MenuItem::with_id(app, "tray_exit", "Exit GUI", true, None::<&str>)?;
    let tray_menu = Menu::with_items(
        app,
        &[
            &tray_restart,
            &PredefinedMenuItem::separator(app)?,
            &tray_start,
            &tray_stop,
            &PredefinedMenuItem::separator(app)?,
            &tray_exit,
        ],
    )?;
    let mut tray_builder = TrayIconBuilder::with_id("main")
        .menu(&tray_menu)
        .show_menu_on_left_click(false)
        .icon_as_template(true)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle_main_window(&tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray_builder = tray_builder.icon(icon.clone());
    }
    let tray = tray_builder.build(app)?;

    Ok(DesktopMenuHandles {
        restart,
        start,
        stop,
        tray_restart,
        tray_start,
        tray_stop,
        file_items: vec![hbbs_out, hbbs_err, hbbr_out, hbbr_err, env],
        _tray: tray,
    })
}

fn menu_action(id: &str) -> String {
    id.strip_prefix("tray_").unwrap_or(id).to_owned()
}

fn toggle_main_window(app: &tauri::AppHandle) {
    let Some(main) = app.get_webview_window("main") else {
        return;
    };
    if main.is_visible().unwrap_or_default() {
        main.hide().unwrap_or_default();
    } else {
        main.show().unwrap_or_default();
        main.unminimize().unwrap_or_default();
        main.set_focus().unwrap_or_default();
    }
}

#[tauri::command]
fn root() -> String {
    path().to_str().unwrap_or_default().to_owned()
}
