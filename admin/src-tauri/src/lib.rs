use proc_macro2::Span;
use syn::FieldMutability;

macro_rules! handle_errors {
    ($body:expr) => {{
        use std::panic::{catch_unwind, set_hook};
        use std::io::{Write};
        use syn::{visit_mut::VisitMut, File};
        use quote::ToTokens;

        set_hook(Box::new(|pi| {
            let any = pi.payload();
            let mut str = None;
            if let Some(s) = any.downcast_ref::<String>() {
                str = Some(s.clone())
            } else if let Some(s) = any.downcast_ref::<&str>() {
                str = Some(s.to_string())
            };
            eprintln!(
                "error\n{}{}",
                str.unwrap_or_default(),
                pi.location()
                    .map(|loc| {
                        format!(
                            "\npanic occurred in file '{}' at line {}",
                            loc.file(),
                            loc.line()
                        )
                    })
                    .unwrap_or_default()
            );
        }));

        catch_unwind(|| $body).map_err(|_| ())
    }};
}

#[tauri::command]
fn greet() -> Result<String, ()> {
    handle_errors!({
        let mut cwd = std::env::current_dir().unwrap();
        cwd.pop();
        cwd.pop();
        cwd.push("admin_example");
        cwd.push("src");
        cwd.push("entities.rs");
        let contents =
            std::fs::read_to_string(cwd.clone()).unwrap();
        //     //(cwd.clone()).unwrap();
        // let mut contents = String::new();
        // file.read_to_string(&mut contents).unwrap();
        let mut parsed =
            syn::parse_str::<File>(&contents).unwrap();
        struct Greet;

        impl VisitMut for Greet {
            fn visit_fields_named_mut(
                &mut self,
                i: &mut syn::FieldsNamed,
            ) {
                i.named.push(syn::Field {
                    attrs: Vec::new(),
                    vis: syn::Visibility::Inherited,
                    mutability: FieldMutability::None,
                    ident: Some(syn::Ident::new(
                        "new_field",
                        Span::call_site(),
                    )),
                    colon_token: Some(Default::default()),
                    ty: syn::parse_quote!(String),
                });
            }
        }
        Greet.visit_file_mut(&mut parsed);
        let unpsarsed = prettyplease::unparse(&parsed);

        std::fs::write(cwd.clone(), unpsarsed.as_bytes())
            .unwrap();

        format!(
            "Hello, Current directory is: {:?}\n{}",
            cwd, unpsarsed
        )
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
