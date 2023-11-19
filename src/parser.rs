extern crate nom;
extern crate nom_locate;

use nom::branch::alt;
use nom::bytes::complete::{tag, take_until, take_while};
use nom::character::complete::multispace0;
use nom::multi::many0;
use nom::sequence::pair;
use nom::IResult;
use nom_locate::{position, LocatedSpan};

type Span<'a> = LocatedSpan<&'a str>;

// TODO: make kind more dynamic for users
// # Overview
//
// Not sure how to make this more dynamic so that even a ergency level could be created.
// For example
// ```rust
// // TODO: would be just a normal ergency level
// // ! TODO: would be a major ergency level
// // !!!! TODO: could be a critical level indicated by the amount of '!'
// ```
//
// also if a todo is made inside a todo maybe that could refer to the parent todo it was created
// in.

#[derive(Debug, Clone, Copy)]
pub enum Kind {
    Todo,
    Fixme,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Token<'a> {
    pub position: Span<'a>,
    pub kind: Kind,
    pub title: &'a str,
    pub message: Vec<&'a str>,
}

impl<'a> Token<'a> {
    pub fn body(&self, username: &str, repo: &str, branch: &str, filename: &str) -> String {
        let filename = filename
            .split(repo)
            .last()
            .map(|s| {
                if !s.starts_with('/') {
                    return format!("/{s}");
                }
                s.to_string()
            })
            .unwrap_or_default();
        self.message
            .iter()
            .map(|s| s.trim())
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
            + &format!(
                "[{:?}](http://github.com/{username}/{repo}/blob/{branch}{filename}#L{})",
                self.kind,
                self.position.location_line()
            )
    }
}

fn parse_kind(s: Span) -> IResult<Span, Kind> {
    let (s, _) = tag("//")(s)?;
    let (s, _) = multispace0(s)?;
    let (s, tag) = alt((tag("FIXME"), tag("TODO")))(s)?;
    match tag.fragment() {
        &"FIXME" => Ok((s, Kind::Fixme)),
        &"TODO" => Ok((s, Kind::Todo)),
        frag => unreachable!("unreachable {frag:?}"),
    }
}

fn parse_issue(mut s: Span) -> IResult<Span, Token> {
    let (span, (_, kind)) = loop {
        match pair(take_until("//"), parse_kind)(s) {
            Ok(r) => break r,
            Err(_) => {
                if s.fragment().is_empty() {
                    return Err(nom::Err::Error(nom::error::make_error(
                        s,
                        nom::error::ErrorKind::Eof,
                    )));
                } else {
                    let (new_span, _) = take_while(|c| c != '\n')(s)?;
                    let (new_span, _) = tag("\n")(new_span)?;
                    s = new_span;
                }
            }
        }
    };
    let (s, position) = position(span)?;
    let (s, _) = tag(":")(s)?;
    let (s, _) = multispace0(s)?;
    let (s, title) = take_while(|c| c != '\n')(s)?;
    let (s, message) = many0(comment)(s)?;

    Ok((
        s,
        Token {
            position,
            kind,
            title: title.fragment(),
            message,
        },
    ))
}

fn comment<'a>(s: Span<'a>) -> IResult<Span, &'a str> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("//")(s)?;
    let (s, line) = take_while(|c| c != '\n')(s)?;
    Ok((s, line.fragment()))
}

pub fn parser_file<'a>(src: &'a str) -> Vec<Token<'a>> {
    let mut tokens = Vec::new();
    let mut input = Span::new(&src);
    while !input.fragment().is_empty() {
        match parse_issue(input) {
            Ok((s, t)) => {
                input = s;
                tokens.push(t);
            }
            Err(_) => {
                break;
            }
        }
    }
    tokens
}
