use std::borrow::Cow;

use chumsky::Parser;
use chumsky::prelude::*;
use chumsky::text::digits;

#[derive(Clone, Debug, Hash, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct LibraryFolder<'a> {
    pub path: Cow<'a, str>,
    pub label: Cow<'a, str>,
    pub contentid: usize,
    pub totalsize: usize,
    pub update_clean_bytes_tally: usize,
    pub time_last_update_verified: usize,
    pub apps_ids: Vec<usize>,
}

fn elem<'a>(elem: &'a str) -> impl Parser<'a, &'a str, &'a str, extra::Err<EmptyErr>> {
    let quote = just("\"");

    just(elem).delimited_by(quote, quote).padded().ignore_then(
        none_of("\"\t\n")
            .repeated()
            .to_slice()
            .delimited_by(quote, quote)
            .padded(),
    )
}

fn apps<'a>() -> impl Parser<'a, &'a str, Vec<usize>, extra::Err<EmptyErr>> {
    let quote = just("\"");

    let first_line = just("apps").delimited_by(quote, quote).padded();

    let line = digits(10)
        .to_slice()
        .padded()
        .delimited_by(quote, quote)
        .then_ignore(digits(10).delimited_by(quote, quote).padded());

    let res = line
        .repeated()
        .collect::<Vec<_>>()
        .try_map(|v, _| {
            v.into_iter()
                .map(|elem: &str| elem.parse::<usize>())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| EmptyErr::default())
        })
        .padded()
        .delimited_by(just("{"), just("}"))
        .padded();

    first_line.ignore_then(res)
}

fn table<'a>() -> impl Parser<'a, &'a str, LibraryFolder<'a>, extra::Err<EmptyErr>> {
    group((
        elem("path"),
        elem("label"),
        elem("contentid").try_map(|v, _| v.parse::<usize>().map_err(|_| EmptyErr::default())),
        elem("totalsize").try_map(|v, _| v.parse::<usize>().map_err(|_| EmptyErr::default())),
        elem("update_clean_bytes_tally")
            .try_map(|v, _| v.parse::<usize>().map_err(|_| EmptyErr::default())),
        elem("time_last_update_verified")
            .try_map(|v, _| v.parse::<usize>().map_err(|_| EmptyErr::default())),
        apps(),
    ))
    .map(
        |(
            path,
            label,
            contentid,
            totalsize,
            update_clean_bytes_tally,
            time_last_update_verified,
            apps,
        )| {
            LibraryFolder {
                path: Cow::Borrowed(path),
                label: Cow::Borrowed(label),
                contentid,
                totalsize,
                update_clean_bytes_tally,
                time_last_update_verified,
                apps_ids: apps,
            }
        },
    )
}

pub fn library_folders<'a>()
-> impl Parser<'a, &'a str, Vec<LibraryFolder<'a>>, extra::Err<EmptyErr>> {
    let quote = just("\"");

    let first_line = just("libraryfolders").delimited_by(quote, quote).padded();

    let elem = digits(10)
        .delimited_by(quote, quote)
        .padded()
        .ignore_then(table().padded().delimited_by(just("{"), just("}").padded()));

    first_line
        .ignore_then(
            elem.repeated()
                .at_least(1)
                .collect::<Vec<_>>()
                .padded()
                .delimited_by(just("{"), just("}")),
        )
        .padded()
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{BufReader, Read},
    };

    use super::*;

    #[test]
    fn test_elem() {
        let input = "\t\t\"path\"\t\t\"/home/arno/.local/share/Steam\"\n";

        let res = elem("path").parse(input).unwrap();

        assert_eq!(res, "/home/arno/.local/share/Steam");
    }

    #[test]
    fn test_apps() {
        let fd = File::open("tests/apps_table_test").unwrap();
        let mut fd = BufReader::new(fd);
        let mut input = String::new();
        fd.read_to_string(&mut input).unwrap();

        let res = apps().parse(&input).unwrap();

        let gt = vec![228980, 1070560, 1391110, 1493710, 1628350, 2180100];

        assert_eq!(res, gt);
    }

    #[test]
    fn test_apps_empty() {
        let fd = File::open("tests/apps_table_test_empty").unwrap();
        let mut fd = BufReader::new(fd);
        let mut input = String::new();
        fd.read_to_string(&mut input).unwrap();

        let res = apps().parse(&input).unwrap();

        let gt = vec![];

        assert_eq!(res, gt);
    }

    #[test]
    fn test_table() {
        let fd = File::open("tests/table_test").unwrap();
        let mut fd = BufReader::new(fd);
        let mut input = String::new();
        fd.read_to_string(&mut input).unwrap();

        let res = table().parse(&input).unwrap();

        let gt = LibraryFolder {
            path: Cow::Borrowed("/home/arno/.local/share/Steam"),
            label: Cow::Borrowed(""),
            contentid: 2793914600858813338,
            totalsize: 0,
            update_clean_bytes_tally: 2147575279,
            time_last_update_verified: 1746192238,
            apps_ids: vec![228980, 1070560, 1391110, 1493710, 1628350, 2180100],
        };

        assert_eq!(gt.to_owned(), res.to_owned());
    }

    #[test]
    fn test_library_folders() {
        let fd = File::open("tests/libraryfolders.vdf").unwrap();
        let mut fd = BufReader::new(fd);
        let mut input = String::new();
        fd.read_to_string(&mut input).unwrap();

        let res = library_folders().parse(&input).unwrap();

        let gt = vec![
            LibraryFolder {
                path: Cow::Borrowed("/home/arno/.local/share/Steam"),
                label: Cow::Borrowed(""),
                contentid: 2793914600858813338,
                totalsize: 0,
                update_clean_bytes_tally: 780965114,
                time_last_update_verified: 1748464240,
                apps_ids: vec![228980, 1070560, 1391110, 1493710, 1628350, 2180100],
            },
            LibraryFolder {
                path: Cow::Borrowed("/home/arno/Games"),
                label: Cow::Borrowed(""),
                contentid: 2997758956146613868,
                totalsize: 510930190336,
                update_clean_bytes_tally: 8015536161,
                time_last_update_verified: 0,
                apps_ids: vec![],
            },
        ];

        assert_eq!(gt, res);
    }
}
