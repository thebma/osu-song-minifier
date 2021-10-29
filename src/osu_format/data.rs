use std::str::FromStr;
use half::{ f16 };

///
/// General todos, fixes and pain points for this file:
/// 
/// 
///
/// 
/// 
/// 

#[repr(u32)] #[derive(Clone, Debug)]
pub enum OsuFileGamemode
{
    Unknown = u32::MAX,
    Osu = 0,
    Taiko = 1,
    Catch = 2,
    Mania = 3
}

impl Default for OsuFileGamemode
{
    fn default() -> Self { OsuFileGamemode::Osu }
}

impl OsuFileGamemode
{
    pub fn from_u32(integer: u32) -> OsuFileGamemode
    {
        match integer
        {
            0 => OsuFileGamemode::Osu,
            1 => OsuFileGamemode::Taiko,
            2 => OsuFileGamemode::Catch,
            3 => OsuFileGamemode::Mania,
            _ => OsuFileGamemode::Unknown
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum OsuFileSampleSet
{
    Normal,
    Soft,
    Drum,
    None,
}

impl Default for OsuFileSampleSet 
{
    fn default() -> Self { OsuFileSampleSet::Normal }
}

impl FromStr for OsuFileSampleSet
{
    type Err = String;

    fn from_str(input: &str) -> Result<OsuFileSampleSet, Self::Err> 
    {
        match input.to_ascii_lowercase().as_str() 
        {
            "normal" => Ok(OsuFileSampleSet::Normal),
            "soft" => Ok(OsuFileSampleSet::Soft),
            "drum" => Ok(OsuFileSampleSet::Drum),
            "none" => Ok(OsuFileSampleSet::Normal),
            _ => Ok(OsuFileSampleSet::None)
        }
    }
}

#[derive(Clone, Debug)]
pub enum OsuFileOverlayPosition
{
    NoChange, 
    Below, 
    Above
}

impl FromStr for OsuFileOverlayPosition
{
    type Err = String;

    fn from_str(input: &str) -> Result<OsuFileOverlayPosition, Self::Err>
    {
        match input.to_ascii_lowercase().as_str() 
        {
            "nochange" => Ok(OsuFileOverlayPosition::NoChange),
            "below" => Ok(OsuFileOverlayPosition::Below),
            "above" => Ok(OsuFileOverlayPosition::Above),
            _ => Err(format!("Cannot convert {} to an OFSampleSet enum.", input)),
        }
    }
}

impl Default for OsuFileOverlayPosition
{
    fn default() -> Self { OsuFileOverlayPosition::NoChange }
}

#[derive(Default, Clone, Debug)]
pub struct OsuFileGeneral 
{
    pub audio_file_name: String,
    pub audio_lead_in: i32,
    pub preview_time: i32,
    pub countdown: u32,
    pub sample_set: OsuFileSampleSet,
    pub stack_leniency: f32,
    pub mode: OsuFileGamemode,
    pub letterbox_in_breaks: bool,
    pub use_skin_sprites: bool,
    pub skin_preference: String,
    pub overlay_position: OsuFileOverlayPosition,
    pub epilepsy_warning: bool,
    pub countdown_offset: u32,
    pub special_style: bool,
    pub widescreen_storyboard: bool,
    pub samples_match_playback_rate: bool
}

#[derive(Default, Clone, Debug)]
pub struct OsuFileEditor 
{
    pub bookmarks: String, //TODO: Make this a comma seperated value class.
    pub distance_spacing: f32,
    pub beat_divisor: f32,
    pub grid_size: u32,
    pub timeline_zoom: f32
}

#[derive(Default, Clone, Debug)]
pub struct OsuFileMetadata 
{
    pub title: String,
    pub title_unicode: String,
    pub artist: String,
    pub artist_unicode: String,
    pub creator: String,
    pub version: String,
    pub source: String,
    pub tags: String, //TODO: Make this a comma seperated value class.
    pub beatmap_id: i64,
    pub beatmap_set_id: i64,
}

#[derive(Default, Clone, Debug)]
pub struct OsuFileDifficulty 
{
    pub hp_drain_rate: f16,
    pub circle_size: f16,
    pub overall_difficulty: f16,
    pub approach_rate: f16,
    pub slider_multiplier: f16,
    pub slider_tick_rate: f16,
}

#[derive(Default, Clone, Debug)]
pub struct OsuFileVideo
{
    pub exists: bool,
    pub start_time: i32,
    pub file_name: String,
    pub x_offset: i32,
    pub y_offset: i32
}

#[derive(Default, Clone, Debug)]
pub struct OsuFileBackground 
{
    pub exists: bool,
    pub file_name: String,
    pub x_offset: i32,
    pub y_offset: i32,
}

#[derive(Default, Clone, Debug)]
pub struct OsuFileBreakPeriod
{
    pub start: u32,
    pub end: u32,
}

#[derive(Default, Clone, Debug)]
pub struct OsuFileEvents 
{
    pub background: OsuFileBackground,
    pub video: OsuFileVideo,
    pub breaks: Vec<OsuFileBreakPeriod>,
    //TODO: Do the storyboard crap.
}

#[derive(Default, Clone, Debug)]
pub struct OsuFileTimingPoints 
{
    //TODO: Parse timinig points.
}

#[derive(Default, Clone, Debug)]
pub struct OsuFileColors 
{
    pub combo_colors: Vec<OsuFileCombo>,
    pub slider_border: OsuFileColor,
    pub slider_track_override: OsuFileColor,
}

#[derive(Default, Clone, Debug)]
pub struct OsuFileCombo
{
    pub index: i8,
    pub color: OsuFileColor
}

#[derive(Default, Clone, Debug)]
pub struct OsuFileColor
{
    pub red: u8,
    pub blue: u8,
    pub green: u8
}

impl OsuFileColor
{
    pub fn from_str(input: String) -> Result<OsuFileColor, String>
    {
        let rgb: Vec<&str> = input.split(",").collect();

        if rgb.len() != 3 
        {
            return Err(String::from("Given string does not represent a color."))
        }

        Ok(OsuFileColor { 
            red: rgb[0].parse::<u8>().unwrap(), 
            green: rgb[1].parse::<u8>().unwrap(), 
            blue: rgb[2].parse::<u8>().unwrap()
        })
    }
}

#[derive(Default, Clone, Debug)]
pub struct OsuFileHitObjects  
{
    //TODO: Parse hit objects.
}

//TODO: Support deprecated variables, turns out some old map still have them.
//TODO: Specialize some of the structs, instead of using generic collections.
#[derive(Default, Clone, Debug)]
pub struct OsuFile
{
    pub version: String,
    pub is_valid: bool,
    pub general_section: OsuFileGeneral,
    pub editor_section: OsuFileEditor,
    pub metadata_section: OsuFileMetadata,
    pub difficulty_section: OsuFileDifficulty,
    pub events_section: OsuFileEvents,
    pub timing_points_section: OsuFileTimingPoints,
    pub colours_section: OsuFileColors,
    pub hit_object_section: OsuFileHitObjects
}
