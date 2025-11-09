struct TestFile {
    inner: Box<dyn Iterator<Item = &'static str>>,
}

impl TestFile {
    fn new(contents: &'static str) -> Self {
        Self {
            inner: Box::new(
                contents
                    .lines()
                    .filter(|s| !s.is_empty() && !s.starts_with('#')),
            ),
        }
    }

    fn read_line(&mut self) -> Option<&'static str> {
        self.inner.next()
    }

    fn consume(&mut self, name: &str) -> Option<&'static str> {
        self.read_line().map(|s| {
            let prefix = format!("{} = ", name);
            assert!(s.starts_with(&prefix), "unexpected line");
            &s[prefix.len()..]
        })
    }

    fn consume_bytes(&mut self, name: &str) -> Option<Vec<u8>> {
        self.consume(name).map(|s| hex::decode(s).unwrap())
    }

    fn consume_u64(&mut self, name: &str) -> Option<u64> {
        self.consume(name).map(|s| s.parse().unwrap())
    }
}

macro_rules! test_file {
    ($file_prefix:literal, $file_suffix:literal) => {{
        TestFile::new(include_str!(concat!(
            "data/",
            $file_prefix,
            $file_suffix,
            ".rsp"
        )))
    }};
}

macro_rules! known_answer_test {
    ($name:ident, $ty:ty, $file_prefix:literal, $file_suffix:literal) => {
        #[test]
        fn $name() {
            let mut f = test_file!($file_prefix, $file_suffix);
            assert_eq!(
                format!("[L = {}]", <$ty>::DIGEST_SIZE),
                f.read_line().unwrap()
            );

            while let Some(length) = f.consume_u64("Len") {
                let mut input = f.consume_bytes("Msg").unwrap();
                if length == 0 {
                    assert_eq!(input, &[0]);
                    input.pop();
                }
                assert_eq!(length, (input.len() as u64) * 8);
                let digest = <$ty>::new().update(&input).finalize();
                assert_eq!(&digest[..], &f.consume_bytes("MD").unwrap()[..]);
            }
        }
    };
}

macro_rules! monte_carlo {
    ($ty:ty, $file_prefix:literal) => {
        #[test]
        fn monte_carlo() {
            let mut f = test_file!($file_prefix, "Monte");
            assert_eq!(
                format!("[L = {}]", <$ty>::DIGEST_SIZE),
                f.read_line().unwrap()
            );

            let mut seed: [u8; <$ty>::DIGEST_SIZE] = [0; <$ty>::DIGEST_SIZE];
            seed.copy_from_slice(&f.consume_bytes("Seed").unwrap());

            let mut expected_count = 0;
            while let Some(count) = f.consume_u64("COUNT") {
                assert_eq!(count, expected_count);
                expected_count += 1;

                let mut md_0 = seed;
                let mut md_1 = seed;
                let mut md_2 = seed;

                for _ in 0..1000 {
                    let md_i = <$ty>::new()
                        .update(&md_0)
                        .update(&md_1)
                        .update(&md_2)
                        .finalize();
                    md_0 = md_1;
                    md_1 = md_2;
                    md_2 = md_i;
                }

                assert_eq!(&md_2[..], &f.consume_bytes("MD").unwrap()[..]);
                seed = md_2;
            }
        }
    };
}

macro_rules! tests {
    ($mod:ident, $ty:ty, $file_prefix:literal) => {
        mod $mod {
            use super::TestFile;
            known_answer_test!(short_msg, $ty, $file_prefix, "ShortMsg");
            known_answer_test!(long_msg, $ty, $file_prefix, "LongMsg");
            monte_carlo!($ty, $file_prefix);
        }
    };
}

tests!(sha224, sha2_const::Sha224, "SHA224");
tests!(sha256, sha2_const::Sha256, "SHA256");
tests!(sha384, sha2_const::Sha384, "SHA384");
tests!(sha512, sha2_const::Sha512, "SHA512");
tests!(sha512_224, sha2_const::Sha512_224, "SHA512_224");
tests!(sha512_256, sha2_const::Sha512_256, "SHA512_256");
