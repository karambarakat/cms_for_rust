//! daemon is responsible for running the subprocess in dev mode, recieving commands from the admin panel, that
//! will include code modification command, providing information about communicating with the
//! subprocess, and authinticating in dev mode
use std::{
    io::{BufRead, BufReader, Write},
    process::{ChildStdin, Command, Stdio},
    sync::Mutex,
};

lazy_static::lazy_static! {
    static ref STD_IN: Mutex<Option<ChildStdin>> = Mutex::new(None);
    static ref EVENTS: Mutex<Vec<Event>> = Mutex::new(Vec::new());
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "data")]
pub enum Event {
    Panic(String),
    Ready,
}

pub fn main() {
    let child_shell = Command::new("/bin/sh")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .env("SECRET", "hello world")
        .spawn()
        .unwrap();

    let mut child_in = child_shell.stdin.unwrap();

    #[cfg(not(test))]
    child_in.write(b"cd client\n").unwrap();
    #[cfg(test)]
    child_in.write(b"cd ../client\n").unwrap();

    child_in
        .write(
            format!(
                "cargo run | echo EXECUTABLE_MANAGER {}\n",
                serde_json::to_string(&Event::Panic(
                    "unkown".to_string()
                ))
                .unwrap()
                .replace("\"", "\\\"")
            )
            .as_bytes(),
        )
        .unwrap();

    STD_IN.lock().unwrap().replace(child_in);

    let mut reader = BufReader::new(child_shell.stdout.unwrap());

    let mut out = String::new();
    loop {
        out.clear();
        reader.read_line(&mut out).unwrap();

        if out.starts_with("EXECUTABLE_MANAGER ") {
            let line =
                out.strip_prefix("EXECUTABLE_MANAGER ").unwrap();

            let event =
                serde_json::from_str::<Event>(line).unwrap();

            EVENTS.lock().unwrap().push(event);
        } else {
            print!("supprocess: {out}");
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_main() {}
}
