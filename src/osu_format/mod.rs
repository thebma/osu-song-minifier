pub mod data;

use std::io::{BufRead, BufReader};
use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;
use half::{ f16 };

use data::{
    OsuFile,
    OsuFileConfig,
    OsuFileBackground,
    OsuFileCombo,
    OsuFileColor,
    OsuFileVideo,
    OsuFileSampleSet,
    OsuFileGamemode,
    OsuFileTimingPoint,
    OsuFileHitObject,
    OsuFileOverlayPosition,
    OsuFileEditorBookmarks,
    OsuFileMetadataTags,
    CsvValue
};

///
/// General todo's for this file:
/// - Ensure safe conversion OsuFile string -> i8/u8/u32/i32/f32/enum.
///   Move this to a util file, to deduplictate the code and some extra for error handling.
/// - Parse storyboard elements for Events section.
/// - Parse the "Effects" field of Timingpoint, require some bitwise magic.
/// 
/// Long term
/// - Support non utf-8 files.
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
            Ok((key.to_lowercase(), value))
        }
        else if kvp_len > 2
        {
            let key = kvp[0].trim().to_owned();
            let kvp_slice: Vec<&str> = kvp.into_iter().skip(1).take(kvp_len - 1).collect();
            let value = kvp_slice.join(":");
            Ok((key.to_lowercase(), value))
        }
        else
        {
            let err = format!("invalid key-value pair on value: {} ", line);
            Err(err)
        }
    }

    fn match_csv(&self, line: String) -> Result<Vec<CsvValue>, String>
    {
        let csv: Vec<&str> = line.split(",").collect();

        if csv.len() <= 0 
        {
            return Err("no values inside csv.".to_owned());
        }

        let mut values: Vec<CsvValue> = Vec::new();

        for csv_part in csv
        {   
            values.push(CsvValue { value: csv_part.to_owned() });
        }

        Ok(values)
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

            match key.as_ref() 
            {
                "audiofilename" => { section.audio_file_name = value },
                "audioleadin" => { section.audio_lead_in = as_i32() },
                "previewtime" => { section.preview_time = as_i32(); },
                "countdown" => { section.countdown = as_u32(); },
                "sampleset" => { section.sample_set = as_sample_set(); },
                "stackleniency" => { section.stack_leniency = as_f32(); },
                "mode" => { section.mode = as_game_mode(); },
                "letterboxinbreaks" => { section.letterbox_in_breaks = as_bool(); },
                "useskinsprites" => { section.use_skin_sprites = as_bool(); },
                "overlayposition" => { section.overlay_position = as_overlay(); },
                "skinpreference" => { section.skin_preference = value },
                "epilepsywarning" => { section.epilepsy_warning = as_bool() },
                "countdownoffset" => { section.countdown_offset = as_u32()},
                "specialstyle" => { section.special_style = as_bool()},
                "widescreenstoryboard" => { section.widescreen_storyboard = as_bool() },
                "samplesmatchplaybackrate" => { section.samples_match_playback_rate = as_bool() },
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
            let as_u32 = || -> u32 { value.parse::<u32>().unwrap() };
            let as_f32 = || -> f32 { value.parse::<f32>().unwrap() };

            match key.as_ref()
            {
                "bookmarks" => { section.bookmarks = OsuFileEditorBookmarks::from_str(&value).unwrap() },
                "distancespacing" => { section.distance_spacing = as_f32(); },
                "beatdivisor" => { section.beat_divisor = as_f32(); },
                "gridsize" => { section.grid_size = as_u32(); },
                "timelinezoom" => { section.timeline_zoom = as_f32(); }
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
            let as_i64 = |v: String| -> i64 { v.parse::<i64>().unwrap() };

            match key.as_ref() 
            {
                "title" => { section.title = value },
                "titleunicode" => { section.title_unicode =  value },
                "artist" => { section.artist = value },
                "artistunicode" => { section.artist_unicode = value },
                "creator" => { section.creator = value },
                "version" => { section.version = value },
                "source" => { section.source =  value },
                "tags" => { section.tags = OsuFileMetadataTags::from_str(&value).unwrap(); },
                "beatmapid" => { section.beatmap_id= as_i64(value); },
                "beatmapsetid" => { section.beatmap_set_id = as_i64(value); },
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
            let as_f16 = || -> f16 { f16::from_f32(value.parse::<f32>().unwrap()) };
            
            match key.as_ref()
            {
                "hpdrainrate" => { section.hp_drain_rate = as_f16(); },
                "circlesize" => { section.circle_size = as_f16(); },
                "overalldifficulty" => { section.overall_difficulty = as_f16(); },
                "approachrate" => { section.approach_rate = as_f16(); },
                "slidermultiplier" => { section.slider_multiplier = as_f16(); },
                "slidertickrate" => { section.slider_tick_rate= as_f16(); },
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

    fn parse_events(&mut self, line: String) -> Result<(), String>
    {
        let line_split: Vec<&str> = line.split(",").collect();

        if line_split.len() == 0
        {
            return Err(format!("no csv string provided, got: {}", line));
        }

        let mut section = self.events_section.clone();
        let event_type = line_split[0];

        if line_split.len() >= 3 && (event_type == "0" || event_type == "Background") 
        {
            let file = line_split[2].to_owned().replace("\"", "");
            let x_offset = if line_split.len() == 4 { line_split[3].parse::<i32>().unwrap() } else { 0 };
            let y_offset = if line_split.len() == 5 { line_split[4].parse::<i32>().unwrap() } else { 0 };
            
            section.background = OsuFileBackground 
            {
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
            
            section.video = OsuFileVideo 
            {
                exists: true, 
                start_time: start_time,
                file_name: file,
                x_offset: x_offset,
                y_offset: y_offset
            };  
        }

        self.events_section = section; 
        Ok(())
    }

    fn parse_timing_points(&mut self, line: String) -> Result<(), String>
    {
        let mut section = self.timing_points_section.clone();
        let csv_match_result = self.match_csv(line);

        if let Ok(csv_match) = csv_match_result
        {
            let mut index: u32 = 0;
            let mut timing_point: OsuFileTimingPoint = OsuFileTimingPoint { ..Default::default() }; 

            for csv in csv_match
            {
                match index
                {
                    0 => { timing_point.time = csv.to_f32() },
                    1 => { timing_point.beat_length = csv.to_f32() },
                    2 => { timing_point.meter = csv.to_i32() },
                    3 => { timing_point.sample_set = OsuFileSampleSet::from_u32(csv.to_u32()) },
                    4 => { timing_point.sample_index = csv.to_i32(); },
                    5 => { timing_point.volume = csv.to_i32(); },
                    6 => { timing_point.uninherited = csv.to_bool() },
                    7 => { timing_point.effects = csv.to_i32() }
                    _ => { }
                }

                index += 1;
            }

            section.timing_points.push(timing_point);
        }

        self.timing_points_section = section;
        Ok(())
    }

    fn parse_colours(&mut self, line: String) -> Result<(), String>
    {
        let kvp = self.match_kvp(line);

        if let Ok((key, value)) = kvp
        {
            let mut section = self.colours_section.clone();

            if key.starts_with("combo")
            {
                let num: String = key.replace("combo", "");
                let index = num.parse::<i8>().unwrap();

                if let Ok(color) = OsuFileColor::from_str(value)
                {
                    section.combo_colors.push(OsuFileCombo { index: index, color: color });
                }
            }
            else if key.starts_with("sliderborder")
            {
                section.slider_border = OsuFileColor::from_str(value).unwrap();
            }
            else if key.starts_with("slidertrackoverride")
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

    fn parse_hit_objects(&mut self, line: String) -> Result<(), String>
    {
        let mut section = self.hit_object_section.clone();
        let csv_match_result = self.match_csv(line);

        if let Ok(csv_match) = csv_match_result
        {
            let mut index: u32 = 0;
            let mut hit_object: OsuFileHitObject = OsuFileHitObject { ..Default::default() }; 

            for csv in csv_match
            {
                match index
                {
                    0 => { hit_object.x = csv.to_i32() },
                    1 => { hit_object.y = csv.to_i32() },
                    2 => { hit_object.time = csv.to_i32() },
                    3 => { hit_object.hit_sound= csv.to_u8() },
                    4 => { hit_object.params = csv.value },
                    5 => { hit_object.hit_sample = csv.value },
                    _ => { }
                }

                index += 1;
            }

            section.hit_objects.push(hit_object);
        }

        self.hit_object_section = section;
        Ok(())
    }

    pub fn parse(&mut self, file: PathBuf, config: OsuFileConfig)
    {
        let osu_file = File::open(file)
            .expect("Failed reading .osu file.");
        
        let file_reader: BufReader<File> = BufReader::new(osu_file);
        let mut context: String = String::new();

        self.is_valid = true;

        let no_op = || -> Result<(), String> { Ok(()) };

        for line_wrap in file_reader.lines()
        {
            if !self.is_valid 
            { 
                break; 
            }

            let mut line: String = String::new();

            if let Err(_) = line_wrap 
            {
                continue;
            }
            else if let Ok(unwrapped_line) = line_wrap 
            {
                line = unwrapped_line;
            }
            
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
                    .take_while(|c| 
                        match c 
                        {
                            ']' => false,
                            _ => true
                        }
                    )
                    .collect();

                if heading.is_empty() { continue; };
                context = heading.to_lowercase();
            }
            else 
            {
                //TODO: This is really clunky and hard to read, can we handle this in the function we're calling?
                let result: Result<(), String> = match context.as_ref()
                {
                    "" => self.parse_version(line),
                    "general" => if config.parse_general { self.parse_general(line) } else { no_op() },
                    "editor" => if config.parse_editor { self.parse_editor(line) } else { no_op() },
                    "metadata" => if config.parse_metadata { self.parse_metadata(line) } else { no_op() },
                    "difficulty" => if config.parse_difficulty { self.parse_difficulty(line) } else { no_op() },
                    "events" => if config.parse_events { self.parse_events(line) } else { no_op() },
                    "timingpoints" => if config.parse_timing_points { self.parse_timing_points(line) } else { no_op() },
                    "colours" => if config.parse_colours { self.parse_colours(line) } else { no_op() },
                    "hitobjects" => if config.parse_hit_objects { self.parse_hit_objects(line) } else { no_op() },
                    _ => Err(format!("Context {} was not handled.", context), )
                };


                match result 
                {
                    Err(err) => 
                    {
                        //NOTE: If we parsed the file and found that the version is incorrect....
                        //      Then this is a faulty file to begin with.
                        if context == "" 
                        {
                            println!("Tried parsing an invalid file, error: {}", err); 
                            self.is_valid = false;
                        }
                        else
                        {
                            println!("Failed to parse line for with error: {}\n\tContext {}\n\tValue {}", err, context, line_copy); 
                        }
                    }
                    _ => { },
                };
            }
        }
    }
}