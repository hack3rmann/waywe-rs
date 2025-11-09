macro_rules! tests {
    ($mod:ident, $ty:ty, $reference:ty) => {
        mod $mod {
            use proptest::{arbitrary::any, prop_assert_eq, proptest, strategy::Strategy};
            use sha2::Digest;

            fn hash_input() -> impl Strategy<Value = Vec<u8>> {
                proptest::collection::vec(any::<u8>(), 0..<$ty>::BLOCK_SIZE * 4)
            }

            proptest! {
                #[test]
                fn single_update(input in hash_input()) {
                    let digest = <$ty>::new().update(&input).finalize();
                    let expected = <$reference>::digest(&input);
                    prop_assert_eq!(&digest[..], &expected[..]);
                }

                #[test]
                fn multiple_updates(inputs in proptest::array::uniform4(hash_input())) {
                    let digest = inputs
                        .iter()
                        .fold(<$ty>::new(), |state, input| state.update(input))
                        .finalize();
                    let expected = <$reference>::digest(&inputs.concat());
                    prop_assert_eq!(&digest[..], &expected[..]);
                }
            }
        }
    };
}

tests!(sha224, sha2_const::Sha224, sha2::Sha224);
tests!(sha256, sha2_const::Sha256, sha2::Sha256);
tests!(sha384, sha2_const::Sha384, sha2::Sha384);
tests!(sha512, sha2_const::Sha512, sha2::Sha512);
tests!(sha512_224, sha2_const::Sha512_224, sha2::Sha512Trunc224);
tests!(sha512_256, sha2_const::Sha512_256, sha2::Sha512Trunc256);
