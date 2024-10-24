// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet() -> Result<String, ()> {
    // print current directory
    let mut cwd = std::env::current_dir().unwrap();

    cwd.push("admin_example");

    Ok(format!(
        "Hello, Current directory is: {:?}", cwd
    ))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
