mod osu_format;
mod osu_detect;

use std::{fs, io};
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

    if songs_path.exists() 
    {
        match iterate_songs(songs_path)
        {
            Ok(_) => { println!("Successfully parsed Osu! directory.")},
            Err(err) => { panic!("Failed to parse you Osu! directory, error: {}", err)}
        }
    }
    else
    {
        panic!("We found an Osu! directory, but it does not contain a Songs folder. Do you have a partial installation?");
    }

    let time = Instant::now().saturating_duration_since(start).as_secs_f32();
    println!("Execution time: {} seconds", time);
}

fn recurse_directory(
    path: PathBuf, 
    predicate: impl Fn(PathBuf) -> bool, 
    callback: impl Fn(PathBuf) -> Result<(), io::Error>
) -> Result<(), io::Error>
{
    for entry in fs::read_dir(path)?
    {
        let entry: fs::DirEntry = entry?;
        let path = entry.path();
        
        if predicate(path.clone()) {
            callback(path.clone())?;
        }
    }

    Ok(())
}

fn iterate_songs(songs_folder: PathBuf) -> Result<(), io::Error>
{
    recurse_directory(songs_folder,
        | path | { path.exists() && path.is_dir() },
        | path | { evaluate_song(path) }
    )
}

fn evaluate_song(song_path: PathBuf) -> Result<(), io::Error>
{
    iterate_song_files(song_path)
}

fn iterate_song_files(song_path: PathBuf) -> Result<(), io::Error>
{
    recurse_directory(song_path,
        | path | { path.exists() && path.is_file() },
        | path | { evaluate_song_files(path) }
    )
}

fn evaluate_song_files(song_file_path: PathBuf) -> Result<(), io::Error>
{
    let extension = song_file_path.extension();

    if let Some(ext) = extension 
    {
        let ext_str: &str = ext.to_str().unwrap();

        if ext_str.contains("osu")
        {
            let osu_file: osu_format::OsuFile = read_osu_file(song_file_path);
        }
    }

    Ok(())
}

fn read_osu_file(osu_path: PathBuf) -> osu_format::OsuFile
{
    let mut file = osu_format::OsuFile::new();
    file.parse(osu_path);
    return file;
}