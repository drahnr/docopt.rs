use std::collections::HashMap;

use serde::Deserialize;

use crate::{Docopt, ArgvMap, Error};
use crate::Value::{self, Switch, Plain};

fn get_args(doc: &str, argv: &[&'static str]) -> ArgvMap {
    let dopt = match Docopt::new(doc) {
        Err(err) => panic!("Invalid usage: {}", err),
        Ok(dopt) => dopt,
    };
    match dopt.argv(vec!["cmd"].iter().chain(argv.iter())).parse() {
        Err(err) => panic!("{}", err),
        Ok(vals) => vals,
    }
}

fn map_from_alist(alist: Vec<(&'static str, Value)>)
                 -> HashMap<String, Value> {
    alist.into_iter().map(|(k, v)| (k.to_string(), v)).collect()
}

fn same_args(expected: &HashMap<String, Value>, got: &ArgvMap) {
    for (k, ve) in expected.iter() {
        match got.map.find(k) {
            None => panic!("EXPECTED has '{}' but GOT does not.", k),
            Some(vg) => {
                assert!(ve == vg,
                        "{}: EXPECTED = '{:?}' != '{:?}' = GOT", k, ve, vg)
            }
        }
    }
    for (k, vg) in got.map.iter() {
        match got.map.find(k) {
            None => panic!("GOT has '{}' but EXPECTED does not.", k),
            Some(ve) => {
                assert!(vg == ve,
                        "{}: GOT = '{:?}' != '{:?}' = EXPECTED", k, vg, ve)
            }
        }
    }
}

macro_rules! test_expect(
    ($name:ident, $doc:expr, $args:expr, $expected:expr) => (
        #[test]
        fn $name() {
            let vals = get_args($doc, $args);
            let expected = map_from_alist($expected);
            same_args(&expected, &vals);
        }
    );
);

macro_rules! test_user_error(
    ($name:ident, $doc:expr, $args:expr) => (
        #[test]
        #[should_panic]
        fn $name() { get_args($doc, $args); }
    );
);

test_expect!(test_issue_13, "Usage: prog file <file>", &["file", "file"],
             vec![("file", Switch(true)),
                  ("<file>", Plain(Some("file".to_string())))]);

test_expect!(test_issue_129, "Usage: prog [options]

Options:
    --foo ARG   Foo foo.",
             &["--foo=a b"],
             vec![("--foo", Plain(Some("a b".into())))]);

#[test]
fn regression_issue_12() {
    const USAGE: &'static str = "
    Usage:
        whisper info <file>
        whisper update <file> <timestamp> <value>
        whisper mark <file> <value>
    ";

    #[derive(Deserialize, Debug)]
    struct Args {
        arg_file: String,
        cmd_info: bool,
        cmd_update: bool,
        arg_timestamp: u64,
        arg_value: f64,
    }

    let dopt: Args = Docopt::new(USAGE)
        .unwrap()
        .argv(&["whisper", "mark", "./p/blah", "100"])
        .deserialize()
        .unwrap();
    assert_eq!(dopt.arg_timestamp, 0);
}

#[test]
fn regression_issue_195() {
    const USAGE: &'static str = "
    Usage:
        slow [-abcdefghijklmnopqrs...]
    ";

    let argv = &["slow", "-abcdefghijklmnopqrs"];
    let dopt : Docopt = Docopt::new(USAGE).unwrap().argv(argv);

    dopt.parse().unwrap();
}

#[test]
fn regression_issue_219() {
    #[derive(Deserialize)]
    struct Args {
        arg_type: Vec<String>,
        arg_param: Vec<String>,
    }

    const USAGE: &'static str = "
    Usage:
        encode [-v <type> <param>]...
    ";

    let argv = &["encode", "-v", "bool", "true", "string", "foo"];
    let args: Args = Docopt::new(USAGE).unwrap().argv(argv).deserialize().unwrap();
    assert_eq!(args.arg_type, vec!["bool".to_owned(), "string".to_owned()]);
    assert_eq!(args.arg_param, vec!["true".to_owned(), "foo".to_owned()]);
}

#[test]
fn test_unit_struct() {
    const USAGE: &'static str = "
    Usage:
        cargo version [options]

    Options:
        -h, --help               Print this message
    ";

    #[derive(Deserialize)]
    struct Options;

    let argv = &["cargo", "version"];
    let dopt: Result<Options, Error>= Docopt::new(USAGE)
        .unwrap()
        .argv(argv)
        .deserialize();
    assert!(dopt.is_ok());
}



#[test]
fn post_double_dash_is_always_arg() {
    const USAGE: &str = "
    Usage:
        whisper info [--foo] [-- <arbitrary>...]
    ";

    #[derive(Deserialize, Debug)]
    struct Args {
        cmd_info: bool,
        flag_foo: bool,
        arg_arbitrary: Vec<String>,
    }

    const ARGS: &[&str] = &["whisper", "info", "--foo", "--", "./p/blah", "foo", "-fff", "--yy"];
    let dopt: Args = Docopt::new(USAGE)
        .expect("Parsing usage works. qed")
        .argv(ARGS)
        .deserialize()
        .expect("Deserializing of test works");
    assert_eq!(dopt.cmd_info, true);
    assert_eq!(dopt.flag_foo, true);
    assert_eq!(dopt.arg_arbitrary, &ARGS[4..]);
}

mod testcases;
mod suggestions;
