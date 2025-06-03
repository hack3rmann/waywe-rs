// TODO(ArnoDarkrose): make everything I've made so far combinators instead of parsers

use nom::bytes::complete::{tag, take_while};
use nom::character::complete::line_ending;
use nom::error::{FromExternalError, ParseError};
use nom::multi::many;
use nom::sequence::{delimited, preceded, terminated};
use nom::{AsChar, IResult, Parser};

#[derive(Clone, Debug, Hash, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct Table {
    pub path: String,
    pub label: String,
    pub contentid: usize,
    pub totalsize: usize,
    pub update_clean_bytes_tally: usize,
    pub time_last_update_verified: usize,
    pub apps_ids: Vec<usize>,
}

#[derive(thiserror::Error, Debug)]
pub enum LibraryFoldersParseError<T> {
    #[error(transparent)]
    NomError(#[from] nom::Err<nom::error::Error<T>>),

    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
}

impl<'a> ParseError<&'a str> for LibraryFoldersParseError<&'a str> {
    fn from_error_kind(input: &'a str, kind: nom::error::ErrorKind) -> Self {
        Self::NomError(nom::Err::Error(nom::error::Error::from_error_kind(
            input, kind,
        )))
    }

    // TODO(ArnoDarkrose)
    fn append(_: &'a str, _: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

impl<'a, E> FromExternalError<&'a str, E> for LibraryFoldersParseError<&'a str> {
    fn from_external_error(input: &'a str, kind: nom::error::ErrorKind, _: E) -> Self {
        Self::from_error_kind(input, kind)
    }
}

fn delimited_in_quotes(
    elem: &str,
) -> impl Parser<&str, Output = &str, Error = LibraryFoldersParseError<&str>> {
    delimited(tag("\""), tag(elem), tag("\""))
}

fn table_element(
    elem: &str,
) -> impl Parser<&str, Output = &str, Error = LibraryFoldersParseError<&str>> {
    let path = preceded(take_while(AsChar::is_space), delimited_in_quotes(elem));

    let path = preceded(path, take_while(AsChar::is_space));
    let until_quotes = take_while(|c| c != '\"');
    let in_quotes = delimited(tag("\""), until_quotes, tag("\""));
    let path = preceded(path, in_quotes);

    terminated(path, line_ending)
}

fn apps_table<'a>()
-> impl Parser<&'a str, Output = Vec<usize>, Error = LibraryFoldersParseError<&'a str>> {
    let first_line = delimited(
        take_while(AsChar::is_space),
        delimited_in_quotes("apps"),
        take_while(AsChar::is_space),
    );
    let first_line = preceded(first_line, line_ending);

    let until_quotes = take_while(|c| c != '\"');
    let in_quotes = delimited(tag("\""), until_quotes, tag("\""));
    let app_id = preceded(take_while(AsChar::is_space), in_quotes);
    let app_id = terminated(app_id, take_while(|c| !AsChar::is_newline(c)));
    let app_id = terminated(app_id, line_ending);

    let apps_ids = many(0.., app_id)
        .map(|vec: Vec<&str>| {
            vec.into_iter()
                .map(|v| v.parse::<usize>())
                .collect::<Result<Vec<_>, _>>()
        })
        .map_res(|v| match v {
            Ok(v) => Ok(v),
            Err(e) => Err(LibraryFoldersParseError::<&str>::ParseIntError(e)),
        });

    let open_brace = delimited(
        take_while(AsChar::is_space),
        tag("{"),
        preceded(take_while(AsChar::is_space), line_ending),
    );

    let closing_brace = delimited(
        take_while(AsChar::is_space),
        tag("}"),
        preceded(take_while(AsChar::is_space), line_ending),
    );

    let apps_ids = delimited(open_brace, apps_ids, closing_brace);

    preceded(first_line, apps_ids)
}

fn table<'a>() -> impl Parser<&'a str, Output = Table, Error = LibraryFoldersParseError<&'a str>> {
    let open_brace = delimited(
        take_while(AsChar::is_space),
        tag("{"),
        preceded(take_while(AsChar::is_space), line_ending),
    );

    let closing_brace = delimited(
        take_while(AsChar::is_space),
        tag("}"),
        preceded(take_while(AsChar::is_space), line_ending),
    );

    (
        preceded(open_brace, table_element("path")),
        table_element("label"),
        table_element("contentid")
            .map(|v| v.parse())
            .map_res(|v| match v {
                Ok(v) => Ok(v),
                Err(e) => Err(LibraryFoldersParseError::<&str>::ParseIntError(e)),
            }),
        table_element("totalsize")
            .map(|v| v.parse())
            .map_res(|v| match v {
                Ok(v) => Ok(v),
                Err(e) => Err(LibraryFoldersParseError::<&str>::ParseIntError(e)),
            }),
        table_element("update_clean_bytes_tally")
            .map(|v| v.parse())
            .map_res(|v| match v {
                Ok(v) => Ok(v),
                Err(e) => Err(LibraryFoldersParseError::<&str>::ParseIntError(e)),
            }),
        table_element("time_last_update_verified")
            .map(|v| v.parse())
            .map_res(|v| match v {
                Ok(v) => Ok(v),
                Err(e) => Err(LibraryFoldersParseError::<&str>::ParseIntError(e)),
            }),
        terminated(apps_table(), closing_brace),
    )
        .map(
            |(
                path,
                label,
                contentid,
                totalsize,
                update_clean_bytes_tally,
                time_last_update_verified,
                apps_ids,
            )| {
                Table {
                    path: path.to_string(),
                    label: label.to_string(),
                    contentid,
                    totalsize,
                    update_clean_bytes_tally,
                    time_last_update_verified,
                    apps_ids,
                }
            },
        )
}

// NOTE: this doesn't work
pub fn library_folders(i: &str) -> IResult<&str, Vec<Table>, LibraryFoldersParseError<&str>> {
    let first_line = delimited_in_quotes("libraryfolders");

    let first_line = preceded(first_line, take_while(AsChar::is_space));
    let first_line = terminated(first_line, line_ending);

    let open_brace = delimited(
        take_while(AsChar::is_space),
        tag("{"),
        preceded(take_while(AsChar::is_space), line_ending),
    );

    let closing_brace = delimited(
        take_while(AsChar::is_space),
        tag("}"),
        preceded(take_while(AsChar::is_space), line_ending),
    );

    let library_folders = preceded(first_line, open_brace);

    let until_quotes = take_while(|c| c != '\"');
    let in_quotes = delimited(tag("\""), until_quotes, tag("\""));

    let element = preceded(take_while(AsChar::is_space), in_quotes);
    let element = preceded(element, take_while(AsChar::is_space));
    let element = preceded(element, line_ending);
    let element = preceded(element, table());
    let element = terminated(element, closing_brace);

    let elements = many(0.., element);

    let mut library_folders = preceded(library_folders, elements);

    let res = library_folders.parse(i)?;

    Ok(res)
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{BufReader, Read},
    };

    use super::*;

    #[test]
    // #[ignore = "requires test file"]
    fn test_table_elements() {
        let fd = File::open("table_elems_test").unwrap();
        let mut fd = BufReader::new(fd);

        let mut buf = String::new();
        fd.read_to_string(&mut buf).unwrap();

        let (rem, path) = table_element("path").parse(&buf).unwrap();
        assert_eq!(path, "/home/arno/.local/share/Steam");

        let (rem, label) = table_element("label").parse(rem).unwrap();
        assert_eq!(label, "");

        let (rem, contentid) = table_element("contentid").parse(rem).unwrap();
        assert_eq!(contentid, "2793914600858813338");

        let (rem, totalsize) = table_element("totalsize").parse(rem).unwrap();
        assert_eq!(totalsize, "0");

        let (rem, update_clean_bytes_tally) = table_element("update_clean_bytes_tally")
            .parse(rem)
            .unwrap();
        assert_eq!(update_clean_bytes_tally, "2147575279");

        let (_rem, time_last_update_verified) = table_element("time_last_update_verified")
            .parse(rem)
            .unwrap();
        assert_eq!(time_last_update_verified, "1746192238");
    }

    #[test]
    // #[ignore = "requires test file"]
    fn test_apps_table() {
        let fd = File::open("apps_table_test").unwrap();
        let mut fd = BufReader::new(fd);

        let mut buf = String::new();
        fd.read_to_string(&mut buf).unwrap();

        let (_rem, apps_ids) = apps_table().parse(&buf).unwrap();

        let gt = vec![228980, 1070560, 1391110, 1493710, 1628350, 2180100];
        assert_eq!(apps_ids, gt);
    }

    #[test]
    // #[ignore = "requires test file"]
    fn test_table() {
        let fd = File::open("table_test").unwrap();
        let mut fd = BufReader::new(fd);

        let mut buf = String::new();
        fd.read_to_string(&mut buf).unwrap();

        let (_rem, table) = table().parse(&buf).unwrap();

        let gt = Table {
            path: "/home/arno/.local/share/Steam".to_owned(),
            label: "".to_string(),
            contentid: 2793914600858813338,
            totalsize: 0,
            update_clean_bytes_tally: 2147575279,
            time_last_update_verified: 1746192238,
            apps_ids: vec![228980, 1070560, 1391110, 1493710, 1628350, 2180100],
        };

        assert_eq!(table, gt);
    }

    #[test]
    fn test_library_folders() {
        let fd = File::open("library_folders_test").unwrap();
        let mut fd = BufReader::new(fd);

        let mut buf = String::new();
        fd.read_to_string(&mut buf).unwrap();

        let (_rem, library_folders) = library_folders(&buf).unwrap();

        dbg!(library_folders);
    }
}
