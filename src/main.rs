use std::fs;
use clap::Parser as ParserClap;

use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    character::complete::{anychar, multispace0},
    combinator::{opt, peek, recognize, verify},
    error::ParseError,
    multi::{many0, many_till},
    IResult, InputLength, Parser,
};

pub fn take_ni(s: &str, n: usize) -> IResult<&str, &str> {
    verify(take(n), |val: &str| val.parse::<u16>().is_ok())(s)
}

pub fn parse_line_generic<'a>(
    parse_date: &dyn Fn(&str) -> IResult<&str, &str>,
    s: &'a str,
) -> IResult<&'a str, (&'a str, &'a str, String)> {
    let (s, date) = recognize(parse_date)(s)?;
    let (s, _) = multispace0(s)?;
    let (s, log_level) = alt((tag("INFO"), tag("WARN"), tag("ERROR")))(s)?;
    let (s, _) = multispace0(s)?;
    let (s, s1) = many_till(anychar, peek(parse_date))(s)?;
    let s2 = s1.0.iter().collect::<String>();
    Ok((s, (log_level, date, s2)))
}

fn parse_many<I, O, E, F>(parser: F, content: I) -> Vec<O>
where
    I: Clone + InputLength + Copy,
    F: Parser<I, O, E>,
    E: ParseError<I>,
{
    let (_, s1) = many0(parser)(content).unwrap_or_else(|_| (content, vec![]));
    s1
}

fn take_date_aws(s: &str) -> IResult<&str, &str> {
    fn opt_date(s: &str) -> IResult<&str, &str> {
        let (s, _) = take_ni(s, 4)?;
        let (s, _) = tag("-")(s)?;
        let (s, _) = take_ni(s, 2)?;
        let (s, _) = tag("-")(s)?;
        let (s, _) = take_ni(s, 2)?;
        let (s, _) = multispace0(s)?;
        Ok((s, ""))
    }
    let (s, _) = multispace0(s)?;
    let (s, _) = opt(opt_date)(s)?;
    let (s, _) = take_ni(s, 2)?;
    let (s, _) = tag(":")(s)?;
    let (s, _) = take_ni(s, 2)?;
    let (s, _) = tag(":")(s)?;
    let (s, _) = take_ni(s, 2)?;
    let (s, _) = tag(",")(s)?;
    let (s, _) = take_ni(s, 3)?;
    Ok((s, ""))
}

fn parse_content(parse_date: &dyn Fn(&str) -> IResult<&str, &str>, content: &str) {
    let parse_line3_generic = |s| parse_line_generic(parse_date, s);
    let s2 = parse_many(parse_line3_generic, content);
    s2.iter()
        .filter(|(log_level, _, _)| *log_level == "ERROR")
        .for_each(|(log_level, date, content)| {
            println!("Item:{} date:{}", log_level, date);
            println!("\tContent:{}", content);
            println!("----------------------------------");
        });
}

#[derive(ParserClap)]
struct Cli {
    /// The path to the file to read
    #[clap(parse(from_os_str))]
    path: std::path::PathBuf,
}

fn main() {
    let args = Cli::parse();
    let file_name = args.path.as_os_str().to_str().unwrap();
    let contents = fs::read_to_string(file_name).expect("Something went wrong reading the file");
    parse_content(&take_date_aws, contents.as_str());
}
