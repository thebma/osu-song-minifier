pub mod data;

use std::io::{BufRead, BufReader};
use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;
use half::{ f16 };

use data::{
    OsuFile,
    OsuFileBackground,
    OsuFileCombo,
    OsuFileColor,
    OsuFileVideo,
    OsuFileSampleSet,
    OsuFileGamemode,
    OsuFileOverlayPosition
};

///
/// General todo's for this file:
/// - Ensure safe conversion OsuFile string -> i8/u8/u32/i32/f32/enum.
///   Move this to a util file, to deduplictate the code and some extra for error handling.
/// - Perhaps we need to resort to unions, explore options to see what is easier (i.e. nested structs vs. union)
/// 
impl OsuFile
{
    pub fn new() -> OsuFile
    {
        OsuFile::default()
    }

    fn match_kvp(&self, line: String) -> Result<(String, String), String>
    {        
        let kvp: Vec<&str> = line.split(':').collect();
        let kvp_len = kvp.len();

        if kvp_len == 2
        {
            let key = kvp[0].trim().to_owned();
            let value = kvp[1].trim().to_owned();
            Ok((key, value))
        }
        else if kvp_len > 2
        {
            let key = kvp[0].trim().to_owned();
            let kvp_slice: Vec<&str> = kvp.into_iter().skip(1).take(kvp_len - 1).collect();
            let value = kvp_slice.join(":");
            Ok((key, value))
        }
        else
        {
            let err = format!("invalid key-value pair on value: {} ", line);
            Err(err)
        }
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

    fn parse_general(&mut self, line: String) -> Result<(), String>
    {
        let kvp = self.match_kvp(line);

        if let Ok((key, value)) = kvp
        {
            let mut section = self.general_section.clone();

            //TODO: generalize these functions.
            let as_u32 = || -> u32 { value.parse::<u32>().unwrap() };
            let as_i32 = || -> i32 { value.parse::<i32>().unwrap() };
            let as_f32 = || -> f32 { value.parse::<f32>().unwrap() };
            let as_bool = || -> bool { if value == "1"  { true } else { false } };
            let as_sample_set = || -> OsuFileSampleSet { OsuFileSampleSet::from_str(&value).unwrap() };
            let as_game_mode = || -> OsuFileGamemode { OsuFileGamemode::from_u32(as_u32()) };
            let as_overlay = || -> OsuFileOverlayPosition { OsuFileOverlayPosition::from_str(&value).unwrap()};

            match key.as_ref() {
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
        }
        else if let Err(error) = kvp 
        {
            return Err(error);
        }

        Ok(())
    }

    fn parse_editor(&mut self, line: String) -> Result<(), String>
    {
        let kvp = self.match_kvp(line);

        if let Ok((key, value)) = kvp
        {
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
        }        
        else if let Err(error) = kvp 
        {
            return Err(error);
        }

        Ok(())
    }
    
    fn parse_metadata(&mut self, line: String) -> Result<(), String>
    {
        let kvp = self.match_kvp(line);

        if let Ok((key, value)) = kvp
        {
            let mut section = self.metadata_section.clone();

            // //TODO: generalize these functions.
            let as_i64 = |v: String| -> i64 { v.parse::<i64>().unwrap() };

            match key.as_ref() {
                "Title" => { section.title = value },
                "TitleUnicode" => { section.title_unicode =  value },
                "Artist" => { section.artist = value },
                "ArtistUnicode" => { section.artist_unicode = value },
                "Creator" => { section.creator = value },
                "Version" => { section.version = value },
                "Source" => { section.source =  value },
                "Tags" => { section.tags = value},
                "BeatmapID" => { section.beatmap_id= as_i64(value); },
                "BeatmapSetID" => { section.beatmap_set_id = as_i64(value); },
                _ => { println!("Unknown field {} inside metadata section with value: {}", "", value); }
            }

            self.metadata_section = section;
        }
        else if let Err(error) = kvp 
        {
            return Err(error);
        }

        Ok(())
    }

    fn parse_difficulty(&mut self, line: String) -> Result<(), String>
    {
        let kvp = self.match_kvp(line);

        if let Ok((key, value)) = kvp
        {
            let mut section = self.difficulty_section.clone();
            
            //TODO: generalize these functions.
            let as_f16 = || -> f16 { f16::from_f32(value.parse::<f32>().unwrap()) };
            
            match key.as_ref()
            {
                "HPDrainRate" => { section.hp_drain_rate = as_f16(); },
                "CircleSize" => { section.circle_size = as_f16(); },
                "OverallDifficulty" => { section.overall_difficulty = as_f16(); },
                "ApproachRate" => { section.approach_rate = as_f16(); },
                "SliderMultiplier" => { section.slider_multiplier = as_f16(); },
                "SliderTickRate" => { section.slider_tick_rate= as_f16(); },
                _ => { println!("Unknown field {} inside difficulty section with value: {}", key, value); }
            }

            self.difficulty_section = section;
        }
        else if let Err(error) = kvp 
        {
            return Err(error);
        }

        Ok(())
    }

    //TODO: This would require it's own functions like match_kvp, but then for match_array. As Events, TimingPoints and HitObjects use this.
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
            
            section.background = OsuFileBackground {
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
            
            section.video = OsuFileVideo {
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
        let kvp = self.match_kvp(line);

        if let Ok((key, value)) = kvp
        {
            let mut section = self.colours_section.clone();

            if key.starts_with("Combo")
            {
                let num: String = key.replace("Combo", "");
                let index = num.parse::<i8>().unwrap();

                if let Ok(color) = OsuFileColor::from_str(value)
                {
                    section.combo_colors.push(OsuFileCombo { 
                        index: index, color: color
                    });
                }
            }
            else if key.starts_with("SliderBorder")
            {
                section.slider_border = OsuFileColor::from_str(value).unwrap();
            }
            else if key.starts_with("SliderTrackOverride")
            {
                section.slider_track_override = OsuFileColor::from_str(value).unwrap();
            }
            else
            {
                println!("Unhandled key-value pair in Colours, key: {}, value: {}", key, value);
            }

            self.colours_section = section;
        }
        else if let Err(error) = kvp 
        {
            return Err(error);
        }

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