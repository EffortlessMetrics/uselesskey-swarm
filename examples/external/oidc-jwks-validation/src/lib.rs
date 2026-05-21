use std::collections::BTreeSet;

use serde_json::Value;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum JwksValidationError {
    MissingKeys,
    EmptyKeys,
    MissingKid,
    DuplicateKey,
    DuplicateKid,
    WrongKty,
    UnsupportedAlg,
    MissingMaterial,
    MalformedMaterial,
}

pub fn validate_oidc_jwks(jwks: &Value) -> Result<(), JwksValidationError> {
    let keys = jwks
        .get("keys")
        .and_then(Value::as_array)
        .ok_or(JwksValidationError::MissingKeys)?;
    if keys.is_empty() {
        return Err(JwksValidationError::EmptyKeys);
    }

    let mut kids = BTreeSet::new();
    let mut material = BTreeSet::new();
    for key in keys {
        let kid = key
            .get("kid")
            .and_then(Value::as_str)
            .filter(|kid| !kid.is_empty())
            .ok_or(JwksValidationError::MissingKid)?;

        if key.get("kty").and_then(Value::as_str) != Some("RSA") {
            return Err(JwksValidationError::WrongKty);
        }
        if key.get("alg").and_then(Value::as_str) != Some("RS256") {
            return Err(JwksValidationError::UnsupportedAlg);
        }
        let n = key
            .get("n")
            .and_then(Value::as_str)
            .ok_or(JwksValidationError::MissingMaterial)?;
        let e = key
            .get("e")
            .and_then(Value::as_str)
            .ok_or(JwksValidationError::MissingMaterial)?;
        if !is_unpadded_base64url(n) || !is_unpadded_base64url(e) {
            return Err(JwksValidationError::MalformedMaterial);
        }

        if !material.insert(format!("{kid}:{n}:{e}")) {
            return Err(JwksValidationError::DuplicateKey);
        }
        if !kids.insert(kid.to_string()) {
            return Err(JwksValidationError::DuplicateKid);
        }
    }

    Ok(())
}

fn is_unpadded_base64url(value: &str) -> bool {
    !value.is_empty()
        && !value.contains('=')
        && value
            .bytes()
            .all(|byte| matches!(byte, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_'))
}
