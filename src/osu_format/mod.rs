pub mod data;

use std::io::{BufRead, BufReader};
use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;

use data::{
    OsuFile, 
    OFSectionBackground, OFSectionColour, OFSectionVideo,
    OFSampleSet, OFGameMode, OFOverlayPosition
};

///
/// General todo's for this file:
/// - Ensure safe conversion of string -> i8/u8/u32/i32/f32/enum.
///   Move this to a util file, to deduplictate the code and some extra for error handling.
/// - Use Result(T, R) for functions instead of passing an referenced error variable.
///   Perhaps we need to resort to unions, explore options to see what is easier (i.e. nested structs vs. union)
/// - Generalize the parse_* functions, deduplicate the code.
/// - Break this file up into 2 other files: osu_format_data.rs and osu_format_util (for BOM stuff).
/// - Get rid of the horrible "OsuFilesSection" aka "OFSection" prefixes.
/// 
impl OsuFile
{
    pub fn new() -> OsuFile
    {
        OsuFile::default()
    }

    fn parse_version(&mut self, line: String) -> Result<(), String>
    {
        if !line.contains("osu file format") 
        {
            return Err("File does not contain a version number.".to_owned());
        }

        self.version = line.trim().to_owned();
        Ok(())
    }

    fn get_key_value(&mut self, line: String) -> (bool, String, String)
    {
        let kvp: Vec<&str> = line.split(':').collect();
        let mut key: String = String::new();
        let mut value: String = String::new();
        let mut valid: bool = false;

        if kvp.len() == 2
        {
            key = kvp[0].trim().to_owned();
            value = kvp[1].trim().to_owned();
            valid = true;
        }
        else if kvp.len() > 2
        {
            key = kvp[0].trim().to_owned();

            let kvp_len = kvp.len();
            let kvp_slice: Vec<&str> = kvp.into_iter().skip(1).take(kvp_len-1).collect();
            value = kvp_slice.join(":");
            valid = true;
        }
        
        return (valid, key, value); 
    }

    fn parse_general(&mut self, line: String) -> Result<(), String>
    {
        let (found, key, value) = self.get_key_value(line.clone());

        if !found
        {
            return Err(format!("invalid key-value pair for line {} in section Editor", line));
        }

        let mut section = self.general_section.clone();

        //TODO: generalize these functions.
        let as_u32 = || -> u32 { value.parse::<u32>().unwrap() };
        let as_i32 = || -> i32 { value.parse::<i32>().unwrap() };
        let as_f32 = || -> f32 { value.parse::<f32>().unwrap() };
        let as_bool = || -> bool { if value == "1"  { true } else { false } };
        let as_sample_set = || -> OFSampleSet { OFSampleSet::from_str(&value).unwrap() };
        let as_game_mode = || -> OFGameMode { OFGameMode::from_u32(as_u32()) };
        let as_overlay = || -> OFOverlayPosition { OFOverlayPosition::from_str(&value).unwrap()};

        match key.as_ref()
        {
            "AudioFilename" => { section.audio_file_name = value },
            "AudioLeadIn" => { section.audio_lead_in = as_i32() },
            "PreviewTime" => { section.preview_time = as_i32(); },
            "Countdown" => { section.countdown = as_u32(); },
            "SampleSet" => { section.sample_set = as_sample_set(); },
            "StackLeniency" => { section.stack_leniency = as_f32(); },
            "Mode" => { section.mode = as_game_mode(); },
            "LetterboxInBreaks" => { section.letterbox_in_breaks = as_bool(); },
            "UseSkinSprites" => { section.use_skin_sprites = as_bool(); },
            "OverlayPosition" => { section.overlay_position = as_overlay(); },
            "SkinPreference" => { section.skin_preference = value },
            "EpilepsyWarning" => { section.epilepsy_warning = as_bool() },
            "CountdownOffset" => { section.countdown_offset = as_u32()},
            "SpecialStyle" => { section.special_style = as_bool()},
            "WidescreenStoryboard" => { section.widescreen_storyboard = as_bool() },
            "SamplesMatchPlaybackRate" => { section.samples_match_playback_rate = as_bool() },
            _ => { println!("Unknown field {} inside general section with value: {}", key, value); }
        }

        self.general_section = section;

        Ok(())
    }

    fn parse_editor(&mut self, line: String) -> Result<(), String>
    {
        let (found, key, value) = self.get_key_value(line.clone());

        if !found
        {
            return Err(format!("invalid key-value pair for line {} in section Editor", line));
        }

        let mut section = self.editor_section.clone();

        //TODO: generalize these functions.
        let as_u32 = || -> u32 { value.parse::<u32>().unwrap() };
        let as_f32 = || -> f32 { value.parse::<f32>().unwrap() };

        match key.as_ref()
        {
            "Bookmarks" => { section.bookmarks = value; },
            "DistanceSpacing" => { section.distance_spacing = as_f32(); },
            "BeatDivisor" => { section.beat_divisor = as_f32(); },
            "GridSize" => { section.grid_size = as_u32(); },
            "TimelineZoom" => { section.timeline_zoom = as_f32(); }
            _ => { println!("Unknown field {} inside editor section with value: {}", key, value); }
        }

        self.editor_section = section;
        Ok(())
    }

    fn parse_metadata(&mut self, line: String) -> Result<(), String>
    {
        let (found, key, value) = self.get_key_value(line.clone());
        
        if !found
        {
            return Err(format!("invalid key-value pair for line {} in section Metadata", line))
        }
        
        let mut section = self.metadata_section.clone();

        //TODO: generalize these functions.
        let as_i64 = || -> i64 { value.parse::<i64>().unwrap() };
        
        match key.as_ref()
        {
            "Title" => { section.title = value; },
            "TitleUnicode" => { section.title_unicode = value},
            "Artist" => { section.artist = value },
            "ArtistUnicode" => { section.artist_unicode = value },
            "Creator" => { section.creator = value },
            "Version" => { section.version = value; },
            "Source" => { section.source = value; },
            "Tags" => { section.tags = value },
            "BeatmapID" => { section.beatmap_id= as_i64(); },
            "BeatmapSetID" => { section.beatmap_set_id = as_i64(); },
            _ => { println!("Unknown field {} inside metadata section with value: {}", key, value); }
        }

        self.metadata_section = section;
        Ok(())
    }

    fn parse_difficulty(&mut self, line: String) -> Result<(), String>
    {
        let (found, key, value) = self.get_key_value(line.clone());
        
        if !found
        {
            return Err(format!("invalid key-value pair for line {} in section Difficulty", line));
        }
        
        let mut section = self.difficulty_section.clone();
   
        //TODO: generalize these functions.
        let as_f32 = || -> f32 { value.parse::<f32>().unwrap() };

        match key.as_ref()
        {
            "HPDrainRate" => { section.hp_drain_rate = as_f32(); },
            "CircleSize" => { section.circle_size = as_f32(); },
            "OverallDifficulty" => { section.overall_difficulty = as_f32(); },
            "ApproachRate" => { section.approach_rate = as_f32(); },
            "SliderMultiplier" => { section.slider_multiplier = as_f32(); },
            "SliderTickRate" => { section.slider_tick_rate= as_f32(); },
            _ => { println!("Unknown field {} inside difficulty section with value: {}", key, value); }
        }

        self.difficulty_section = section;
        Ok(())
    }

    fn parse_events(&mut self, line: String) -> Result<(), String>
    {
        let line_split: Vec<&str> = line.split(",").collect();

        if line_split.len() == 0
        {
            return Err(format!("no csv string provided, got: {}", line));
        }

        let event_type = line_split[0];
        let mut section = self.events_section.clone();

        if line_split.len() >= 3 && (event_type == "0" || event_type == "Background") 
        {
            let file = line_split[2].to_owned().replace("\"", "");
            let x_offset = if line_split.len() == 4 { line_split[3].parse::<i32>().unwrap() } else { 0 };
            let y_offset = if line_split.len() == 5 { line_split[4].parse::<i32>().unwrap() } else { 0 };
            
            section.background = OFSectionBackground {
                exists: true, 
                file_name: file,
                x_offset: x_offset,
                y_offset: y_offset
            };  
        }

        if line_split.len() >= 3 && (event_type == "1" || event_type == "Video")
        {
            let start_time = line_split[1].parse::<i32>().unwrap();
            let file = line_split[2].to_owned().replace("\"", "");
            let x_offset = if line_split.len() == 4 { line_split[3].parse::<i32>().unwrap() } else { 0 };
            let y_offset = if line_split.len() == 5 { line_split[4].parse::<i32>().unwrap() } else { 0 };
            
            section.video = OFSectionVideo {
                exists: true, 
                start_time: start_time,
                file_name: file,
                x_offset: x_offset,
                y_offset: y_offset
            };  
        }

        //TODO: parse storyboard.
        self.events_section = section; 
        Ok(())
    }

    fn parse_timing_points(&self, line: String) -> Result<(), String>
    {
        //TODO: parse hit objects.
        Ok(())
    }

    fn parse_colours(&mut self, line: String) -> Result<(), String>
    {
        let (found, key, value) = self.get_key_value(line.clone());

        if !found
        {
            return Err(format!("invalid key-value pair for line {} in section Colour", line))
        }

        let mut section = self.colours_section.clone();

        if key.starts_with("Combo")
        {
            let num: String = key.replace("Combo", "");
            let index = num.parse::<i8>().unwrap();

            match OFSectionColour::from_str(value, index) {
                Ok(v) => { section.colours.push(v); },
                Err(e) => { println!("Converting value from string failed: {}", e)}
            }
        }
        else if key.starts_with("SliderBorder")
        {
            section.slider_border = OFSectionColour::from_str(value, -1).unwrap();
        }
        else if key.starts_with("SliderTrackOverride")
        {
            section.slider_track_override = OFSectionColour::from_str(value, -1).unwrap();
        }
        else if key.starts_with("MyLifeIsMeaningless")
        {
            //NOTE: A random tid-bit I stumbled upon on in one of the ~7500 beatmaps.
            //      from this beatmap: https://osu.ppy.sh/beatmapsets/1357481#osu/2809324.
            section.my_life_is_meaning_less = "Hey cheer up Myahkey, life is worth living for.".to_owned();
        }
        else
        {
            println!("Unhandled key-value pair in Colours, key: {}, value: {}", key, value);
        }

        self.colours_section = section;
        Ok(())
    }

    fn parse_hit_objects(&self, line: String) -> Result<(), String>
    {
        Ok(())
        //TODO: Parse hit objects.
        //println!("Handle hit objects...")
    }

    pub fn parse(&mut self, file: PathBuf)
    {
        let path = file.clone();
        println!("Parsing .osu file: {:?}", path);

        let osu_file = File::open(file)
            .expect("Failed reading .osu file.");
        
        let file_reader: BufReader<File> = BufReader::new(osu_file);
        let mut context: String = String::new();

        for line_wrap in file_reader.lines()
        {
            let line: String = line_wrap.unwrap().clone();
            let line_copy = line.clone();

            if line.trim().is_empty() || line.starts_with("//")
            {
                continue;
            }

            //Adjust the current context if needed.
            if line.starts_with("[") && line.ends_with("]") 
            {
                let heading: String = line
                    .chars()
                    .skip(1)
                    .take_while(|c| match c {
                        ']' => false,
                        _ => true
                    })
                    .collect();

                if heading.is_empty() { continue; };
                context = heading.to_lowercase();
            }
            else 
            {
                let result: Result<(), String> = match context.as_ref()
                {
                    "" => self.parse_version(line),
                    "general" => self.parse_general(line),
                    "editor" => self.parse_editor(line),
                    "metadata" => self.parse_metadata(line),
                    "difficulty" => self.parse_difficulty(line),
                    "events" => self.parse_events(line),
                    "timingpoints" => self.parse_timing_points(line),
                    "colours" => self.parse_colours(line),
                    "hitobjects" => self.parse_hit_objects(line),
                    _ => Err(format!("Context {} was not handled.", context), )
                };

                match result {
                    Err(err) => { println!("Failed to parse for with error: {}\n\tContext {}\n\tValue {}", err, context, line_copy); }
                    _ => {},
                };
            }
        }
    }
}