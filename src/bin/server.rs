use anyhow::Context;

use glob::glob;
use rand::{seq::SliceRandom, thread_rng};
use rodio::{cpal::Stream, Decoder, OutputStream, Sink};
use std::{
    fs::File,
    future,
    io::{BufReader, Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
    sync::{Arc, Mutex},
    thread::{self, sleep},
    time::Duration,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let socket_path = "musicsocket";

    if std::fs::metadata(socket_path).is_ok() {
        println!("A socket is already present. Deleting...");
        std::fs::remove_file(socket_path)
            .with_context(|| format!("could not delete previous socket at {:?}", socket_path))?;
    }

    let mut musics = Arc::new(Mutex::new(Vec::new()));
    for entry in glob("/home/luis/Music/**/*.mp3").expect("Failed to read glob pattern") {
        match entry {
            std::result::Result::Ok(path) => musics.lock().unwrap().push(path),
            Err(e) => println!("{:?}", e),
        }
    }
    let mut rng = thread_rng();

    let mut count: Arc<usize> = 0.into();

    musics.lock().unwrap().shuffle(&mut rng);

    let unix_listener =
        UnixListener::bind(socket_path).context("Could not create the unix socket")?;

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    loop {
        let (unix_stream, _socket_address) = unix_listener
            .accept()
            .context("Failed at accepting a connection on the unix listener")?;
    let sink: Sink = Sink::try_new(&stream_handle).unwrap();
        let musics = musics.clone();
        let count = count.clone();
        let path = &musics.lock().unwrap()[0];
        let file = BufReader::new(File::open(path).unwrap());
        let source = Decoder::new(file).unwrap();
        sink.append(source);

        //tokio::spawn(async move{
        //handle_stream(unix_stream, &sink, musics, count).await;
        //});

    }
}

async fn handle_stream(
    mut unix_stream: UnixStream,
    sink: &Sink,
    musics: Arc<Mutex<Vec<PathBuf>>>,
    count: Arc<usize>,
) -> anyhow::Result<()> {
    let mut message = String::new();

    println!("chegou aqui oh");

    unix_stream
        .read_to_string(&mut message)
        .context("null message")?;

    if message.starts_with("play") {
        println!("playing...");

        if sink.is_paused() {
            sink.play();
        }

        let path = &musics.lock().unwrap()[*count];
        let file = BufReader::new(File::open(path).unwrap());
        let source = Decoder::new(file).unwrap();
        sink.append(source);
    }

    if message.starts_with("pause") {
        println!("pausing...");
        if sink.is_paused() {
            sink.play();
        } else {
            sink.pause();
        }
    }
    if message.starts_with("exit") {
        println!("exiting...");
        sink.clear();
    }
    if message.starts_with("list") {
        println!("listing...");

        //let mut list_names = String::new();
        //let mut list = Vec::with_capacity(10);
        //let x = musics.lock().unwrap().to_vec();

        //for i in 0..9 {
        //    list.push(x[i])
        //}

        //for i in list {
        //    list_names.push_str(i.display().to_string().as_str());
        //    list_names.push_str("\n")
        //}
        //unix_stream
        //    .write(list_names.as_bytes())
        //    .context("failed to print list")?;
    }
    if message.starts_with("search") {
        println!("searching...");

        let arg = message.split_off("search".len());
        let mut list = String::new();

        let _matches: Vec<PathBuf> = musics.lock().unwrap()
            .to_vec()
            .into_iter()          
            .filter(|x| {
                list.push_str(x.display().to_string().as_str());
                list.push_str("\n");
                x.display()
                    .to_string()
                    .to_lowercase()
                    .contains(arg.as_str())
            })
            .collect();

        unix_stream
            .write(list.as_bytes())
            .context("could not connect with client")?;
    }
    if message.starts_with("path") {
        println!("chegou aqui legal");

        message.split_off("path".len());

        let path = PathBuf::from(message.clone());

        println!("{}", path.display());

        musics.lock().unwrap().insert(*count, path)
    }
    if message.starts_with("next") {
        println!("next...");
        sink.skip_one()
    }

    Ok(())
}
async fn play_song(sink: &Sink) -> &Sink {
        print!("{}", sink.len());
        sink
    }
