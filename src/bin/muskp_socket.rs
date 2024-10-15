use anyhow::{Context, Ok};

use glob::glob;
use rand::{seq::SliceRandom, thread_rng};
use rodio::{OutputStream, Sink, Source, Decoder};
use std::{
    fmt::Debug,
    fs::File,
    io::{BufReader, Read},
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
    sync::Arc,
};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let socket_path = "/tmp/muskp";

    if std::fs::metadata(socket_path).is_ok() {
        println!("A socket is already present. Deleting...");
        std::fs::remove_file(socket_path)
            .with_context(|| format!("could not delete previous socket at {:?}", socket_path))?;
    }

    let mut musics = Vec::new();
    for entry in glob("/home/luis/Music/**/*.mp3").expect("Failed to read glob pattern") {
        match entry {
            std::result::Result::Ok(path) => musics.push(path),
            Err(e) => println!("{:?}", e),
        }
    }
    let mut rng = thread_rng();

    musics.shuffle(&mut rng);

    let musics = Arc::new(musics);
    let unix_listener =
        UnixListener::bind(socket_path).context("Could not create the unix socket")?;

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Arc::new(Sink::try_new(&stream_handle).unwrap());

    for stream in unix_listener.incoming() {
        match stream {
            std::result::Result::Ok(stream) => {
                let sink = Arc::clone(&sink);
                let mut musics = Arc::clone(&musics);
                let t = tokio::spawn(async move { handle_stream(stream) });
                let mut option = t.await.unwrap();
                let _p = tokio::spawn(async move {
                    for i in 0..musics.len() {
                        let sink = Arc::clone(&sink);
                        let mut musics = Arc::clone(&musics);

                        if option == "list" {
                            for i in 0..musics.len() {
                                let song = musics[i].display();
                                println!("index: {i} Music: {song}")
                            }
                        }

                        if option == "pause" {
                            if sink.is_paused() {
                                sink.play()
                            } else {
                                sink.pause()
                            }
                            break;
                        }
                        if option == "next" {
                            sink.skip_one();
                            break;
                        }
                        if option.starts_with("path") {
                            let path_str = option.split_off("path".len());
                            let mut path = PathBuf::from(path_str.trim());
                            play_song(&sink, &path);
                            break;
                        }

                        println!("Index:{i}");
                        play_song(&sink, &musics[i])
                    }
                });
            }
            Err(_err) => {}
        }
    }
    Ok(())
}
fn play_song(sink: &Sink, path: &PathBuf) {
    let file = BufReader::new(File::open(path).unwrap());
    let source = Decoder::new(file).unwrap();
    println!("{}", path.display());
    sink.append(source);
    sink.sleep_until_end()
}

fn handle_stream(mut unix_stream: UnixStream) -> String {
    let mut message = String::new();

    unix_stream.read_to_string(&mut message).unwrap();

    if message.starts_with("play") {
        println!("playing...");
    }
    if message.starts_with("pause") {
        println!("pausing...");
    }
    if message.starts_with("list") {
        println!("listing...");
    }
    if message.starts_with("path") {
        println!("playing next...");
    }
    if message.starts_with("next") {
        println!("next...");
    }
    return message;
}
