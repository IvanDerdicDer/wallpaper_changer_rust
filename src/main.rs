use std::collections::HashMap;
use std::path::{Path, PathBuf};
use chrono::{Local};
use geodate::{sun_transit, moon_transit};
use confy;
use serde::{Serialize, Deserialize};
use directories::ProjectDirs;
use std::fs;
use toml;


#[derive(Serialize, Deserialize, Debug)]
struct WallpaperChangerConfig {
    longitude: f64,
    latitude: f64,
    wallpaper_pack: String,
}

impl Default for WallpaperChangerConfig {
    fn default() -> Self {
        Self {
            longitude: 45.71422303959685,
            latitude: 15.817379689633158,
            wallpaper_pack: "".to_string(),
        }
    }
}


#[derive(Serialize, Deserialize, Debug)]
struct WallpaperPackConfig {
    midnight: Vec<String>,
    sunrise: Vec<String>,
    noon: Vec<String>,
    sunset: Vec<String>,
    moonrise: Vec<String>,
    moonset: Vec<String>
}


#[derive(Hash, PartialEq, Eq, Debug)]
enum SunAndMoonKeys {
    MIDNIGHT,
    SUNRISE,
    NOON,
    SUNSET,
    MOONRISE,
    MOONSET,
}


fn get_day_sun_and_moon_position_times(
    today_posix: i64,
    longitude: f64,
    latitude: f64,
) -> Result<HashMap<SunAndMoonKeys, i64>, String> {
    let mut sun_and_moon = HashMap::new();

    sun_and_moon.insert(
        SunAndMoonKeys::SUNRISE,
        sun_transit::get_sunrise(
            today_posix,
            longitude,
            latitude,
        ).ok_or_else(|| "Can't get sunrise.")?,
    );
    sun_and_moon.insert(
        SunAndMoonKeys::SUNSET,
        sun_transit::get_sunset(
            today_posix,
            longitude,
            latitude,
        ).ok_or_else(|| "Can't get sunset.")?,
    );

    sun_and_moon.insert(
        SunAndMoonKeys::NOON,
        sun_transit::get_noon(
            today_posix,
            longitude,
        ),
    );
    sun_and_moon.insert(
        SunAndMoonKeys::MIDNIGHT,
        sun_transit::get_midnight(
            today_posix,
            longitude,
        ),
    );

    sun_and_moon.insert(
        SunAndMoonKeys::MOONRISE,
        moon_transit::get_moonrise(
            today_posix,
            longitude,
            latitude,
        ).ok_or_else(|| "Can't get moonrise.")?,
    );
    sun_and_moon.insert(
        SunAndMoonKeys::MOONSET,
        moon_transit::get_moonset(
            today_posix,
            longitude,
            latitude,
        ).ok_or_else(|| "Can't get moonset.")?,
    );

    Ok(sun_and_moon)
}


fn timestamp_splitter(
    start: i64,
    end: i64,
    chunks: i64
) -> Vec<i64> {
    let step = (end - start) / chunks;

    (0..chunks).map(|x| start + x * step).collect()
}


fn main() {
    let app_name= "wallpaper_changer_rust".to_string();
    let config_name = "wallpaper_changer_config.toml".to_string();
    let wallpaper_pack_config_name = "wallpaper_pack_config.toml".to_string();

    let project_dirs: ProjectDirs = ProjectDirs::from(
        "hr",
        "IDerdic",
        &app_name
    ).unwrap();

    let wallpaper_packs_dir = project_dirs
        .data_local_dir()
        .to_path_buf()
        .join("wallpaper_packs")
        .to_str()
        .unwrap()
        .to_string();

    if !Path::new(&wallpaper_packs_dir).exists() {
        fs::create_dir_all(&wallpaper_packs_dir).unwrap();
    }

    let today = Local::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap();

    let config_path = project_dirs
        .config_local_dir()
        .to_path_buf()
        .join(&config_name)
        .to_str()
        .unwrap()
        .to_string();

    let config: WallpaperChangerConfig = confy::load_path(&config_path).unwrap();

    if config.wallpaper_pack.eq("") {
        println!("Wallpaper pack is not selected.\nCheck the config folder at path: {config_path}");
        return;
    }

    let wallpaper_pack_dir = PathBuf::new()
        .join(&wallpaper_packs_dir)
        .join(&config.wallpaper_pack)
        .to_str()
        .unwrap()
        .to_string();

    let wallpaper_pack_config_path = PathBuf::new()
        .join(&wallpaper_pack_dir)
        .join(&wallpaper_pack_config_name)
        .to_str()
        .unwrap()
        .to_string();

    let wallpaper_pack_config: WallpaperPackConfig = toml::from_str(
            &fs::read_to_string(&wallpaper_pack_config_path).unwrap()
        )
        .unwrap();

    let sun_and_moon = get_day_sun_and_moon_position_times(
        today.timestamp(),
        config.longitude,
        config.latitude,
    ).unwrap();

    println!("{:?}", sun_and_moon);
    println!("{:?}", config.wallpaper_pack);
}
