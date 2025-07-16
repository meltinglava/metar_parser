use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take},
    character::complete::u32,
    combinator::{all_consuming, map_parser, value},
};

use crate::optional_data::OptionalData;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pressure {
    pub value: OptionalData<u32, 4>,
    pub unit: PressureUnit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressureUnit {
    Hectopascals,
    InchesOfMercury,
}

pub(crate) fn nom_pressure(input: &str) -> IResult<&str, Pressure> {
    (
        nom_pressure_unit,
        OptionalData::optional_field(map_parser(take(4usize), all_consuming(u32))),
    )
        .map(|(unit, value)| Pressure { value, unit })
        .parse(input)
}

fn nom_pressure_unit(input: &str) -> IResult<&str, PressureUnit> {
    alt((
        value(PressureUnit::Hectopascals, tag("Q")),
        value(PressureUnit::InchesOfMercury, tag("A")),
    ))
    .parse(input)
}
