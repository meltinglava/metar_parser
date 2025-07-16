use std::io::{BufRead, BufReader, Read};

use itertools::Itertools;
use nom::{
    AsChar, Finish, IResult, Parser,
    bytes::complete::{tag, take_till},
    character::complete::char,
    combinator::opt,
    sequence::preceded,
};

use crate::{
    obscuration::{Obscuration, nom_obscuration},
    pressure::{Pressure, nom_pressure},
    temprature::{Temprature, nom_temprature},
    units::timestamp::{Timestamp, nom_metar_timestamp},
    wind::{Wind, nom_wind},
};

#[derive(Debug, Clone)]
pub struct Metar {
    pub raw: String,
    pub icao: String,
    pub timestamp: Timestamp,
    pub auto: bool,
    pub wind: Wind,
    pub obscuration: Obscuration,
    pub temprature: Temprature,
    pub pressure: Pressure,
    pub nosig: bool,
    pub remarks: Option<String>,
}

pub fn nom_parse_metar(input: &str) -> IResult<&str, Metar> {
    let (rest, (icao, timestamp, auto, wind, obscuration, temprature, pressure, nosig, remark)) = (
        nom::bytes::complete::take(4usize),
        preceded(char(' '), nom_metar_timestamp),
        opt(tag(" AUTO")),
        preceded(char(' '), nom_wind),
        preceded(char(' '), nom_obscuration),
        preceded(char(' '), nom_temprature),
        preceded(char(' '), nom_pressure),
        opt(tag(" NOSIG")),
        opt(preceded(tag(" RMK "), take_till(char::is_newline))),
    )
        .parse(input)?;
    Ok((
        rest,
        Metar {
            raw: input.to_string(),
            icao: icao.to_string(),
            timestamp,
            auto: auto.is_some(),
            wind,
            obscuration,
            temprature,
            pressure,
            nosig: nosig.is_some(),
            remarks: remark.map(str::to_string),
        },
    ))
}

fn parse_metars<R: Read>(input: R) -> Result<Vec<(String, Metar)>, nom::error::Error<String>> {
    let reader = BufReader::new(input);
    let results = reader
        .lines()
        .filter_map(Result::ok)
        .map(|m| -> Result<(String, Metar), nom::error::Error<String>> {
            let (rest, metar) = nom_parse_metar(&m).finish()?;
            Ok((rest.to_string(), metar))
        })
        .try_collect();
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        let input = std::fs::File::open("test.metars").unwrap();
        let metars = parse_metars(input).unwrap();
        assert!(metars.len() > 0);
    }
}
