use anyhow::{Context, Ok};

use glob::glob;
use rand::{seq::SliceRandom, thread_rng};
use rodio::{cpal::Stream, decoder, Decoder, OutputStream, Sink, Source};
use std::{
    fs::File, future, io::{BufReader, Read, Write}, os::unix::net::{UnixListener, UnixStream}, path::PathBuf, sync::{Arc, BarrierWaitResult, Mutex}, thread::{self, sleep}, time::Duration
};
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

    let mut count: usize = 0;

    musics.shuffle(&mut rng);

    let musics = Arc::new(musics);
    let unix_listener =
        UnixListener::bind(socket_path).context("Could not create the unix socket")?;

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Arc::new(Sink::try_new(&stream_handle).unwrap());

    unix_listener
        .set_nonblocking(true)
        .expect("could not set non blocking socket");

    for stream in unix_listener.incoming() {
        match stream {
            std::result::Result::Ok(stream) => {
                let sink = Arc::clone(&sink);
                let musics = Arc::clone(&musics);
                let t = thread::spawn(|| 
                    handle_stream(stream)
                );
                let mut option = t.join().unwrap();
                let p = thread::spawn(move || loop {
                    let sink = Arc::clone(&sink);
                    let musics = Arc::clone(&musics);
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
                        let mut path = PathBuf::new();
                        path.push(path_str);
                        play_song(&sink, &path);
                        break;
                    }
                    play_song(&sink, &musics[count]);
                    count += 1
                }).join().unwrap();
            }
            Err(err) => {}
        }
    }
    println!("sera que chega aqui");
    Ok(())
}
fn play_song(sink: &Sink, path: &PathBuf) {
    let file = BufReader::new(File::open(path).unwrap());
    let source = Decoder::new(file).unwrap();
    let duration = &source.total_duration().unwrap();
    println!("{}", path.display());
    sink.append(source);

    sleep(*duration);
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
    if message.starts_with("exit") {
        println!("exiting...");
    }
    if message.starts_with("list") {
        println!("listing...");
    }
    if message.starts_with("search") {
        println!("searching...");
    }
    if message.starts_with("path") {
        println!("chegou aqui legal");
    }
    if message.starts_with("next") {
        println!("next...");
    }
    return message;
}
