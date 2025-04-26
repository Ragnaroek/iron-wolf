use core::str;
use std::collections::HashMap;
use std::io::Cursor;

use serde::{Deserialize, Serialize};

use crate::def::{Assets, IWConfig, WeaponType, derive_from};
use crate::gamedata;
use crate::loader::Loader;
use crate::map::{MapFileType, MapSegs, MapType, load_map, load_map_headers, load_map_offsets};
use crate::patch::{PatchConfig, graphic_patch};
use crate::sd::Sound;
use crate::util::new_data_reader;

pub static GAMEPAL: &'static [u8] = include_bytes!("../assets/gamepal.bin");
pub static SIGNON: &'static [u8] = include_bytes!("../assets/signon.bin");

pub const GRAPHIC_DICT: &'static str = "VGADICT";
pub const GRAPHIC_HEAD: &'static str = "VGAHEAD";
pub const GRAPHIC_DATA: &'static str = "VGAGRAPH";
pub const MAP_HEAD: &'static str = "MAPHEAD";
pub const GAME_MAPS: &'static str = "GAMEMAPS";
pub const GAMEDATA: &'static str = "VSWAP";
pub const CONFIG_DATA: &'static str = "CONFIG";
pub const AUDIO_HEAD: &'static str = "AUDIOHED";
pub const AUDIO_DATA: &'static str = "AUDIOT";

const BLOCK: usize = 64;

#[derive(Clone, Copy)]
pub enum WolfFile {
    GraphicDict,
    GraphicHead,
    GraphicData,
    MapHead,
    GameMaps,
    GameData,
    ConfigData,
    AudioHead,
    AudioData,
}

// Contains everything from the generated header from the original.
#[derive(Debug)]
pub struct WolfVariant {
    pub file_ending: &'static str,
    pub num_episodes: usize,
    pub num_pics: usize,
    pub start_pics: usize,
    pub start_music: usize,
    pub start_adlib_sound: usize,
    pub start_digi_sound: usize,
    pub start_end_text: usize,
    pub graphic_num_shift: usize, // relative to the W3D6 graphic definitions
}

static SOD_FILE_ENDING: &str = "SOD";

pub static W3D1: WolfVariant = WolfVariant {
    file_ending: "WL1",
    num_episodes: 1,
    num_pics: 144,
    start_pics: 3,
    start_music: 261,
    start_adlib_sound: 87,
    start_digi_sound: 174,
    start_end_text: 155,
    graphic_num_shift: 12,
};

// TOOD define W3D3 version data
pub static W3D3: WolfVariant = WolfVariant {
    file_ending: "WL3",
    num_episodes: 3,
    num_pics: 0,
    start_pics: 0,
    start_music: 0,
    start_adlib_sound: 0,
    start_digi_sound: 0,
    start_end_text: 0,
    graphic_num_shift: 0,
};

pub static W3D6: WolfVariant = WolfVariant {
    file_ending: "WL6",
    num_episodes: 6,
    num_pics: 132,
    start_pics: 3,
    start_music: 261,
    start_adlib_sound: 87,
    start_digi_sound: 174,
    start_end_text: 143,
    graphic_num_shift: 0,
};

pub static SOD: WolfVariant = WolfVariant {
    file_ending: SOD_FILE_ENDING,
    num_episodes: 4,
    num_pics: 147,
    start_pics: 3,
    start_music: 243,
    start_adlib_sound: 81,
    start_digi_sound: 162,
    start_end_text: 168,
    graphic_num_shift: 0,
};

pub fn derive_variant(iw_config: &IWConfig) -> Result<&'static WolfVariant, String> {
    let mut data_path = iw_config.data.wolf3d_data.clone();
    data_path.push(file_name(WolfFile::GameData, &W3D6));
    if data_path.try_exists().map_err(|e| e.to_string())? {
        return Ok(&W3D6);
    }

    data_path.pop();
    data_path.push(file_name(WolfFile::GameData, &SOD));
    if data_path.try_exists().map_err(|e| e.to_string())? {
        return Ok(&SOD);
    }

    data_path.pop();
    data_path.push(file_name(WolfFile::GameData, &W3D1));
    if data_path.try_exists().map_err(|e| e.to_string())? {
        return Ok(&W3D1);
    }

    Err("NO WOLFENSTEIN 3-D DATA FILES to be found!".to_string())
}

pub fn is_sod(variant: &WolfVariant) -> bool {
    variant.file_ending == SOD_FILE_ENDING
}

#[derive(Serialize, Deserialize)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub fn gamepal_color(ix: usize) -> RGB {
    let offset = ix * 3;
    RGB {
        r: GAMEPAL[offset] << 2,
        g: GAMEPAL[offset + 1] << 2,
        b: GAMEPAL[offset + 2] << 2,
    }
}

pub fn file_name(file: WolfFile, variant: &WolfVariant) -> String {
    let f = match file {
        WolfFile::GraphicDict => GRAPHIC_DICT,
        WolfFile::GraphicHead => GRAPHIC_HEAD,
        WolfFile::GraphicData => GRAPHIC_DATA,
        WolfFile::MapHead => MAP_HEAD,
        WolfFile::GameMaps => GAME_MAPS,
        WolfFile::GameData => GAMEDATA,
        WolfFile::ConfigData => CONFIG_DATA,
        WolfFile::AudioHead => AUDIO_HEAD,
        WolfFile::AudioData => AUDIO_DATA,
    };
    f.to_owned() + "." + variant.file_ending
}

#[repr(usize)]
#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub enum SoundName {
    HITWALL,       // 0
    SELECTWPN,     // 1 //unused
    SELECTITEM,    // 2 //unused
    HEARTBEAT,     // 3 //unused
    MOVEGUN2,      // 4
    MOVEGUN1,      // 5
    NOWAY,         // 6
    NAZIHITPLAYER, // 7 //unused
    SCHABBSTHROW,  // 8
    PLAYERDEATH,   // 9
    DOGDEATH,      // 10
    ATKGATLING,    // 11
    GETKEY,        // 12
    NOITEM,        // 13 //unused
    WALK1,         // 14 //unused
    WALK2,         // 15 //unused
    TAKEDAMAGE,    // 16 //unused
    GAMEOVER,      // 17 //unused
    OPENDOOR,      // 18
    CLOSEDOOR,     // 19
    DONOTHING,     // 20
    HALT,          // 21
    DEATHSCREAM2,  // 22
    ATKKNIFE,      // 23
    ATKPISTOL,     // 24
    DEATHSCREAM3,  // 25
    ATKMACHINEGUN, // 26
    HITENEMY,      // 27 //unused
    SHOOTDOOR,     // 28 //unused
    DEATHSCREAM1,  // 29
    GETMACHINE,    // 30
    GETAMMO,       // 31
    SHOOT,         // 32
    HEALTH1,       // 33
    HEALTH2,       // 34
    BONUS1,        // 35
    BONUS2,        // 36
    BONUS3,        // 37
    GETGATLING,    // 38
    ESCPRESSED,    // 39
    LEVELDONE,     // 40
    DOGBARK,       // 41
    ENDBONUS1,     // 42
    ENDBONUS2,     // 43
    BONUS1UP,      // 44
    BONUS4,        // 45
    PUSHWALL,      // 46
    NOBONUS,       // 47
    PERCENT100,    // 48
    BOSSACTIVE,    // 49 //unused
    MUTTI,         // 50
    SCHUTZAD,      // 51
    AHHHG,         // 52
    DIE,           // 53
    EVA,           // 54
    GUTENTAG,      // 55
    LEBEN,         // 56
    SCHEIST,       // 57
    NAZIFIRE,      // 58
    BOSSFIRE,      // 59
    SSFIRE,        // 60
    SLURPIE,       // 61
    TOTHUND,       // 62
    MEINGOTT,      // 63
    SCHABBSHA,     // 64
    HITLERHA,      // 65
    SPION,         // 66
    NEINSOVAS,     // 67
    DOGATTACK,     // 68
    FLAMETHROWER,  // 69
    MECHSTEP,      // 70
    GOOBS,         // 71 //unused
    YEAH,          // 72
    DEATHSCREAM4,  // 73
    DEATHSCREAM5,  // 74
    DEATHSCREAM6,  // 75
    DEATHSCREAM7,  // 76
    DEATHSCREAM8,  // 77
    DEATHSCREAM9,  // 78
    DONNER,        // 79
    EINE,          // 80
    ERLAUBEN,      // 81
    KEIN,          // 82
    MEIN,          // 83
    ROSE,          // 84
    MISSILEFIRE,   // 85
    MISSILEHIT,    // 86
}

#[repr(usize)]
#[derive(Clone, Copy, Debug)]
pub enum Music {
    CORNER,   // 0
    DUNGEON,  // 1
    WARMARCH, // 2
    GETTHEM,  // 3
    HEADACHE, // 4
    HITLWLTZ, // 5
    INTROCW3, // 6
    NAZINOR,  // 7
    NAZIOMI,  // 8
    POW,      // 9
    SALUTE,   // 10
    SEARCHN,  // 11
    SUSPENSE, // 12
    VICTORS,  // 13
    WONDERIN, // 14
    FUNKYOU,  // 15
    ENDLEVEL, // 16
    GOINGAFT, // 17
    PREGNANT, // 18
    ULTIMATE, // 19
    NAZIRAP,  // 20
    ZEROHOUR, // 21
    TWELFTH,  // 22
    ROSTER,   // 23
    URAHERO,  // 24
    VICMARCH, // 25
    PACMAN,   // 26
}

// num values are chunk offsets. They need to be translated to
// picture offset in the graphics array with GraphicNum::PG13PIC - STARTPICS.
derive_from! {
    #[repr(usize)]
    #[derive(Copy, Clone, PartialEq, Debug)]
    pub enum GraphicNum {
        NONE = 0,
        HBLAZEPIC = 5,
        HTOPWINDOWPIC = 6,
        HLEFTWINDOWPIC = 7,
        HRIGHTWINDOWPIC = 8,
        HBOTTOMINFOPIC = 9,
        // Lump Start
        COPTIONSPIC = 10,
        CCURSOR1PIC = 11,
        CCURSOR2PIC = 12,
        CNOTSELECTEDPIC = 13,
        CSELECTEDPIC = 14,
        CFXTITLEPIC = 15,
        CDIGITITLEPIC = 16,
        CMUSICTITLEPIC = 17,
        CMOUSELBACKPIC = 18,
        CBABYMODEPIC = 19,
        CEASYPIC = 20,
        CNORMALPIC = 21,
        CHARDPIC = 22,
        CLOADSAVEDISKPIC = 23,
        CDISKLOADING1PIC = 24,
        CDISKLOADING2PIC = 25,
        CCONTROLPIC = 26,
        CCUSTOMIZEPIC = 27,
        CLOADGAMEPIC = 28,
        CSAVEGAMEPIC = 29,
        CEPISODE1PIC = 30,
        CEPISODE2PIC = 31,
        CEPISODE3PIC = 32,
        CEPISODE4PIC = 33,
        CEPISODE5PIC = 34,
        CEPISODE6PIC = 35,
        CCODEPIC = 36,
        CTIMECODEPIC = 37,
        CLEVELPIC = 38,
        CNAMEPIC = 39,
        CSCOREPIC = 40,

        GUYPIC = 43,
        COLONPIC = 44,
        NUM0PIC = 45,
        NUM1PIC = 46,
        NUM2PIC = 47,
        NUM3PIC = 48,
        NUM4PIC = 49,
        NUM5PIC = 50,
        NUM6PIC = 51,
        NUM7PIC = 52,
        NUM8PIC = 53,
        NUM9PIC = 54,
        PERCENTPIC = 55,
        APIC = 56,
        BPIC = 57,
        CPIC = 58,
        DPIC = 59,
        EPIC = 60,
        FPIC = 61,
        GPIC = 62,
        HPIC = 63,
        IPIC = 64,
        JPIC = 65,
        KPIC = 66,
        LPIC = 67,
        MPIC = 68,
        NPIC = 69,
        OPIC = 70,
        PPIC = 71,
        QPIC = 72,
        RPIC = 73,
        SPIC = 74,
        TPIC = 75,
        UPIC = 76,
        VPIC = 77,
        WPIC = 78,
        XPIC = 79,
        YPIC = 80,
        ZPIC = 81,
        EXPOINTPIC = 82,
        APOSTROPHEPIC = 83,
        GUY2PIC = 84,
        BJWINSPIC = 85,
        STATUSBARPIC = 86,
        TITLEPIC = 87,
        PG13PIC = 88,
        CREDITSPIC = 89,
        HIGHSCOREPIC = 90,
        // TODO add missing pics
        // Lump Start
        KNIFEPIC = 91,
        GUNPIC = 92,
        MACHINEGUNPIC = 93,
        GATLINGGUNPIC = 94,
        NOKEYPIC = 95,
        GOLDKEYPIC = 96,
        SILVERKEYPIC = 97,
        NBLANKPIC = 98,
        N0PIC = 99,
        N1PIC = 100,
        N2PIC = 101,
        N3PIC = 102,
        N4PIC = 103,
        N5PIC = 104,
        N6PIC = 105,
        N7PIC = 106,
        N8PIC = 107,
        N9PIC = 108,
        FACE1APIC = 109,
        FACE1BPIC = 110,
        FACE1CPIC = 111,
        FACE2APIC = 112,
        FACE2BPIC = 113,
        FACE2CPIC = 114,
        FACE3APIC = 115,
        FACE3BPIC = 116,
        FACE3CPIC = 117,
        FACE4APIC = 118,
        FACE4BPIC = 119,
        FACE4CPIC = 120,
        FACE5APIC = 121,
        FACE5BPIC = 122,
        FACE5CPIC = 123,
        FACE6APIC = 124,
        FACE6BPIC = 125,
        FACE6CPIC = 126,
        FACE7APIC = 127,
        FACE7BPIC = 128,
        FACE7CPIC = 129,
        FACE8APIC = 130,
        GOTGATLINGPIC = 131,
        MUTANTBJPIC = 132,
        PAUSEDPIC = 133,
        GETPSYCHEDPIC = 134,
    }
}

pub fn face_pic(n: usize) -> GraphicNum {
    let offset = GraphicNum::FACE1APIC as usize + n;
    match offset {
        109 => GraphicNum::FACE1APIC,
        110 => GraphicNum::FACE1BPIC,
        111 => GraphicNum::FACE1CPIC,
        112 => GraphicNum::FACE2APIC,
        113 => GraphicNum::FACE2BPIC,
        114 => GraphicNum::FACE2CPIC,
        115 => GraphicNum::FACE3APIC,
        116 => GraphicNum::FACE3BPIC,
        117 => GraphicNum::FACE3CPIC,
        118 => GraphicNum::FACE4APIC,
        119 => GraphicNum::FACE4BPIC,
        120 => GraphicNum::FACE4CPIC,
        121 => GraphicNum::FACE5APIC,
        122 => GraphicNum::FACE5BPIC,
        123 => GraphicNum::FACE5CPIC,
        124 => GraphicNum::FACE6APIC,
        125 => GraphicNum::FACE6BPIC,
        126 => GraphicNum::FACE6CPIC,
        127 => GraphicNum::FACE7APIC,
        128 => GraphicNum::FACE7BPIC,
        129 => GraphicNum::FACE7CPIC,
        130 => GraphicNum::FACE8APIC,
        _ => GraphicNum::FACE1APIC,
    }
}

// GraphicNum::N0PIC to GraphicNum N9PIC conversion (number for the HUD).
// If n > 9 GraphicNum::NBLANKPIC is returned.
pub fn n_pic(n: usize) -> GraphicNum {
    match n {
        0 => GraphicNum::N0PIC,
        1 => GraphicNum::N1PIC,
        2 => GraphicNum::N2PIC,
        3 => GraphicNum::N3PIC,
        4 => GraphicNum::N4PIC,
        5 => GraphicNum::N5PIC,
        6 => GraphicNum::N6PIC,
        7 => GraphicNum::N7PIC,
        8 => GraphicNum::N8PIC,
        9 => GraphicNum::N9PIC,
        _ => GraphicNum::NBLANKPIC,
    }
}

// GraphicNum::NUM0PIC to GraphicNum NUM9PIC conversion (number for the info screens).
// If n > 9 GraphicNum::NBLANKPIC is returned.
pub fn num_pic(n: usize) -> GraphicNum {
    match n {
        0 => GraphicNum::NUM0PIC,
        1 => GraphicNum::NUM1PIC,
        2 => GraphicNum::NUM2PIC,
        3 => GraphicNum::NUM3PIC,
        4 => GraphicNum::NUM4PIC,
        5 => GraphicNum::NUM5PIC,
        6 => GraphicNum::NUM6PIC,
        7 => GraphicNum::NUM7PIC,
        8 => GraphicNum::NUM8PIC,
        9 => GraphicNum::NUM9PIC,
        _ => GraphicNum::NBLANKPIC,
    }
}

pub fn weapon_pic(w: Option<WeaponType>) -> GraphicNum {
    match w {
        Option::None => GraphicNum::N0PIC,
        Option::Some(WeaponType::Knife) => GraphicNum::KNIFEPIC,
        Option::Some(WeaponType::Pistol) => GraphicNum::GUNPIC,
        Option::Some(WeaponType::MachineGun) => GraphicNum::MACHINEGUNPIC,
        Option::Some(WeaponType::ChainGun) => GraphicNum::GATLINGGUNPIC,
    }
}

const NUMTILE8: usize = 72;

const STARTFONT: usize = 1;
const STRUCTPIC: usize = 0;
const STARTTILE8: usize = 135;
const STARTTILE8M: usize = 136;
const STARTEXTERNS: usize = 136;
const NUM_FONT: usize = 2;
pub const NUM_DIGI_SOUNDS: usize = 47;

pub struct Graphic {
    pub data: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

#[derive(Debug)]
pub struct Font {
    pub height: u16,
    pub location: [u16; 256],
    pub width: [u8; 256],
    pub data: Vec<Vec<u8>>,
}

pub struct TileData {
    pub tile8: Vec<Vec<u8>>,
}

pub struct Huffnode {
    bit0: u16,
    bit1: u16,
}

#[derive(Copy, Clone)]
pub enum DigiChannel {
    Any,
    Player,
    Boss,
}

pub struct DigiMapEntry {
    pub sound: SoundName,
    pub page_no: usize,
    pub channel: DigiChannel,
}

pub static DIGI_MAP: [DigiMapEntry; NUM_DIGI_SOUNDS] = [
    DigiMapEntry {
        sound: SoundName::HALT,
        page_no: 0,
        channel: DigiChannel::Any,
    },
    DigiMapEntry {
        sound: SoundName::DOGBARK,
        page_no: 1,
        channel: DigiChannel::Any,
    },
    DigiMapEntry {
        sound: SoundName::CLOSEDOOR,
        page_no: 2,
        channel: DigiChannel::Any,
    },
    DigiMapEntry {
        sound: SoundName::OPENDOOR,
        page_no: 3,
        channel: DigiChannel::Any,
    },
    DigiMapEntry {
        sound: SoundName::ATKMACHINEGUN,
        page_no: 4,
        channel: DigiChannel::Player,
    },
    DigiMapEntry {
        sound: SoundName::ATKPISTOL,
        page_no: 5,
        channel: DigiChannel::Player,
    },
    DigiMapEntry {
        sound: SoundName::ATKGATLING,
        page_no: 6,
        channel: DigiChannel::Player,
    },
    DigiMapEntry {
        sound: SoundName::SCHUTZAD,
        page_no: 7,
        channel: DigiChannel::Any,
    },
    DigiMapEntry {
        sound: SoundName::GUTENTAG,
        page_no: 8,
        channel: DigiChannel::Any,
    },
    DigiMapEntry {
        sound: SoundName::MUTTI,
        page_no: 9,
        channel: DigiChannel::Any,
    },
    DigiMapEntry {
        sound: SoundName::BOSSFIRE,
        page_no: 10,
        channel: DigiChannel::Boss,
    },
    DigiMapEntry {
        sound: SoundName::SSFIRE,
        page_no: 11,
        channel: DigiChannel::Any,
    },
    DigiMapEntry {
        sound: SoundName::DEATHSCREAM1,
        page_no: 12,
        channel: DigiChannel::Any,
    },
    DigiMapEntry {
        sound: SoundName::DEATHSCREAM2,
        page_no: 13,
        channel: DigiChannel::Any,
    },
    DigiMapEntry {
        sound: SoundName::DEATHSCREAM3,
        page_no: 13,
        channel: DigiChannel::Any,
    },
    DigiMapEntry {
        sound: SoundName::TAKEDAMAGE,
        page_no: 14,
        channel: DigiChannel::Any,
    },
    DigiMapEntry {
        sound: SoundName::PUSHWALL,
        page_no: 15,
        channel: DigiChannel::Any,
    },
    DigiMapEntry {
        sound: SoundName::LEBEN,
        page_no: 20,
        channel: DigiChannel::Any,
    },
    DigiMapEntry {
        sound: SoundName::NAZIFIRE,
        page_no: 21,
        channel: DigiChannel::Any,
    },
    DigiMapEntry {
        sound: SoundName::SLURPIE,
        page_no: 22,
        channel: DigiChannel::Any,
    },
    DigiMapEntry {
        sound: SoundName::YEAH,
        page_no: 32,
        channel: DigiChannel::Any,
    },
    DigiMapEntry {
        sound: SoundName::DOGDEATH,
        page_no: 16,
        channel: DigiChannel::Any,
    },
    DigiMapEntry {
        sound: SoundName::AHHHG,
        page_no: 17,
        channel: DigiChannel::Any,
    },
    DigiMapEntry {
        sound: SoundName::DIE,
        page_no: 18,
        channel: DigiChannel::Any,
    },
    DigiMapEntry {
        sound: SoundName::EVA,
        page_no: 19,
        channel: DigiChannel::Any,
    },
    DigiMapEntry {
        sound: SoundName::TOTHUND,
        page_no: 23,
        channel: DigiChannel::Any,
    },
    DigiMapEntry {
        sound: SoundName::MEINGOTT,
        page_no: 24,
        channel: DigiChannel::Any,
    },
    DigiMapEntry {
        sound: SoundName::SCHABBSHA,
        page_no: 25,
        channel: DigiChannel::Any,
    },
    DigiMapEntry {
        sound: SoundName::HITLERHA,
        page_no: 26,
        channel: DigiChannel::Any,
    },
    DigiMapEntry {
        sound: SoundName::SPION,
        page_no: 27,
        channel: DigiChannel::Any,
    },
    DigiMapEntry {
        sound: SoundName::NEINSOVAS,
        page_no: 28,
        channel: DigiChannel::Any,
    },
    DigiMapEntry {
        sound: SoundName::DOGATTACK,
        page_no: 29,
        channel: DigiChannel::Any,
    },
    DigiMapEntry {
        sound: SoundName::LEVELDONE,
        page_no: 30,
        channel: DigiChannel::Any,
    },
    DigiMapEntry {
        sound: SoundName::MECHSTEP,
        page_no: 31,
        channel: DigiChannel::Any,
    },
    DigiMapEntry {
        sound: SoundName::SCHEIST,
        page_no: 33,
        channel: DigiChannel::Any,
    },
    DigiMapEntry {
        sound: SoundName::DEATHSCREAM4,
        page_no: 34,
        channel: DigiChannel::Any, // AIIEEE
    },
    DigiMapEntry {
        sound: SoundName::DEATHSCREAM5,
        page_no: 35,
        channel: DigiChannel::Any, // DEE-DEE
    },
    DigiMapEntry {
        sound: SoundName::DONNER,
        page_no: 36,
        channel: DigiChannel::Any, // EPISODE 4 BOSS DIE
    },
    DigiMapEntry {
        sound: SoundName::EINE,
        page_no: 37,
        channel: DigiChannel::Any, // EPISODE 4 BOSS SIGHTING
    },
    DigiMapEntry {
        sound: SoundName::ERLAUBEN,
        page_no: 38,
        channel: DigiChannel::Any, // EPISODE 6 BOSS SIGHTING
    },
    DigiMapEntry {
        sound: SoundName::DEATHSCREAM6,
        page_no: 39,
        channel: DigiChannel::Any, // FART
    },
    DigiMapEntry {
        sound: SoundName::DEATHSCREAM7,
        page_no: 40,
        channel: DigiChannel::Any, // GASP
    },
    DigiMapEntry {
        sound: SoundName::DEATHSCREAM8,
        page_no: 41,
        channel: DigiChannel::Any, // GUH-BOY!
    },
    DigiMapEntry {
        sound: SoundName::DEATHSCREAM9,
        page_no: 42,
        channel: DigiChannel::Any, // AH GEEZ!
    },
    DigiMapEntry {
        sound: SoundName::KEIN,
        page_no: 43,
        channel: DigiChannel::Any, // EPISODE 5 BOSS SIGHTING
    },
    DigiMapEntry {
        sound: SoundName::MEIN,
        page_no: 44,
        channel: DigiChannel::Any, // EPISODE 6 BOSS DIE
    },
    DigiMapEntry {
        sound: SoundName::ROSE,
        page_no: 45,
        channel: DigiChannel::Any, // EPISODE 5 BOSS DIE
    },
];

pub fn load_all_graphics(
    loader: &dyn Loader,
    patch_config: &Option<PatchConfig>,
) -> Result<(Vec<Graphic>, Vec<Font>, TileData, Vec<String>), String> {
    let grhuffman_bytes = loader.load_wolf_file(WolfFile::GraphicDict);
    let grhuffman = to_huffnodes(grhuffman_bytes);

    let grstarts = loader.load_wolf_file(WolfFile::GraphicHead);
    let grdata = loader.load_wolf_file(WolfFile::GraphicData);

    let picsizes = extract_picsizes(&grdata, &grstarts, &grhuffman, loader.variant());

    let mut fonts = Vec::with_capacity(NUM_FONT);
    for i in STARTFONT..(STARTFONT + NUM_FONT) {
        let font = load_font(i, &grstarts, &grdata, &grhuffman)?;
        fonts.push(font);
    }

    let variant = loader.variant();
    let mut graphics = Vec::with_capacity(variant.num_pics);
    for i in variant.start_pics..(variant.start_pics + variant.num_pics) {
        let g = if let Some(patch_file) = graphic_patch(patch_config, i) {
            let data = loader.load_patch_data_file(patch_file);
            let (w, h) = picsizes[i - variant.start_pics];
            Graphic {
                data,
                width: w,
                height: h,
            }
        } else {
            load_graphic(i, &grstarts, &grdata, &grhuffman, &picsizes, variant)?
        };
        graphics.push(g);
    }

    let tile8 = load_tile8(&grstarts, &grdata, &grhuffman)?;

    let mut texts = Vec::with_capacity(variant.num_episodes);
    for i in variant.start_end_text..(variant.start_end_text + variant.num_episodes) {
        let (pos, compressed) = data_sizes(i, &grstarts)?;
        let source = &grdata[pos..(pos + compressed)];
        let expanded = expand_chunk(i, source, &grhuffman);

        if let Some(ascii) = expanded.as_ascii() {
            texts.push(ascii.as_str().to_owned());
        } else {
            return Err("non ascii found in texts".to_owned());
        }
    }

    Ok((graphics, fonts, TileData { tile8 }, texts))
}

fn extract_picsizes(
    grdata: &Vec<u8>,
    grstarts: &Vec<u8>,
    grhuffman: &Vec<Huffnode>,
    variant: &WolfVariant,
) -> Vec<(usize, usize)> {
    let (complen, explen) = gr_chunk_length(STRUCTPIC, grdata, grstarts);
    let f_offset = (grfilepos(STRUCTPIC, grstarts) + 4) as usize;
    let expanded = huff_expand(&grdata[f_offset..(f_offset + complen)], explen, grhuffman);

    assert!(explen / 4 >= variant.num_pics); // otherwise the data file may not match the code

    let mut picsizes = Vec::with_capacity(variant.num_pics);
    let mut offset = 0;

    // TODO Write util functions for from_le_bytes()..try_into.unwrap noise
    for _ in 0..variant.num_pics
    /* (explen/4)*/
    {
        let width = i16::from_le_bytes(expanded[offset..(offset + 2)].try_into().unwrap()) as usize;
        let height =
            i16::from_le_bytes(expanded[offset + 2..(offset + 4)].try_into().unwrap()) as usize;
        picsizes.push((width as usize, height as usize));
        offset += 4;
    }

    picsizes
}

fn gr_chunk_length(chunk: usize, grdata: &Vec<u8>, grstarts: &Vec<u8>) -> (usize, usize) {
    let file_offset = grfilepos(chunk, grstarts) as usize;
    let chunkexplen =
        u32::from_le_bytes(grdata[file_offset..(file_offset + 4)].try_into().unwrap());
    (
        grfilepos(chunk + 1, grstarts) as usize - file_offset - 4,
        chunkexplen as usize,
    )
}

fn to_huffnodes(bytes: Vec<u8>) -> Vec<Huffnode> {
    let mut nodes = Vec::with_capacity(255);

    let mut offset = 0;
    for _ in 0..255 {
        let bit0 = u16::from_le_bytes(bytes[offset..(offset + 2)].try_into().unwrap());
        let bit1 = u16::from_le_bytes(bytes[(offset + 2)..(offset + 4)].try_into().unwrap());
        nodes.push(Huffnode { bit0, bit1 });
        offset += 4;
    }

    nodes
}

fn load_font(
    chunk: usize,
    grstarts: &Vec<u8>,
    grdata: &Vec<u8>,
    grhuffman: &Vec<Huffnode>,
) -> Result<Font, String> {
    let (pos, compressed) = data_sizes(chunk, grstarts)?;
    let source = &grdata[pos..(pos + compressed)];
    Ok(expand_font(chunk, source, grhuffman))
}

fn expand_font(chunk: usize, compressed: &[u8], grhuffman: &Vec<Huffnode>) -> Font {
    let expanded = expand_chunk(chunk, compressed, grhuffman);

    let mut reader = new_data_reader(&expanded);
    let height = reader.read_u16();

    let mut location = [0; 256];
    for i in 0..256 {
        location[i] = reader.read_u16();
    }

    let mut width = [0; 256];
    for i in 0..256 {
        width[i] = reader.read_u8();
    }
    let mut font_data: Vec<Vec<u8>> = Vec::with_capacity(256);
    for i in 0..256 {
        let bytes = height as usize * width[i] as usize;
        let start = location[i] as usize;
        font_data.push(expanded[start..(start + bytes)].to_vec());
    }
    return Font {
        height,
        location,
        width,
        data: font_data,
    };
}

fn load_graphic(
    chunk: usize,
    grstarts: &Vec<u8>,
    grdata: &Vec<u8>,
    grhuffman: &Vec<Huffnode>,
    picsizes: &Vec<(usize, usize)>,
    variant: &WolfVariant,
) -> Result<Graphic, String> {
    let (pos, compressed) = data_sizes(chunk, grstarts)?;
    let source = &grdata[pos..(pos + compressed)];
    Ok(expand_graphic(chunk, source, grhuffman, picsizes, variant))
}

fn load_tile8(
    grstarts: &Vec<u8>,
    grdata: &Vec<u8>,
    grhuffman: &Vec<Huffnode>,
) -> Result<Vec<Vec<u8>>, String> {
    let (pos, compressed) = data_sizes(STARTTILE8, grstarts)?;
    let source = &grdata[pos..(pos + compressed)];
    let expanded = expand_chunk(STARTTILE8, source, grhuffman);

    let mut result = Vec::with_capacity(NUMTILE8);
    for i in 0..NUMTILE8 {
        result.push(expanded[(i * BLOCK)..(i * BLOCK + BLOCK)].to_vec())
    }
    Ok(result)
}

fn data_sizes(chunk: usize, grstarts: &Vec<u8>) -> Result<(usize, usize), String> {
    let pos_int = grfilepos(chunk, grstarts);
    if pos_int < 0 {
        return Err(format!("could not load chunk {}", pos_int));
    }
    let pos = pos_int as usize;
    let mut next = chunk + 1;
    while grfilepos(next, grstarts) == -1 {
        next += 1;
    }

    let compressed = (grfilepos(next, grstarts) - pos_int) as usize;
    Ok((pos, compressed))
}

fn grfilepos(chunk: usize, grstarts: &Vec<u8>) -> i32 {
    let offset = chunk * 3;
    let value = i32::from_le_bytes([
        grstarts[offset],
        grstarts[offset + 1],
        grstarts[offset + 2],
        0,
    ]);
    if value == 0xffffff { -1 } else { value }
}

fn expand_chunk(chunk: usize, data_in: &[u8], grhuffman: &Vec<Huffnode>) -> Vec<u8> {
    let expanded;
    let data;
    if chunk >= STARTTILE8 && chunk < STARTEXTERNS {
        if chunk < STARTTILE8M {
            expanded = BLOCK * NUMTILE8;
        } else {
            panic!("TILE Expand not yet implemented");
        }
        data = data_in;
    } else {
        expanded = i32::from_le_bytes(data_in[0..4].try_into().unwrap()) as usize;
        data = &data_in[4..]; // skip over length
    }

    huff_expand(data, expanded, grhuffman)
}

fn expand_graphic(
    chunk: usize,
    data: &[u8],
    grhuffman: &Vec<Huffnode>,
    picsizes: &Vec<(usize, usize)>,
    variant: &WolfVariant,
) -> Graphic {
    let expanded = expand_chunk(chunk, data, grhuffman);
    let size = picsizes[chunk - variant.start_pics];
    return Graphic {
        data: expanded,
        width: size.0,
        height: size.1,
    };
}

fn huff_expand(data: &[u8], expanded_len: usize, grhuffman: &Vec<Huffnode>) -> Vec<u8> {
    let mut expanded = vec![0; expanded_len];
    let head = &grhuffman[254];
    let mut written = 0;
    if expanded_len < 0xfff0 {
        let mut node = head;
        let mut read = 0;
        let mut input = data[read];
        read += 1;
        let mut mask: u8 = 0x01;
        while written < expanded_len {
            let node_value = if (input & mask) == 0 {
                // bit not set
                node.bit0
            } else {
                node.bit1
            };

            if mask == 0x80 {
                if read >= data.len() {
                    break;
                }
                input = data[read];
                read += 1;
                mask = 1;
            } else {
                mask <<= 1;
            }

            if node_value < 256 {
                // leaf node, dx is the uncompressed byte!
                expanded[written] = node_value as u8;
                written += 1;
                node = head;
            } else {
                // -256 here, since the huffman optimisation is not done
                node = &grhuffman[(node_value - 256) as usize];
            }
        }
    } else {
        panic!("implement expand 64k data");
    }
    expanded
}

// map stuff

// load map and uncompress it
pub fn load_map_from_assets(assets: &Assets, mapnum: usize) -> Result<MapSegs, String> {
    let mut cursor = Cursor::new(&assets.game_maps);
    load_map(
        &mut cursor,
        &assets.map_headers,
        &assets.map_offsets,
        mapnum,
    )
}

pub fn load_map_headers_from_config(
    loader: &dyn Loader,
) -> Result<(MapFileType, Vec<MapType>), String> {
    let offset_bytes = loader.load_wolf_file(WolfFile::MapHead);
    let map_bytes = loader.load_wolf_file(WolfFile::GameMaps);
    let offsets = load_map_offsets(&offset_bytes)?;
    load_map_headers(&map_bytes, offsets)
}

// gamedata stuff

// loads all assets for the game into memory
pub fn load_assets(sound: &Sound, loader: &dyn Loader) -> Result<Assets, String> {
    let (map_offsets, map_headers) = load_map_headers_from_config(loader)?;

    let gamedata_bytes = loader.load_wolf_file(WolfFile::GameData);
    let gamedata_headers = gamedata::load_gamedata_headers(&gamedata_bytes)?;

    let mut gamedata_cursor = Cursor::new(gamedata_bytes);
    let textures = gamedata::load_all_textures(&mut gamedata_cursor, &gamedata_headers)?;
    let sprites = gamedata::load_all_sprites(&mut gamedata_cursor, &gamedata_headers)?;
    let digi_sounds =
        gamedata::load_all_digi_sounds(sound, &mut gamedata_cursor, &gamedata_headers)?;

    let mut audio_header_cursor = Cursor::new(loader.load_wolf_file(WolfFile::AudioHead));
    let audio_headers = gamedata::load_audio_headers(&mut audio_header_cursor)?;

    let mut audio_cursor = Cursor::new(loader.load_wolf_file(WolfFile::AudioData));
    let audio_sounds =
        gamedata::load_audio_sounds(&audio_headers, &mut audio_cursor, loader.variant())?;

    let game_maps = loader.load_wolf_file(WolfFile::GameMaps);

    Ok(Assets {
        map_headers,
        map_offsets,
        textures,
        sprites,
        game_maps,
        gamedata_headers,
        audio_headers,
        audio_sounds,
        digi_sounds,
    })
}

pub fn load_graphic_assets(loader: &dyn Loader) -> Result<Assets, String> {
    let (map_offsets, map_headers) = load_map_headers_from_config(loader)?;

    let gamedata_bytes = loader.load_wolf_file(WolfFile::GameData);
    let gamedata_headers = gamedata::load_gamedata_headers(&gamedata_bytes)?;

    let mut gamedata_cursor = Cursor::new(gamedata_bytes);
    let textures = gamedata::load_all_textures(&mut gamedata_cursor, &gamedata_headers)?;
    let sprites = gamedata::load_all_sprites(&mut gamedata_cursor, &gamedata_headers)?;
    let game_maps = loader.load_wolf_file(WolfFile::GameMaps);

    Ok(Assets {
        map_headers,
        map_offsets,
        textures,
        sprites,
        game_maps,
        gamedata_headers,
        audio_headers: Vec::with_capacity(0),
        audio_sounds: Vec::with_capacity(0),
        digi_sounds: HashMap::new(),
    })
}
