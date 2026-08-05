include!(concat!(env!("OUT_DIR"), "/fixtures.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_script_materializes_expected_fixtures() -> Result<(), std::str::Utf8Error> {
        assert_eq!(ENTROPY.len(), 64);

        let jwt = std::str::from_utf8(TOKEN)?;
        assert_eq!(jwt.split('.').count(), 3);

        assert!(RSA_PKCS8_DER.starts_with(&[0x30]));
        assert!(
            RSA_PKCS8_PEM.starts_with(b"-----BEGIN PRIVATE KEY-----"),
            "pkcs8 pem fixture should have private key header"
        );

        Ok(())
    }
}
