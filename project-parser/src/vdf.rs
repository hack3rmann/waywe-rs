use nom::bytes::complete::{tag, take_while};
use nom::character::complete::line_ending;
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

fn delimited_in_quotes(
    elem: &str,
) -> impl Parser<&str, Output = &str, Error = nom::error::Error<&str>> {
    delimited(tag("\""), tag(elem), tag("\""))
}

fn table_element(elem: &str) -> impl Parser<&str, Output = &str, Error = nom::error::Error<&str>> {
    let path = preceded(take_while(AsChar::is_space), delimited_in_quotes(elem));

    let path = preceded(path, take_while(AsChar::is_space));
    let until_quotes = take_while(|c| c != '\"');
    let in_quotes = delimited(tag("\""), until_quotes, tag("\""));
    let path = preceded(path, in_quotes);

    terminated(path, line_ending)
}

fn apps_table(i: &str) -> IResult<&str, Vec<usize>, LibraryFoldersParseError<&str>> {
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

    let apps_ids = many(0.., app_id);

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

    let mut apps_ids = preceded(first_line, apps_ids);

    let (rem, apps_ids): (_, Vec<&str>) = apps_ids
        .parse(i)
        .map_err(|v| nom::Err::Error(LibraryFoldersParseError::NomError(v)))?;

    let apps_ids: Result<Vec<usize>, _> = apps_ids.into_iter().map(|v| v.parse()).collect();
    let apps_ids =
        apps_ids.map_err(|e| nom::Err::Error(LibraryFoldersParseError::ParseIntError(e)))?;

    Ok((rem, apps_ids))
}

pub fn table(i: &str) -> IResult<&str, Table, LibraryFoldersParseError<&str>> {
    // TODO(ArnoDarkrose): implement the rest of the function

    let open_brace = delimited(
        take_while(AsChar::is_space),
        tag("{"),
        preceded(take_while(AsChar::is_space), line_ending),
    );

    let mut closing_brace = delimited(
        take_while(AsChar::is_space),
        tag("}"),
        preceded(take_while(AsChar::is_space), line_ending),
    );

    let (
        rem,
        (path, label, contentid, totalsize, update_clean_bytes_tally, time_last_update_verified),
    ) = (
        preceded(open_brace, table_element("path")),
        table_element("label"),
        table_element("contentid"),
        table_element("totalsize"),
        table_element("update_clean_bytes_tally"),
        table_element("time_last_update_verified"),
    )
        .parse(i)
        .map_err(|v| nom::Err::Error(LibraryFoldersParseError::NomError(v)))?;

    let (rem, apps_ids) = apps_table(rem)?;

    let (rem, _) = closing_brace
        .parse(rem)
        .map_err(|v| nom::Err::Error(LibraryFoldersParseError::NomError(v)))?;

    let contentid: usize = contentid
        .parse()
        .map_err(|v| nom::Err::Error(LibraryFoldersParseError::ParseIntError(v)))?;
    let totalsize: usize = totalsize
        .parse()
        .map_err(|v| nom::Err::Error(LibraryFoldersParseError::ParseIntError(v)))?;
    let update_clean_bytes_tally: usize = update_clean_bytes_tally
        .parse()
        .map_err(|v| nom::Err::Error(LibraryFoldersParseError::ParseIntError(v)))?;
    let time_last_update_verified: usize = time_last_update_verified
        .parse()
        .map_err(|v| nom::Err::Error(LibraryFoldersParseError::ParseIntError(v)))?;

    let res = Table {
        path: path.to_owned(),
        label: label.to_owned(),
        contentid,
        totalsize,
        update_clean_bytes_tally,
        time_last_update_verified,
        apps_ids,
    };

    Ok((rem, res))
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{BufReader, Read},
    };

    use super::*;

    #[test]
    #[ignore = "requires test file"]
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
    #[ignore = "requires test file"]
    fn test_apps_table() {
        let fd = File::open("apps_table_test").unwrap();
        let mut fd = BufReader::new(fd);

        let mut buf = String::new();
        fd.read_to_string(&mut buf).unwrap();

        let (_rem, apps_ids) = apps_table(&buf).unwrap();

        let gt = vec![228980, 1070560, 1391110, 1493710, 1628350, 2180100];

        assert_eq!(apps_ids, gt);
    }

    #[test]
    #[ignore = "requires test file"]
    fn test_table() {
        let fd = File::open("table_test").unwrap();
        let mut fd = BufReader::new(fd);

        let mut buf = String::new();
        fd.read_to_string(&mut buf).unwrap();

        let (_rem, table) = table(&buf).unwrap();

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
}
