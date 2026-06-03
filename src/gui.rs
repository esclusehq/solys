// Windows GUI Module for Solys Agent
// Provides system tray icon and status window

#[cfg(target_os = "windows")]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(target_os = "windows")]
use std::sync::Arc;

#[cfg(target_os = "windows")]
use tray_item::{TrayItem, IconSource};

#[cfg(target_os = "windows")]
pub struct GuiApp {
    _tray: TrayItem,
    running: Arc<AtomicBool>,
}

#[cfg(target_os = "windows")]
impl GuiApp {
    pub fn new() -> anyhow::Result<Self> {
        // Create tray item with label only (no icon for now)
        let mut tray = TrayItem::new("Escluse Agent", IconSource::Resource("escluse-agent"))?;
        
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();
        
        // Add menu items
        tray.add_menu_item("Show Status", move || {
            show_status_window();
        })?;
        
        tray.add_menu_item("Open Config Folder", move || {
            open_config_folder();
        })?;
        
        tray.add_menu_item("Restart Agent", move || {
            // Signal agent to restart
            println!("Restart requested");
        })?;
        
        tray.add_menu_item("Exit", move || {
            running_clone.store(false, Ordering::SeqCst);
        })?;
        
        // Inner tray menu separator (not supported in this version)
        
        Ok(GuiApp { _tray: tray, running })
    }
    
    pub fn run(&self) {
        // Wait until user clicks Exit
        while self.running.load(Ordering::SeqCst) {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}

#[cfg(target_os = "windows")]
fn show_status_window() {
    // Simple message box for now
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    
    let title_wide: Vec<u16> = OsStr::new("Escluse Agent Status").encode_wide().chain(Some(0)).collect();
    let message = format!(
        "Escluse Agent is running in the background.\n\n\
        Status: Active\n\
        Backend: wss://app.escluse.com/api/ws/node\n\n\
        The agent will continue running even after closing this window.\n\
        Use the system tray icon to control the agent."
    );
    let message_wide: Vec<u16> = OsStr::new(&message).encode_wide().chain(Some(0)).collect();
    
    unsafe {
        winapi::um::winuser::MessageBoxW(
            std::ptr::null_mut(),
            message_wide.as_ptr(),
            title_wide.as_ptr(),
            winapi::um::winuser::MB_OK | winapi::um::winuser::MB_ICONINFORMATION,
        );
    }
}

#[cfg(target_os = "windows")]
fn open_config_folder() {
    use std::process::Command;
    
    if let Some(app_data) = std::env::var_os("APPDATA") {
        let config_path = std::path::PathBuf::from(&app_data).join("escluse-agent");
        let _ = Command::new("explorer")
            .arg(config_path)
            .spawn();
    }
}

#[cfg(not(target_os = "windows"))]
pub struct GuiApp;

#[cfg(not(target_os = "windows"))]
impl GuiApp {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(GuiApp)
    }
    
    pub fn run(&self) {
        // No-op on non-Windows
    }
}

// Run GUI mode - spawn agent in background and run GUI
#[cfg(target_os = "windows")]
pub async fn run_gui_mode(agent_fut: std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send>>) -> anyhow::Result<()> {
    use tokio::task;
    
    // Spawn agent in background task
    let agent_handle = task::spawn(async move {
        agent_fut.await
    });
    
    // Show startup notification
    show_notification("Escluse Agent", "Agent started successfully and running in the background.\n\nUse the system tray icon to check status.");
    
    // Run GUI in main thread (this blocks)
    let gui = GuiApp::new()?;
    gui.run();
    
    // User clicked Exit - abort agent
    agent_handle.abort();
    
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub async fn run_gui_mode(agent_fut: std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send>>) -> anyhow::Result<()> {
    // On non-Windows, just run the agent directly
    agent_fut.await
}

#[cfg(target_os = "windows")]
pub fn show_notification(title: &str, message: &str) {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    
    let title_wide: Vec<u16> = OsStr::new(title).encode_wide().chain(Some(0)).collect();
    let message_wide: Vec<u16> = OsStr::new(message).encode_wide().chain(Some(0)).collect();
    
    unsafe {
        winapi::um::winuser::MessageBoxW(
            std::ptr::null_mut(),
            message_wide.as_ptr(),
            title_wide.as_ptr(),
            winapi::um::winuser::MB_OK | winapi::um::winuser::MB_ICONINFORMATION,
        );
    }
}

#[cfg(not(target_os = "windows"))]
fn show_notification(_title: &str, _message: &str) {
    // No-op on non-Windows
}
