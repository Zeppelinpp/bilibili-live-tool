use crate::state::AppState;
use tauri::Manager;

#[tauri::command]
pub fn window_min(window: tauri::Window) {
    let _ = window.minimize();
}

#[tauri::command]
pub fn window_max(window: tauri::Window) -> Result<bool, String> {
    let is_max = window.is_maximized().map_err(|e| e.to_string())?;
    if is_max {
        window.unmaximize().map_err(|e| e.to_string())?;
    } else {
        window.maximize().map_err(|e| e.to_string())?;
    }
    Ok(!is_max)
}

#[tauri::command]
pub fn window_close(window: tauri::Window) {
    let _ = window.close();
}

#[tauri::command]
pub fn window_drag(window: tauri::Window, _x: i32, _y: i32) {
    let _ = window.start_dragging();
}

#[tauri::command]
pub fn set_window_background(window: tauri::Window, r: u8, g: u8, b: u8) {
    let _ = window.set_background_color(Some(tauri::window::Color(r, g, b, 255)));
}

#[tauri::command]
pub async fn open_danmaku_float(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    // If already open, focus and return
    if let Some(win) = app_handle.get_webview_window("danmaku-float") {
        let _ = win.set_focus();
        return Ok(());
    }

    // Read saved state or use defaults
    let config = state.config.lock().await;
    let (width, height, x, y) = config
        .data()
        .float_window
        .as_ref()
        .map(|f| (f.width, f.height, f.x, f.y))
        .unwrap_or((320.0, 450.0, 0.0, 0.0));
    drop(config);

    let mut builder = tauri::WebviewWindowBuilder::new(
        &app_handle,
        "danmaku-float",
        tauri::WebviewUrl::App("/".into()),
    )
    .title("Monitor")
    .decorations(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .transparent(true)
    .inner_size(width, height);

    // Only set position if we have a saved state; otherwise center
    if width > 0.0 && height > 0.0 && x != 0.0 && y != 0.0 {
        builder = builder.position(x, y);
    } else {
        builder = builder.center();
    }

    builder.build().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn close_danmaku_float(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let Some(win) = app_handle.get_webview_window("danmaku-float") else {
        return Ok(());
    };

    // Read current position and size
    let pos = win.outer_position().map_err(|e| e.to_string())?;
    let size = win.inner_size().map_err(|e| e.to_string())?;

    // Save to config
    let mut config = state.config.lock().await;
    config.data_mut().float_window = Some(crate::models::config::FloatWindowState {
        x: pos.x as f64,
        y: pos.y as f64,
        width: size.width as f64,
        height: size.height as f64,
    });
    let _ = config.save();
    drop(config);

    // Close the window
    let _ = win.close();
    Ok(())
}
