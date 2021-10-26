mod osu_format;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Instant};

/*
    General todo's for this application:
    - Properly handle Option<> / Error handling on IO errors.
      Unwrapping and praying for the best is the current approach, which arguebly is not the most bulletproof :)

    - Automatically scan for the installation location for Osu!.
      The .osu file handler should contain the path.

    - Optionally zip the entire cleaned directory for passing around.
    - Deduplicate the directory indexing.
    - Do general testing and see how my code breaks.
*/
fn main() 
{
    let start = Instant::now(); 
    let mut root: String = String::new();

    //detect: HKEY_CLASSES_ROOT\osu\shell\open\command
    if true { //if laptop...
        root = String::from("M:\\Games\\Osu");
    }
    else {
        root = String::from("D:\\Games\\Osu");
    }
    
    
    let songs_path: PathBuf = Path::new(&root).join("Songs");
    if songs_path.exists() {
        recurse_songs(songs_path).unwrap()
    }

    let time = Instant::now().saturating_duration_since(start).as_secs_f32();
    println!("Execution time: {} seconds", time);
}

fn recurse_songs(songs_folder: PathBuf) -> Result<(), io::Error>
{
    for entry in fs::read_dir(songs_folder)?
    {
        let entry: fs::DirEntry = entry?;
        let path = entry.path();

        if path.is_dir() 
        {
            process_song(entry.path())?;
        }
    }

    Ok(())
}

fn process_song(song_path: PathBuf) -> Result<(), io::Error>
{
    for entry in fs::read_dir(song_path)? 
    {
        let entry: fs::DirEntry = entry?;
        let path: PathBuf = entry.path();

        if path.is_file()
        {
            let extension = path.extension();

            if let Some(ext) = extension 
            {
                let ext_str: &str = ext.to_str().unwrap();

                if ext_str.contains("osu")
                {
                    //println!("evaluating: {:?}", path);
                    find_critical_files(path).unwrap();
                }
            }
        }
    }

    Ok(())
}


///
/// Find critical files to keep within a song directory.
/// osu_path is the .osu file with in a song directory, this contains the metadata what the background and audio files are.
/// Returns a list of files to keep.
/// 
fn find_critical_files(osu_path: PathBuf)  -> Result<Vec<String>, io::Error>
{
    let mut files_to_keep: Vec<String> = Vec::new();
    let mut file = osu_format::OsuFile::new();
    file.parse(osu_path);

    println!("Audio file is {}", file.general_section.audio_file_name);
    println!("Background file is {:?}", file.events_section.background);
    println!("Video file is {:?}", file.events_section.video);

    Ok(files_to_keep)
}