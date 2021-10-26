mod osu_format;
mod osu_detect;

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Instant};

/*
    General todo's for this application:
    - Properly handle Option<> / Error handling on IO errors.
      Unwrapping and praying for the best is the current approach, which arguebly is not the most bulletproof :)
        Whilest generalizing the iteration code, we nuked all the Result<> structures, an even worse idea.
        Might wanna test this throughougly!
    - Optionally zip the entire cleaned directory for passing around.
        Having this option would be nice if someone is not willing to sacrifice his Songs folder.
*/
fn main() 
{
    let start = Instant::now(); 
    let mut root: String = String::new();

    match osu_detect::where_is_osu() {
        Ok(v) => { root = v; },
        Err(_) => { println!("Unable to locate Osu! install path."); }
    };
    
    let songs_path: PathBuf = Path::new(&root).join("Songs");

    if songs_path.exists() {
        iterate_songs(songs_path);
    }

    let time = Instant::now().saturating_duration_since(start).as_secs_f32();
    println!("Execution time: {} seconds", time);
}

fn recurse_directory(path: PathBuf, predicate: impl Fn(PathBuf) -> bool, callback: impl Fn(PathBuf))
{
    for entry in fs::read_dir(path).unwrap()
    {
        let entry: fs::DirEntry = entry.unwrap();
        let path = entry.path();
        
        if predicate(path.clone()) {
            callback(path.clone());
        }
    }
}

fn iterate_songs(songs_folder: PathBuf)
{
    recurse_directory(songs_folder,
        | path | { path.exists() && path.is_dir() },
        | path | { evaluate_song(path); }
    );
}

fn evaluate_song(song_path: PathBuf)
{
    iterate_song_files(song_path);
}

fn iterate_song_files(song_path: PathBuf)
{
    recurse_directory(song_path,
        | path | { path.exists() && path.is_file() },
        | path | { evaluate_song_files(path); }
    );
}

fn evaluate_song_files(song_file_path: PathBuf)
{
    let extension = song_file_path.extension();

    if let Some(ext) = extension 
    {
        let ext_str: &str = ext.to_str().unwrap();

        if ext_str.contains("osu")
        {
            find_critical_files(song_file_path);
        }
    }
}

///
/// Find critical files to keep within a song directory.
/// osu_path is the .osu file with in a song directory, this contains the metadata what the background and audio files are.
/// Returns a list of files to keep.
/// 
fn find_critical_files(osu_path: PathBuf)
{
    let mut files_to_keep: Vec<String> = Vec::new();
    let mut file = osu_format::OsuFile::new();
    file.parse(osu_path);

    println!("Audio file is {}", file.general_section.audio_file_name);
    println!("Background file is {:?}", file.events_section.background);
    println!("Video file is {:?}", file.events_section.video);
}