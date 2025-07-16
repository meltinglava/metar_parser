use nom::{
    Parser,
    branch::alt,
    bytes::complete::{tag, take},
    character::complete::{alphanumeric1, u32},
    combinator::{all_consuming, map, map_parser, opt, value},
    multi::many0,
    sequence::{preceded, separated_pair, terminated},
};

use crate::{
    optional_data::OptionalData,
    units::altitudes::{CloudHeight, nom_cloud_height},
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Obscuration {
    Described(DescribedObscuration),
    Cavok,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DescribedObscuration {
    pub visibility: Visibility,
    pub rvr: Vec<Rvr>,
    pub clouds: Vec<Cloud>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Rvr {
    pub runway: String,
    pub value: OptionalData<u32, 4>,
    pub distance_modifier: Option<DistanceModifier>,
    pub comment: Option<Trend>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Trend {
    Increasing,
    Decreasing,
    NoDistinctChange,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Cloud {
    pub coverage: OptionalData<CloudCoverage, 3>,
    pub height: OptionalData<CloudHeight, 3>,
    pub cloud_type: Option<OptionalData<String, 3>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CloudCoverage {
    Few,
    Scattered,
    Broken,
    Overcast,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Visibility {
    Meters(OptionalData<u32, 4>),
    StatuteMiles(StatuteMilesVisibility),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StatuteMilesVisibility {
    pub whole: Option<u32>,
    pub fraction: Option<(u32, u32)>,
    pub modifier: Option<DistanceModifier>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DistanceModifier {
    LessThan,
    GreaterThan,
}

pub(crate) fn nom_obscuration(input: &str) -> nom::IResult<&str, Obscuration> {
    alt((
        value(Obscuration::Cavok, tag("CAVOK")),
        map(nom_described_obscuration, Obscuration::Described),
    ))
    .parse(input)
}

fn nom_described_obscuration(input: &str) -> nom::IResult<&str, DescribedObscuration> {
    map(
        (nom_visibility, many0(nom_rvr), many0(nom_cloud)),
        |(visibility, rvr, clouds)| DescribedObscuration {
            visibility,
            rvr,
            clouds,
        },
    )
    .parse(input)
}

fn nom_visibility(input: &str) -> nom::IResult<&str, Visibility> {
    alt((
        map(nom_statute_miles_visibility, Visibility::StatuteMiles),
        map(OptionalData::optional_field(map_parser(take(4usize), all_consuming(u32))), Visibility::Meters),
    ))
    .parse(input)
}

fn nom_fraction(input: &str) -> nom::IResult<&str, (u32, u32)> {
    separated_pair(u32, tag("/"), u32).parse(input)
}

fn nom_statute_miles_visibility(input: &str) -> nom::IResult<&str, StatuteMilesVisibility> {
    let fraction_only = map(
        (
            opt(nom_distance_modifier),
            terminated(nom_fraction, tag("SM")),
        ),
        |(modifier, fraction)| StatuteMilesVisibility {
            whole: None,
            fraction: Some(fraction),
            modifier,
        },
    );

    let whole = map(
        terminated(
            (
                opt(nom_distance_modifier),
                u32,
                opt(preceded(tag(" "), nom_fraction)),
            ),
            tag("SM"),
        ),
        |(modifier, whole, fraction)| StatuteMilesVisibility {
            whole: Some(whole),
            fraction,
            modifier,
        },
    );

    alt((fraction_only, whole)).parse(input)
}

fn nom_distance_modifier(input: &str) -> nom::IResult<&str, DistanceModifier> {
    alt((
        value(DistanceModifier::LessThan, tag("M")),
        value(DistanceModifier::GreaterThan, tag("P")),
    ))
    .parse(input)
}

fn nom_rvr(input: &str) -> nom::IResult<&str, Rvr> {
    map(
        preceded(
            tag("R"),
            separated_pair(
                alphanumeric1,
                tag("/"),
                (
                    opt(nom_distance_modifier),
                    OptionalData::optional_field(map_parser(take(4usize), all_consuming(u32))),
                    opt(alt((
                        value(Trend::Decreasing, tag("D")),
                        value(Trend::Increasing, tag("U")),
                        value(Trend::NoDistinctChange, tag("N")),
                    ))),
                ),
            ),
        ),
        |(runway, (distance_modifier, value, comment))| Rvr {
            runway: runway.to_string(),
            value,
            distance_modifier,
            comment,
        },
    )
    .parse(input)
}

fn nom_cloud_coverage(input: &str) -> nom::IResult<&str, OptionalData<CloudCoverage, 3>> {
    OptionalData::optional_field(alt((
        value(CloudCoverage::Few, tag("FEW")),
        value(CloudCoverage::Scattered, tag("SCT")),
        value(CloudCoverage::Broken, tag("BKN")),
        value(CloudCoverage::Overcast, tag("OVC")),
    )))
    .parse(input)
}

fn nom_cloud_type(input: &str) -> nom::IResult<&str, OptionalData<String, 3>> {
    OptionalData::optional_field(map(alphanumeric1, |s: &str| s.to_string())).parse(input)
}

fn nom_cloud(input: &str) -> nom::IResult<&str, Cloud> {
    let (input, coverage) = nom_cloud_coverage.parse(input)?;
    let (input, height) = nom_cloud_height.parse(input)?;
    let (input, cloud_type) = opt(nom_cloud_type).parse(input)?;
    Ok((
        input,
        Cloud {
            coverage,
            height,
            cloud_type,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
}
