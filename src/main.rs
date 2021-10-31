mod osu_format;
mod osu_detect;

use std::{fs, io};
use std::path::{Path, PathBuf};
use std::time::{Instant};

use osu_format::data::OsuFile;
use osu_format::data::OsuFileConfig;

/*
    General todo's for this application:
    - Properly handle Option<> / Error handling on IO errors.
      Unwrapping and praying for the best is the current approach, which arguebly is not the most bulletproof :)
        Whilest generalizing the iteration code, we nuked all the Result<> structures, an even worse idea.
        Might wanna test this throughougly!
    - Specify modes for "cleaning"...
        "Destructive" => Delete the files directly from your Songs folder.
        "Copy" => Create a new directory with the files.
        "Zip" => Same as copy, but outputs a zip.
    - Handle when no Osu! installation was found.
        Perhaps even offer an manual way of configuring Osu! installation path.
    - Multi-threaded beatmap processing.
    - Filter out non-Osu! gamemodes (taiko, ctb).
*/
#[derive(Default)]
pub struct ShadowTransaction
{
    from: PathBuf,
    to: PathBuf
}

#[tokio::main]
async fn main() 
{
    let start = Instant::now(); 
    let mut root: String = String::new();

    match osu_detect::where_is_osu() 
    {
        Ok(v) => { root = v; },
        Err(_) => { println!("Unable to locate Osu! install path."); }
    };
    
    let osu_path: PathBuf = Path::new(&root).to_path_buf();
    let songs_path: PathBuf = osu_path.join("Songs");

    if songs_path.exists() 
    {
        match iterate_songs(osu_path, songs_path).await
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

fn recurse_directory(path: PathBuf, predicate: impl Fn(PathBuf) -> bool) -> Vec<PathBuf>
{
    let mut found: Vec<PathBuf> = Vec::new();

    if let Ok(entries) = fs::read_dir(path) 
    {
        for entry in entries 
        {
            if let Ok(entry) = entry 
            {
                let path = entry.path();

                if predicate(path.clone()) 
                {
                    found.push(path.clone());
                }
            }
        }
    }

    return found;
}

async fn iterate_songs(osu_path: PathBuf, songs_folder: PathBuf) -> Result<(), io::Error>
{
    let songs = recurse_directory(songs_folder, | path | { path.exists() && path.is_dir() });
    let mut transactions: Vec<ShadowTransaction> = Vec::new();

    for song in songs 
    {
        evaluate_song(&mut transactions, osu_path.clone(), song)?;
    }

    //perform_transactions(&transactions).await;
    Ok(())
}

fn evaluate_song(transactions: &mut Vec<ShadowTransaction>, osu_path: PathBuf, song_path: PathBuf) -> Result<(), io::Error>
{
    println!("Parsing song: {:?}", song_path);
    Ok(iterate_song_files(transactions, osu_path, song_path)?)
}

fn iterate_song_files(transactions: &mut Vec<ShadowTransaction>, osu_path: PathBuf, song_path: PathBuf) -> Result<(), io::Error>
{
    let path = song_path.clone();
    let mut keep: Vec<PathBuf> = Vec::new();

    let files: Vec<PathBuf> = recurse_directory(song_path, |path| { path.exists() && path.is_file() });

    for file in files 
    {
        keep.extend(evaluate_song_files(path.clone(), file));
    }

    keep.sort();
    keep.dedup();

    save_transaction(transactions, osu_path, path, keep);
    Ok(())
}
    
fn evaluate_song_files(song_path: PathBuf, song_file_path: PathBuf) -> Vec<PathBuf>
{
    let song_file_clone = song_file_path.clone();
    let song_path_clone = song_path.clone();
    let extension = song_file_clone.extension();
    
    if let Some(ext) = extension 
    {
        let ext_str: &str = ext.to_str().unwrap();

        if !ext_str.contains("osu")
        {
            return Vec::new();
        }
        
        let mut paths_to_keep: Vec<PathBuf> = Vec::new();
        let mut osu_file: OsuFile = OsuFile::new();
        osu_file.parse(song_file_path, OsuFileConfig {
            parse_colours: false,
            parse_difficulty: false,
            parse_editor: false,
            parse_metadata: false,
            ..Default::default()
        });

        if !osu_file.is_valid 
        {
            return Vec::new();
        }

        paths_to_keep.push(song_file_clone);

        if osu_file.events_section.background.exists 
        {
            let background = Path::new(&song_path_clone).join(osu_file.events_section.background.file_name).clone();
            paths_to_keep.push(background);
        }

        let audio = Path::new(&song_path_clone).join(osu_file.general_section.audio_file_name);
        paths_to_keep.push(audio);

        return paths_to_keep;
    }

    return Vec::new();
}

fn strip_option(opt: Option<&str>) -> String
{
    if let Some(stripped) = opt 
    {
        return stripped.to_owned();
    }

    return String::new();
}

fn save_transaction(transactions: &mut Vec<ShadowTransaction>, osu_path: PathBuf, song_folder: PathBuf, keep: Vec<PathBuf>)
{
    let shadow = Path::new(&osu_path).join("Shadow");
    let osu = Path::new(&osu_path).join("Songs");

    let shadow_str: String = strip_option(shadow.to_str());
    let osu_str: String = strip_option(osu.to_str());
    let song_str: String = strip_option(song_folder.to_str());

    let new_str = song_str.replace(&osu_str, &shadow_str);
    let new_path = Path::new(&new_str);

    if !new_path.exists() 
    {
        match fs::create_dir_all(new_path)
        {
            Ok(_) => {},
            Err(e) => { println!("Failed to create shadow directory: {}", e)}
        }
    }

    for file in keep 
    {
        let mut file_str = strip_option(file.to_str());
        file_str = file_str.replace(&osu_str, &shadow_str);
        let file_path = Path::new(&file_str);

        transactions.push(ShadowTransaction {
            from: file,
            to: file_path.to_path_buf()
        });
    }
}

async fn perform_transactions(transactions: &Vec<ShadowTransaction>)
{
    for transaction in transactions
    {
        let mut to_path: PathBuf = Path::new(&transaction.to).to_path_buf();
        to_path.pop();

        tokio::fs::create_dir_all(&to_path).await.unwrap();

        if Path::new(&transaction.from).exists()
        {
            println!("from {:?} to {:?}", transaction.from, transaction.to);
            tokio::fs::copy(&transaction.from, &transaction.to).await.unwrap();
        }
    }
}