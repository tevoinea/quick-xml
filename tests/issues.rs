//! Regression tests found in various issues.
//!
//! Name each module / test as `issue<GH number>` and keep sorted by issue number

use std::sync::mpsc;

use quick_xml::events::{BytesDecl, BytesStart, BytesText, Event};
use quick_xml::name::QName;
use quick_xml::reader::Reader;
use quick_xml::Error;

/// Regression test for https://github.com/tafia/quick-xml/issues/115
#[test]
fn issue115() {
    let mut r = Reader::from_str("<tag1 attr1='line 1\nline 2'></tag1>");
    match r.read_event() {
        Ok(Event::Start(e)) if e.name() == QName(b"tag1") => {
            let v = e.attributes().map(|a| a.unwrap().value).collect::<Vec<_>>();
            assert_eq!(v[0].clone().into_owned(), b"line 1\nline 2");
        }
        _ => (),
    }
}

/// Regression test for https://github.com/tafia/quick-xml/issues/360
#[test]
fn issue360() {
    let (tx, rx) = mpsc::channel::<Event>();

    std::thread::spawn(move || {
        let mut r = Reader::from_str("<tag1 attr1='line 1\nline 2'></tag1>");
        loop {
            let event = r.read_event().unwrap();
            if event == Event::Eof {
                tx.send(event).unwrap();
                break;
            } else {
                tx.send(event).unwrap();
            }
        }
    });
    for event in rx.iter() {
        println!("{:?}", event);
    }
}

/// Regression test for https://github.com/tafia/quick-xml/issues/514
mod issue514 {
    use super::*;
    use pretty_assertions::assert_eq;

    /// Check that there is no unexpected error
    #[test]
    fn no_mismatch() {
        let mut reader = Reader::from_str("<some-tag><html>...</html></some-tag>");

        let outer_start = BytesStart::new("some-tag");
        let outer_end = outer_start.to_end().into_owned();

        let html_start = BytesStart::new("html");
        let html_end = html_start.to_end().into_owned();

        assert_eq!(reader.read_event().unwrap(), Event::Start(outer_start));
        assert_eq!(reader.read_event().unwrap(), Event::Start(html_start));

        reader.check_end_names(false);

        assert_eq!(reader.read_text(html_end.name()).unwrap(), "...");

        reader.check_end_names(true);

        assert_eq!(reader.read_event().unwrap(), Event::End(outer_end));
        assert_eq!(reader.read_event().unwrap(), Event::Eof);
    }

    /// Canary check that legitimate error is reported
    #[test]
    fn mismatch() {
        let mut reader = Reader::from_str("<some-tag><html>...</html></other-tag>");

        let outer_start = BytesStart::new("some-tag");

        let html_start = BytesStart::new("html");
        let html_end = html_start.to_end().into_owned();

        assert_eq!(reader.read_event().unwrap(), Event::Start(outer_start));
        assert_eq!(reader.read_event().unwrap(), Event::Start(html_start));

        reader.check_end_names(false);

        assert_eq!(reader.read_text(html_end.name()).unwrap(), "...");

        reader.check_end_names(true);

        match reader.read_event() {
            Err(Error::EndEventMismatch { expected, found }) => {
                assert_eq!(expected, "some-tag");
                assert_eq!(found, "other-tag");
            }
            x => panic!(
                r#"Expected `Err(EndEventMismatch("some-tag", "other-tag"))`, but found {:?}"#,
                x
            ),
        }
        assert_eq!(reader.read_event().unwrap(), Event::Eof);
    }
}

/// Regression test for https://github.com/tafia/quick-xml/issues/604
mod issue604 {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn short() {
        let data = b"<?xml version=\"1.0\"?><!-->";
        let mut reader = Reader::from_reader(data.as_slice());
        let mut buf = Vec::new();
        assert_eq!(
            reader.read_event_into(&mut buf).unwrap(),
            Event::Decl(BytesDecl::new("1.0", None, None))
        );
        match reader.read_event_into(&mut buf) {
            Err(Error::UnexpectedEof(reason)) => assert_eq!(reason, "Comment"),
            x => panic!(
                r#"Expected `Err(UnexpectedEof("Comment"))`, but found {:?}"#,
                x
            ),
        }
        assert_eq!(reader.read_event_into(&mut buf).unwrap(), Event::Eof);
    }

    #[test]
    fn long() {
        let data = b"<?xml version=\"1.0\"?><!--->";
        let mut reader = Reader::from_reader(data.as_slice());
        let mut buf = Vec::new();
        assert_eq!(
            reader.read_event_into(&mut buf).unwrap(),
            Event::Decl(BytesDecl::new("1.0", None, None))
        );
        match reader.read_event_into(&mut buf) {
            Err(Error::UnexpectedEof(reason)) => assert_eq!(reason, "Comment"),
            x => panic!(
                r#"Expected `Err(UnexpectedEof("Comment"))`, but found {:?}"#,
                x
            ),
        }
        assert_eq!(reader.read_event_into(&mut buf).unwrap(), Event::Eof);
    }

    /// According to the grammar, `>` is allowed just in start of comment.
    /// See https://www.w3.org/TR/xml11/#sec-comments
    #[test]
    fn short_valid() {
        let data = b"<?xml version=\"1.0\"?><!-->-->";
        let mut reader = Reader::from_reader(data.as_slice());
        let mut buf = Vec::new();
        assert_eq!(
            reader.read_event_into(&mut buf).unwrap(),
            Event::Decl(BytesDecl::new("1.0", None, None))
        );
        assert_eq!(
            reader.read_event_into(&mut buf).unwrap(),
            Event::Comment(BytesText::from_escaped(">"))
        );
        assert_eq!(reader.read_event_into(&mut buf).unwrap(), Event::Eof);
    }

    /// According to the grammar, `->` is allowed just in start of comment.
    /// See https://www.w3.org/TR/xml11/#sec-comments
    #[test]
    fn long_valid() {
        let data = b"<?xml version=\"1.0\"?><!--->-->";
        let mut reader = Reader::from_reader(data.as_slice());
        let mut buf = Vec::new();
        assert_eq!(
            reader.read_event_into(&mut buf).unwrap(),
            Event::Decl(BytesDecl::new("1.0", None, None))
        );
        assert_eq!(
            reader.read_event_into(&mut buf).unwrap(),
            Event::Comment(BytesText::from_escaped("->"))
        );
        assert_eq!(reader.read_event_into(&mut buf).unwrap(), Event::Eof);
    }
}
