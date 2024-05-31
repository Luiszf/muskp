use anyhow::{Context, Ok};
use dialoguer::theme::Theme;
use std::io::{BufReader, Read};
use std::os::unix::net::{SocketAddr, UnixListener};
use std::sync::atomic::AtomicBool;
use std::thread;
use std::{io::Write, os::unix::net::UnixStream};

use clap::{Arg, Command, Parser};
use dialoguer::FuzzySelect;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let c = Command::new("muskp")
        .subcommand(
            Command::new("play"),
        )
        .subcommand(
            Command::new("search").arg(Arg::new("name").short('n').action(clap::ArgAction::Append)),
        )
        .subcommand(Command::new("pause"))
        .subcommand(Command::new("exit"))
        .subcommand(Command::new("list"))
        .subcommand(Command::new("next"))
        .get_matches();


    let socket_path = "musicsocket";

    let mut socket = UnixStream::connect(socket_path).context("could not connect")?;

    match c.subcommand_name() {
        Some("play") => {
            socket
                .write(b"play")
                .context("server did not responded")?;
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
            let query = c
                .subcommand_matches("search")
                .unwrap()
                .get_one::<String>("name")
                .ok_or("muito trint")
                .expect("file not provided");

            let mut message = String::new();
            message.push_str("search");
            message.push_str(query);

            socket
                .write(message.as_bytes())
                .context("server did not responded")?;
            
            socket.shutdown(std::net::Shutdown::Write);
            
            let mut response = String::new();
            socket.read_to_string(&mut response).context("maneiro");
            
            let list: Vec<&str> = response.split("\n").collect();
            
            let selection = FuzzySelect::new()
                .with_prompt("What do you choose?")
                .items(&list)
                .interact()
                .unwrap();

            println!("{}", list[selection]);
            
            let mut selection_mes = String::new();
            selection_mes.push_str("path");
            selection_mes.push_str(list[selection]);

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
