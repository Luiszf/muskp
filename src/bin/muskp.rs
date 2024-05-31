use anyhow::{Context, Ok};
use clap::{Arg, Command, Parser};
use dialoguer::theme::Theme;
use dialoguer::FuzzySelect;
use glob::glob;
use std::io::{BufReader, Read};
use std::os::unix::net::{SocketAddr, UnixListener};
use std::path::Display;
use std::sync::atomic::AtomicBool;
use std::thread;
use std::{io::Write, os::unix::net::UnixStream};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let c = Command::new("muskp")
        .subcommand(Command::new("play"))
        .subcommand(Command::new("search"))
        .subcommand(Command::new("pause"))
        .subcommand(Command::new("exit"))
        .subcommand(Command::new("list"))
        .subcommand(Command::new("next"))
        .get_matches();

    let socket_path = "/tmp/muskp";

    let mut socket = UnixStream::connect(socket_path).context("could not connect")?;

    match c.subcommand_name() {
        Some("play") => {
            socket.write(b"play").context("server did not responded")?;
        }
        Some("pause") => {
            socket.write(b"pause").context("server did not responded")?;
        }
        Some("exit") => {
            socket.write(b"exit").context("server did not responded")?;
        }
        Some("list") => {
            socket.write(b"list").context("server did not responded")?;
            socket.shutdown(std::net::Shutdown::Write);

            let mut messege = String::new();

            socket.read_to_string(&mut messege);

            println!("{}", messege)
        }
        Some("next") => {
            socket.write(b"next").context("server did not responded")?;
        }
        Some("search") => {
            let mut musics = Vec::new();
            for entry in glob("/home/luis/Music/**/*.mp3").expect("Failed to read glob pattern") {
                match entry {
                    std::result::Result::Ok(path) => musics.push(path),
                    Err(e) => println!("{:?}", e),
                }
            }
            let musics: Vec<Display> = musics.iter_mut().map(|x| x.display()).collect();

            let selection = FuzzySelect::new()
                .with_prompt("What do you choose?")
                .items(&musics)
                .interact()
                .unwrap();

            println!("{}", musics[selection]);

            let mut selection_mes = String::new();
            selection_mes.push_str("path");
            selection_mes.push_str(musics[selection].to_string().as_str());

            socket
                .write(selection_mes.as_bytes())
                .context("server did not responded")?;
        }
        _ => {
            println!("not valid command")
        }
    }
    Ok(())
}
