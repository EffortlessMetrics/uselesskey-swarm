use uselesskey::{Factory, NegativeToken, RsaFactoryExt, RsaSpec, TokenFactoryExt, TokenSpec};

#[test]
fn facade_generates_positive_rsa_jwk_fixture() {
    let fx = Factory::deterministic_from_str("external-rust-test-fixtures");
    let issuer = fx.rsa("issuer", RsaSpec::rs256());
    let issuer_again = fx.rsa("issuer", RsaSpec::rs256());

    assert_eq!(issuer.kid(), issuer_again.kid());
    assert_eq!(issuer.public_jwk().to_value()["kty"], "RSA");
    assert_eq!(issuer.public_jwk().to_value()["alg"], "RS256");
}

#[test]
fn facade_generates_negative_token_shape_for_parser_tests() {
    let fx = Factory::deterministic_from_str("external-rust-test-fixtures");
    let token = fx.token("api", TokenSpec::api_key());
    let near_miss = token.negative_value(NegativeToken::NearMissApiKey);

    assert!(token.value().starts_with("uk_test_"));
    assert!(!near_miss.starts_with("uk_test_"));
    assert!(example_api_key_parser_accepts(token.value()));
    assert!(!example_api_key_parser_accepts(&near_miss));
}

fn example_api_key_parser_accepts(value: &str) -> bool {
    value.starts_with("uk_test_")
}

#[test]
fn facade_generates_jwt_claim_negatives_for_validator_tests() {
    let fx = Factory::deterministic_from_str("external-rust-test-fixtures");
    let token = fx.token("issuer", TokenSpec::oauth_access_token());

    assert_eq!(
        example_jwt_claim_validator_accepts(token.value(), "tests"),
        Ok(())
    );

    let alg_none = token.negative_value(NegativeToken::AlgNone);
    assert_eq!(
        example_jwt_claim_validator_accepts(&alg_none, "tests"),
        Err(JwtExampleValidationError::HeaderPolicy)
    );

    let bad_audience = token.negative_value(NegativeToken::BadAudience);
    assert_eq!(
        example_jwt_claim_validator_accepts(&bad_audience, "tests"),
        Err(JwtExampleValidationError::Audience)
    );
}

#[test]
fn facade_generates_jwt_parser_negatives_for_validator_tests() {
    let fx = Factory::deterministic_from_str("external-rust-test-fixtures");
    let token = fx.token("issuer", TokenSpec::oauth_access_token());

    let bad_segments = token.negative_value(NegativeToken::MalformedJwtSegmentCount);
    assert_eq!(
        example_jwt_claim_validator_accepts(&bad_segments, "tests"),
        Err(JwtExampleValidationError::SegmentCount)
    );

    let bad_base64url = token.negative_value(NegativeToken::BadBase64UrlSegment);
    assert_eq!(
        example_jwt_claim_validator_accepts(&bad_base64url, "tests"),
        Err(JwtExampleValidationError::BadBase64Url)
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JwtExampleValidationError {
    SegmentCount,
    BadBase64Url,
    HeaderPolicy,
    Audience,
}

fn example_jwt_claim_validator_accepts(
    value: &str,
    expected_audience: &str,
) -> Result<(), JwtExampleValidationError> {
    let parts: Vec<_> = value.split('.').collect();
    if parts.len() != 3 {
        return Err(JwtExampleValidationError::SegmentCount);
    }

    let header = decode_base64url_utf8(parts[0])?;
    if !header.contains("\"alg\":\"RS256\"") || header.contains("\"alg\":\"none\"") {
        return Err(JwtExampleValidationError::HeaderPolicy);
    }

    let payload = decode_base64url_utf8(parts[1])?;
    let expected = format!("\"aud\":\"{expected_audience}\"");
    if !payload.contains(&expected) {
        return Err(JwtExampleValidationError::Audience);
    }

    Ok(())
}

fn decode_base64url_utf8(segment: &str) -> Result<String, JwtExampleValidationError> {
    String::from_utf8(decode_base64url_no_pad(segment)?)
        .map_err(|_| JwtExampleValidationError::BadBase64Url)
}

fn decode_base64url_no_pad(segment: &str) -> Result<Vec<u8>, JwtExampleValidationError> {
    if segment.is_empty() || segment.len() % 4 == 1 {
        return Err(JwtExampleValidationError::BadBase64Url);
    }

    let mut out = Vec::new();
    let mut buffer = 0u32;
    let mut bits = 0u8;

    for byte in segment.bytes() {
        let value = match byte {
            b'A'..=b'Z' => u32::from(byte - b'A'),
            b'a'..=b'z' => u32::from(byte - b'a') + 26,
            b'0'..=b'9' => u32::from(byte - b'0') + 52,
            b'-' => 62,
            b'_' => 63,
            _ => return Err(JwtExampleValidationError::BadBase64Url),
        };

        buffer = (buffer << 6) | value;
        bits += 6;

        while bits >= 8 {
            bits -= 8;
            out.push(((buffer >> bits) & 0xff) as u8);
        }
    }

    if bits > 0 {
        let trailing_mask = (1u32 << bits) - 1;
        if buffer & trailing_mask != 0 {
            return Err(JwtExampleValidationError::BadBase64Url);
        }
    }

    Ok(out)
}
