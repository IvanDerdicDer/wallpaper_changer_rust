use std::collections::HashMap;
use chrono::{Local};
use geodate::{sun_transit, moon_transit};

#[derive(Hash, PartialEq, Eq, Debug)]
enum SunAndMoonKeys {
    MIDNIGHT,
    SUNRISE,
    NOON,
    SUNSET,
    MOONRISE,
    MOONSET
}


fn get_day_sun_and_moon_position_times(
    today_posix: i64,
    longitude: f64,
    latitude: f64
) -> Result<HashMap<SunAndMoonKeys, i64>, String> {
    let mut sun_and_moon = HashMap::new();

    sun_and_moon.insert(
        SunAndMoonKeys::SUNRISE,
        sun_transit::get_sunrise(
            today_posix,
            longitude,
            latitude
        ).ok_or_else(|| "Can't get sunrise.")?
    );
    sun_and_moon.insert(
        SunAndMoonKeys::SUNSET,
        sun_transit::get_sunset(
            today_posix,
            longitude,
            latitude
        ).ok_or_else(|| "Can't get sunset.")?
    );

    sun_and_moon.insert(
        SunAndMoonKeys::NOON,
        sun_transit::get_noon(
            today_posix,
            longitude
        )
    );
    sun_and_moon.insert(
        SunAndMoonKeys::MIDNIGHT,
        sun_transit::get_midnight(
            today_posix,
            longitude
        )
    );

    sun_and_moon.insert(
        SunAndMoonKeys::MOONRISE,
        moon_transit::get_moonrise(
            today_posix,
            longitude,
            latitude
        ).ok_or_else(|| "Can't get moonrise.")?
    );
    sun_and_moon.insert(
        SunAndMoonKeys::MOONSET,
        moon_transit::get_moonset(
            today_posix,
            longitude,
            latitude
        ).ok_or_else(|| "Can't get moonset.")?
    );

    Ok(sun_and_moon)
}


fn main() -> Result<(), String> {
    let today = Local::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| "Invalid datetime")?;

    let latitude = 45.71422303959685;
    let longitude = 15.817379689633158;

    let sun_and_moon = get_day_sun_and_moon_position_times(
        today.timestamp(),
        longitude,
        latitude
    )?;

    println!("{:?}", sun_and_moon.into_keys().collect::<Vec<SunAndMoonKeys>>());

    Ok(())
}
