use std::{thread, time};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use chrono::{Duration, Local, TimeZone};
use confy;
use directories::ProjectDirs;
use geodate::{moon_transit, sun_transit};
use serde::{Deserialize, Serialize};
use toml;
use ctrlc;

#[derive(Serialize, Deserialize, Debug)]
struct WallpaperChangerConfig {
    longitude: f64,
    latitude: f64,
    wallpaper_pack: String,
}

impl Default for WallpaperChangerConfig {
    fn default() -> Self {
        Self {
            longitude: 45.71,
            latitude: 15.81,
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
    Midnight,
    Sunrise,
    Noon,
    Sunset,
    Moonrise,
    Moonset,
    NextDayMidnight,
}


fn get_day_sun_and_moon_position_times(
    today_posix: i64,
    longitude: f64,
    latitude: f64,
) -> Result<HashMap<SunAndMoonKeys, i64>, String> {
    let mut sun_and_moon = HashMap::new();

    sun_and_moon.insert(
        SunAndMoonKeys::Sunrise,
        sun_transit::get_sunrise(
            today_posix,
            longitude,
            latitude,
        ).ok_or_else(|| "Can't get sunrise.")?,
    );
    sun_and_moon.insert(
        SunAndMoonKeys::Sunset,
        sun_transit::get_sunset(
            today_posix,
            longitude,
            latitude,
        ).ok_or_else(|| "Can't get sunset.")?,
    );

    sun_and_moon.insert(
        SunAndMoonKeys::Noon,
        sun_transit::get_noon(
            today_posix,
            longitude,
        ),
    );
    sun_and_moon.insert(
        SunAndMoonKeys::Midnight,
        sun_transit::get_midnight(
            today_posix,
            longitude,
        ),
    );

    sun_and_moon.insert(
        SunAndMoonKeys::Moonrise,
        moon_transit::get_moonrise(
            today_posix,
            longitude,
            latitude,
        ).ok_or_else(|| "Can't get moonrise.")?,
    );
    sun_and_moon.insert(
        SunAndMoonKeys::Moonset,
        moon_transit::get_moonset(
            today_posix,
            longitude,
            latitude,
        ).ok_or_else(|| "Can't get moonset.")?,
    );

    sun_and_moon.insert(
        SunAndMoonKeys::NextDayMidnight,
        (Local.timestamp_opt(today_posix, 0).unwrap() + Duration::days(1)).timestamp()
    );

    Ok(sun_and_moon)
}


fn timestamp_splitter(
    start: i64,
    end: i64,
    chunks: i64
) -> Vec<i64> {
    let step = (end - start) / chunks;

    (1..chunks + 1).map(|x| start + x * step).collect()
}


fn map_images_and_timestamps(
    sun_and_moon: &HashMap<SunAndMoonKeys, i64>,
    wallpaper_pack_config: &WallpaperPackConfig,
    wallpaper_pack_dir: &String
) -> (Vec<PathBuf>, Vec<i64>) {
    let mut to_return_images: Vec<PathBuf> = vec![];
    let mut to_return_timestamps: Vec<i64> = vec![];

    to_return_images.extend(
        wallpaper_pack_config.midnight
            .clone()
            .iter()
            .map(|x| {
                PathBuf::new()
                    .join(wallpaper_pack_dir)
                    .join(x)
            })
            .collect::<Vec<PathBuf>>()
    );
    to_return_timestamps.extend(
        timestamp_splitter(
            sun_and_moon[&SunAndMoonKeys::Midnight],
            sun_and_moon[&SunAndMoonKeys::Moonset],
            wallpaper_pack_config.midnight.len() as i64
        )
    );

    to_return_images.extend(
        wallpaper_pack_config.moonset
            .clone()
            .iter()
            .map(|x| {
                PathBuf::new()
                    .join(wallpaper_pack_dir)
                    .join(x)
            })
            .collect::<Vec<PathBuf>>()
    );
    to_return_timestamps.extend(
        timestamp_splitter(
            sun_and_moon[&SunAndMoonKeys::Moonset],
            sun_and_moon[&SunAndMoonKeys::Sunrise],
            wallpaper_pack_config.moonset.len() as i64
        )
    );

    to_return_images.extend(
        wallpaper_pack_config.sunrise
            .clone()
            .iter()
            .map(|x| {
                PathBuf::new()
                    .join(wallpaper_pack_dir)
                    .join(x)
            })
            .collect::<Vec<PathBuf>>()
    );
    to_return_timestamps.extend(
        timestamp_splitter(
            sun_and_moon[&SunAndMoonKeys::Sunrise],
            sun_and_moon[&SunAndMoonKeys::Noon],
            wallpaper_pack_config.sunrise.len() as i64
        )
    );

    to_return_images.extend(
        wallpaper_pack_config.noon
            .clone()
            .iter()
            .map(|x| {
                PathBuf::new()
                    .join(wallpaper_pack_dir)
                    .join(x)
            })
            .collect::<Vec<PathBuf>>()
    );
    to_return_timestamps.extend(
        timestamp_splitter(
            sun_and_moon[&SunAndMoonKeys::Noon],
            sun_and_moon[&SunAndMoonKeys::Sunset],
            wallpaper_pack_config.noon.len() as i64
        )
    );

    to_return_images.extend(
        wallpaper_pack_config.sunset
            .clone()
            .iter()
            .map(|x| {
                PathBuf::new()
                    .join(wallpaper_pack_dir)
                    .join(x)
            })
            .collect::<Vec<PathBuf>>()
    );
    to_return_timestamps.extend(
        timestamp_splitter(
            sun_and_moon[&SunAndMoonKeys::Sunset],
            sun_and_moon[&SunAndMoonKeys::Moonrise],
            wallpaper_pack_config.sunset.len() as i64
        )
    );

    to_return_images.extend(
        wallpaper_pack_config.moonrise
            .clone()
            .iter()
            .map(|x| {
                PathBuf::new()
                    .join(wallpaper_pack_dir)
                    .join(x)
            })
            .collect::<Vec<PathBuf>>()
    );
    to_return_timestamps.extend(
        timestamp_splitter(
            sun_and_moon[&SunAndMoonKeys::Moonrise],
            sun_and_moon[&SunAndMoonKeys::NextDayMidnight],
            wallpaper_pack_config.moonrise.len() as i64
        )
    );

    (to_return_images, to_return_timestamps)
}


fn main() -> Result<(), String>{
    let app_name= "wallpaper_changer_rust".to_string();
    let config_name = "wallpaper_changer_config.toml".to_string();
    let wallpaper_pack_config_name = "wallpaper_pack_config.toml".to_string();

    let project_dirs: ProjectDirs = ProjectDirs::from(
        "hr",
        "IDerdic",
        &app_name
    ).ok_or_else(|| "Unable to create ProjectDirs struct.")?;

    let wallpaper_packs_dir = project_dirs
        .data_local_dir()
        .to_path_buf()
        .join("wallpaper_packs")
        .to_str()
        .ok_or_else(|| "Unable to convert PathBuf to &str.")?
        .to_string();

    if !Path::new(&wallpaper_packs_dir).exists() {
        fs::create_dir_all(&wallpaper_packs_dir)
            .ok()
            .ok_or_else(|| "Unable to create wallpaper pack directory tree.")?;
    }

    let mut today = Local::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| "Unable to get current day timestamp.")?;

    let config_path = project_dirs
        .config_local_dir()
        .to_path_buf()
        .join(&config_name)
        .to_str()
        .ok_or_else(|| "Unable to convert PathBuf to &str.")?
        .to_string();

    let config: WallpaperChangerConfig = confy::load_path(&config_path)
        .ok()
        .ok_or_else(|| "Unable to load the config file.")?;

    if config.wallpaper_pack.eq("") {
        println!("Wallpaper pack is not selected.\nCheck the config folder at path: {config_path}");
        return Ok(());
    }

    let wallpaper_pack_dir = PathBuf::new()
        .join(&wallpaper_packs_dir)
        .join(&config.wallpaper_pack)
        .to_str()
        .ok_or_else(|| "Unable to convert PathBuf to &str.")?
        .to_string();

    let wallpaper_pack_config_path = PathBuf::new()
        .join(&wallpaper_pack_dir)
        .join(&wallpaper_pack_config_name)
        .to_str()
        .ok_or_else(|| "Unable to convert PathBuf to &str.")?
        .to_string();

    let wallpaper_pack_config: WallpaperPackConfig = toml::from_str(
            &fs::read_to_string(&wallpaper_pack_config_path)
                .ok()
                .ok_or_else(|| "unable to read wallpaper_pack_config.toml to String.")?
        ).ok().ok_or_else(|| "Unable to parse wallpaper_pack_config.toml file.")?;

    let mut sun_and_moon = get_day_sun_and_moon_position_times(
        today.timestamp(),
        config.longitude,
        config.latitude,
    )?;

    let (mut images_seq, mut timestamp_seq) = map_images_and_timestamps(
        &sun_and_moon,
        &wallpaper_pack_config,
        &wallpaper_pack_dir
    );

    let mut current_timestamp = Local::now().timestamp();

    let terminate_loop = Arc::new(AtomicBool::new(false));
    let tl = terminate_loop.clone();

    ctrlc::set_handler(move || {
        tl.store(true, Ordering::SeqCst);
    }).ok().ok_or_else(|| "Unable to set Ctrl+C handler.")?;

    while !terminate_loop.load(Ordering::SeqCst) {
        if current_timestamp > sun_and_moon[&SunAndMoonKeys::NextDayMidnight] {
            today = Local::now()
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .ok_or_else(|| "Unable to get current day timestamp.")?;

            sun_and_moon = get_day_sun_and_moon_position_times(
                today.timestamp(),
                config.longitude,
                config.latitude,
            )?;

            let (images_seq_tmp, timestamp_seq_tmp) = map_images_and_timestamps(
                &sun_and_moon,
                &wallpaper_pack_config,
                &wallpaper_pack_dir
            );

            images_seq = images_seq_tmp;
            timestamp_seq = timestamp_seq_tmp;
        }

        for (index, timestamp) in timestamp_seq.iter().enumerate() {
            if current_timestamp < *timestamp {
                wallpaper::set_from_path(
                    images_seq[index]
                        .to_str()
                        .ok_or_else(|| "Unable to convert PathBuf to &str.")?
                ).ok().ok_or_else(|| "Unable to set wallpaper.")?;
                break;
            }
        }

        thread::sleep(time::Duration::from_secs(30));

        current_timestamp = Local::now().timestamp();
    }

    Ok(())
}
