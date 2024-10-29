use std::io::{BufReader, BufWriter, Read, Write};

use proc_macro2::Span;
use quote::ToTokens;
use syn::{visit_mut::VisitMut, FieldMutability, File};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet() -> Result<String, ()> {
    // print current directory
    let mut cwd = std::env::current_dir().unwrap();

    cwd.push("admin_example");
    cwd.push("entities.rs");

    // read file
    let file = std::fs::File::open(cwd.clone()).unwrap();

    let mut read = BufReader::new(file.try_clone().unwrap());
    let mut contents = String::new();
    read.read_to_string(&mut contents).unwrap();
    let contents = contents.to_token_stream();

    let mut parsed =
        syn::parse::<File>(contents.into()).unwrap();

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

    let mut greet = Greet;
    greet.visit_file_mut(&mut parsed);

    let mut write = BufWriter::new(file);
    write
        .write_all(
            parsed.to_token_stream().to_string().as_bytes(),
        )
        .unwrap();

    parsed.to_token_stream().to_string();

    Ok(format!("Hello, Current directory is: {:?}", cwd))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
