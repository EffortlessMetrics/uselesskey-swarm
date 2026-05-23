use jsonwebtoken::{Algorithm, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uselesskey_core::Factory;
use uselesskey_hmac::{HmacFactoryExt, HmacSpec};
use uselesskey_jsonwebtoken::JwtKeyExt;
use uselesskey_rsa::{RsaFactoryExt, RsaSpec};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

#[test]
fn rs256_fixture_round_trips_and_wrong_key_rejects() {
    let fx = Factory::deterministic_from_str("external-jsonwebtoken-rs256");
    let issuer = fx.rsa("issuer", RsaSpec::rs256());
    let other_issuer = fx.rsa("other-issuer", RsaSpec::rs256());
    let claims = Claims {
        sub: "user-123".to_string(),
        exp: 2_000_000_000,
    };

    let token = encode(
        &Header::new(Algorithm::RS256),
        &claims,
        &issuer.encoding_key(),
    )
    .expect("rs256 token encodes");
    let decoded = decode::<Claims>(
        &token,
        &issuer.decoding_key(),
        &Validation::new(Algorithm::RS256),
    )
    .expect("rs256 token decodes");

    assert_eq!(decoded.claims, claims);
    assert!(
        decode::<Claims>(
            &token,
            &other_issuer.decoding_key(),
            &Validation::new(Algorithm::RS256),
        )
        .is_err()
    );
}

#[test]
fn hs256_fixture_round_trips_and_wrong_secret_rejects() {
    let fx = Factory::deterministic_from_str("external-jsonwebtoken-hs256");
    let secret = fx.hmac("session-secret", HmacSpec::hs256());
    let other_secret = fx.hmac("other-session-secret", HmacSpec::hs256());
    let claims = Claims {
        sub: "user-456".to_string(),
        exp: 2_000_000_000,
    };

    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &secret.encoding_key(),
    )
    .expect("hs256 token encodes");
    let decoded = decode::<Claims>(
        &token,
        &secret.decoding_key(),
        &Validation::new(Algorithm::HS256),
    )
    .expect("hs256 token decodes");

    assert_eq!(decoded.claims, claims);
    assert!(
        decode::<Claims>(
            &token,
            &other_secret.decoding_key(),
            &Validation::new(Algorithm::HS256),
        )
        .is_err()
    );
}

#[test]
fn verifier_policy_rejects_unexpected_algorithm_family() {
    let fx = Factory::deterministic_from_str("external-jsonwebtoken-alg-policy");
    let rsa = fx.rsa("issuer", RsaSpec::rs256());
    let secret = fx.hmac("session-secret", HmacSpec::hs256());
    let claims = Claims {
        sub: "user-789".to_string(),
        exp: 2_000_000_000,
    };

    let hs256_token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &secret.encoding_key(),
    )
    .expect("hs256 token encodes");

    assert!(
        decode::<Claims>(
            &hs256_token,
            &rsa.decoding_key(),
            &Validation::new(Algorithm::RS256),
        )
        .is_err()
    );
}
