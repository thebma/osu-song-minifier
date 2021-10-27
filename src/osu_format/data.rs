use std::str::FromStr;

#[repr(u32)] #[derive(Clone, Debug)]
pub enum OFGameMode
{
    Unknown = u32::MAX,
    Osu = 0,
    Taiko = 1,
    Catch = 2,
    Mania = 3
}

impl Default for OFGameMode
{
    fn default() -> Self { OFGameMode::Osu }
}

impl OFGameMode
{
    pub fn from_u32(integer: u32) -> OFGameMode
    {
        match integer
        {
            0 => OFGameMode::Osu,
            1 => OFGameMode::Taiko,
            2 => OFGameMode::Catch,
            3 => OFGameMode::Mania,
            _ => OFGameMode::Unknown
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum OFSampleSet
{
    Normal,
    Soft,
    Drum,
    None,
}

impl Default for OFSampleSet 
{
    fn default() -> Self { OFSampleSet::Normal }
}

impl FromStr for OFSampleSet
{
    type Err = String;

    fn from_str(input: &str) -> Result<OFSampleSet, Self::Err> 
    {
        match input.to_ascii_lowercase().as_str() 
        {
            "normal" => Ok(OFSampleSet::Normal),
            "soft" => Ok(OFSampleSet::Soft),
            "drum" => Ok(OFSampleSet::Drum),
            "none" => Ok(OFSampleSet::Normal),
            _ => Ok(OFSampleSet::None)
        }
    }
}

#[derive(Clone, Debug)]
pub enum OFOverlayPosition
{
    NoChange, 
    Below, 
    Above
}

impl FromStr for OFOverlayPosition
{
    type Err = String;

    fn from_str(input: &str) -> Result<OFOverlayPosition, Self::Err>
    {
        match input.to_ascii_lowercase().as_str() 
        {
            "nochange" => Ok(OFOverlayPosition::NoChange),
            "below" => Ok(OFOverlayPosition::Below),
            "above" => Ok(OFOverlayPosition::Above),
            _ => Err(format!("Cannot convert {} to an OFSampleSet enum.", input)),
        }
    }
}

impl Default for OFOverlayPosition
{
    fn default() -> Self { OFOverlayPosition::NoChange }
}

#[derive(Default, Clone, Debug)]
pub struct OFSectionGeneral 
{
    pub audio_file_name: String,
    pub audio_lead_in: i32,
    pub preview_time: i32,
    pub countdown: u32,
    pub sample_set: OFSampleSet,
    pub stack_leniency: f32,
    pub mode: OFGameMode,
    pub letterbox_in_breaks: bool,
    pub use_skin_sprites: bool,
    pub skin_preference: String,
    pub overlay_position: OFOverlayPosition,
    pub epilepsy_warning: bool,
    pub countdown_offset: u32,
    pub special_style: bool,
    pub widescreen_storyboard: bool,
    pub samples_match_playback_rate: bool
}

#[derive(Default, Clone, Debug)]
pub struct OFSectionEditor 
{
    pub bookmarks: String,
    pub distance_spacing: f32,
    pub beat_divisor: f32,
    pub grid_size: u32,
    pub timeline_zoom: f32
}

#[derive(Default, Clone, Debug)]
pub struct OFSectionMetadata 
{
    pub title: String,
    pub title_unicode: String,
    pub artist: String,
    pub artist_unicode: String,
    pub creator: String,
    pub version: String,
    pub source: String,
    pub tags: String, //TODO: Make this a OFSectionTagsCollection.
    pub beatmap_id: i64,
    pub beatmap_set_id: i64,
}

#[derive(Default, Clone, Debug)]
pub struct OFSectionDifficulty 
{
    //TODO: 32-bit float might be excessive, in-game it's represented as a 0 to 10 decimal with 2 decimal places.
    //      Half (f16) of even quaters (f8) would do the job and would require subsequent crates to be imported.
    pub hp_drain_rate: f32,
    pub circle_size: f32,
    pub overall_difficulty: f32,
    pub approach_rate: f32,
    pub slider_multiplier: f32,
    pub slider_tick_rate: f32,
}

#[derive(Default, Clone, Debug)]
pub struct OFSectionVideo
{
    pub exists: bool,
    pub start_time: i32,
    pub file_name: String,
    pub x_offset: i32,
    pub y_offset: i32
}

#[derive(Default, Clone, Debug)]
pub struct OFSectionBackground 
{
    pub exists: bool,
    pub file_name: String,
    pub x_offset: i32,
    pub y_offset: i32,
}

#[derive(Default, Clone, Debug)]
pub struct OFSectionBreakPeriod
{
    pub start: u32,
    pub end: u32,
}

#[derive(Default, Clone, Debug)]
pub struct OFSectionEvents 
{
    pub background: OFSectionBackground,
    pub video: OFSectionVideo,
    pub breaks: Vec<OFSectionBreakPeriod>,
    //TODO: Do the storyboard crap.
}

#[derive(Default, Clone, Debug)]
pub struct OFSectionTimingPoints 
{
    //TODO: Parse timinig points.
}

#[derive(Default, Clone, Debug)]
pub struct OFSectionColour
{
    pub index: i8,
    pub red: u8,
    pub blue: u8,
    pub green: u8
}

impl OFSectionColour 
{
    pub fn from_str(input: String, index: i8) -> Result<OFSectionColour, String>
    {
        //TODO: Find a better way to assign indices, generalize this behaviour being independant from "index".
        let rgb: Vec<&str> = input.split(",").collect();

        if rgb.len() != 3 
        {
            return Err(String::from("string did not have the rgb component(s)"))
        }

        Ok(OFSectionColour { 
            index: index,
            red: rgb[0].parse::<u8>().unwrap(), 
            green: rgb[1].parse::<u8>().unwrap(), 
            blue: rgb[2].parse::<u8>().unwrap()
        })
    }
}

#[derive(Default, Clone, Debug)]
pub struct OFSectionColours 
{
    pub colours: Vec<OFSectionColour>,
    pub slider_border: OFSectionColour,
    pub slider_track_override: OFSectionColour,
    pub my_life_is_meaning_less: String
}

#[derive(Default, Clone, Debug)]
pub struct OFSectionHitObjects  
{
    //TODO: Parse hit objects.
}

//TODO: Support deprecated variables.
//TODO: Specialize some of the structs, instead of using generic collections.
#[derive(Default, Clone, Debug)]
pub struct OsuFile
{
    pub version: String,
    pub general_section: OFSectionGeneral,
    pub editor_section: OFSectionEditor,
    pub metadata_section: OFSectionMetadata,
    pub difficulty_section: OFSectionDifficulty,
    pub events_section: OFSectionEvents,
    pub timing_points_section: OFSectionTimingPoints,
    pub colours_section: OFSectionColours,
    pub hit_object_section: OFSectionHitObjects
}
