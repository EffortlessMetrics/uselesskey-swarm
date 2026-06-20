use rand_chacha10::ChaCha20Rng;
use rand_core10::{Rng, SeedableRng};
use uselesskey_core::Factory;

use crate::payload::{canonical_payload, stable_spec_bytes};
use crate::secret::build_secret;
use crate::signature::{perturb_last_hex_digit, sign, signature_header_name};
use crate::{
    DOMAIN_WEBHOOK_FIXTURE, NearMissScenario, NearMissWebhookFixture, WebhookFixture,
    WebhookPayloadSpec, WebhookProfile,
};

/// Extension trait to generate webhook fixtures from [`Factory`].
pub trait WebhookFactoryExt {
    /// Generate a webhook fixture for an explicit profile.
    fn webhook(
        &self,
        profile: WebhookProfile,
        label: impl AsRef<str>,
        payload_spec: WebhookPayloadSpec,
    ) -> WebhookFixture;

    /// Generate a GitHub webhook fixture.
    fn webhook_github(
        &self,
        label: impl AsRef<str>,
        payload_spec: WebhookPayloadSpec,
    ) -> WebhookFixture;

    /// Generate a Stripe webhook fixture.
    fn webhook_stripe(
        &self,
        label: impl AsRef<str>,
        payload_spec: WebhookPayloadSpec,
    ) -> WebhookFixture;

    /// Generate a Slack webhook fixture.
    fn webhook_slack(
        &self,
        label: impl AsRef<str>,
        payload_spec: WebhookPayloadSpec,
    ) -> WebhookFixture;
}

impl WebhookFactoryExt for Factory {
    fn webhook(
        &self,
        profile: WebhookProfile,
        label: impl AsRef<str>,
        payload_spec: WebhookPayloadSpec,
    ) -> WebhookFixture {
        let label = label.as_ref();
        let spec_bytes = stable_spec_bytes(profile, &payload_spec);
        let cached = self.get_or_init(DOMAIN_WEBHOOK_FIXTURE, label, &spec_bytes, "good", |seed| {
            build_fixture_from_seed(profile, label, payload_spec.clone(), seed.bytes())
        });
        cached.as_ref().clone()
    }

    fn webhook_github(
        &self,
        label: impl AsRef<str>,
        payload_spec: WebhookPayloadSpec,
    ) -> WebhookFixture {
        self.webhook(WebhookProfile::GitHub, label, payload_spec)
    }

    fn webhook_stripe(
        &self,
        label: impl AsRef<str>,
        payload_spec: WebhookPayloadSpec,
    ) -> WebhookFixture {
        self.webhook(WebhookProfile::Stripe, label, payload_spec)
    }

    fn webhook_slack(
        &self,
        label: impl AsRef<str>,
        payload_spec: WebhookPayloadSpec,
    ) -> WebhookFixture {
        self.webhook(WebhookProfile::Slack, label, payload_spec)
    }
}

impl WebhookFixture {
    /// Produce a stale-timestamp variant for replay-window tests.
    pub fn near_miss_stale_timestamp(&self, max_age_secs: i64) -> NearMissWebhookFixture {
        let stale_ts = self.timestamp - max_age_secs - 1;
        let mut f = self.with_timestamp(stale_ts);
        f.scenario = NearMissScenario::StaleTimestamp;
        f
    }

    /// Produce a wrong-secret variant for verifier mismatch tests.
    pub fn near_miss_wrong_secret(&self) -> NearMissWebhookFixture {
        let mut wrong_secret = self.secret.clone();
        wrong_secret.push_str("_wrong");
        let mut f = build_near_miss(
            self.profile,
            wrong_secret,
            self.payload.clone(),
            self.timestamp,
        );
        f.scenario = NearMissScenario::WrongSecret;
        f
    }

    /// Produce a tampered-payload variant for integrity tests.
    pub fn near_miss_tampered_payload(&self) -> NearMissWebhookFixture {
        let tampered = format!("{}{}", self.payload, "\n");
        let mut f = build_near_miss(self.profile, self.secret.clone(), tampered, self.timestamp);
        f.scenario = NearMissScenario::TamperedPayload;
        f
    }

    /// Produce a near-miss-signature variant for digest-comparison tests.
    ///
    /// The payload, secret, timestamp, and canonical signature input match the
    /// valid fixture exactly; only one hex digit of the signature header is
    /// flipped. A verifier must reject on the digest comparison itself rather
    /// than on a malformed header shape.
    pub fn near_miss_signature(&self) -> NearMissWebhookFixture {
        let (headers, signature_input) =
            sign(self.profile, &self.secret, &self.payload, self.timestamp);
        let mut f = NearMissWebhookFixture {
            scenario: NearMissScenario::NearMissSignature,
            profile: self.profile,
            secret: self.secret.clone(),
            payload: self.payload.clone(),
            headers,
            timestamp: self.timestamp,
            signature_input,
        };
        let header_name = signature_header_name(self.profile);
        if let Some(value) = f.headers.get(header_name) {
            let perturbed = perturb_last_hex_digit(value);
            f.headers.insert(header_name.to_string(), perturbed);
        }
        f
    }

    /// Produce a malformed-canonical-payload variant for canonicalization tests.
    ///
    /// The body is signed as-is but cannot be parsed as canonical JSON, so a
    /// verifier that canonicalizes the request before checking the digest
    /// rejects at the canonicalization step.
    pub fn near_miss_malformed_canonical_payload(&self) -> NearMissWebhookFixture {
        let malformed = malformed_canonical_payload(&self.payload);
        let mut f = build_near_miss(self.profile, self.secret.clone(), malformed, self.timestamp);
        f.scenario = NearMissScenario::MalformedCanonicalPayload;
        f
    }

    fn with_timestamp(&self, timestamp: i64) -> NearMissWebhookFixture {
        build_near_miss(
            self.profile,
            self.secret.clone(),
            self.payload.clone(),
            timestamp,
        )
    }
}

fn build_near_miss(
    profile: WebhookProfile,
    secret: String,
    payload: String,
    timestamp: i64,
) -> NearMissWebhookFixture {
    let (headers, signature_input) = sign(profile, &secret, &payload, timestamp);
    NearMissWebhookFixture {
        scenario: NearMissScenario::StaleTimestamp,
        profile,
        secret,
        payload,
        headers,
        timestamp,
        signature_input,
    }
}

/// Break a canonical payload so it can no longer be parsed as JSON.
///
/// Dropping the trailing `}` of an object body, or appending an unbalanced
/// brace otherwise, yields deterministic bytes that fail canonicalization
/// while staying clearly fixture-shaped.
fn malformed_canonical_payload(payload: &str) -> String {
    if let Some(trimmed) = payload.strip_suffix('}') {
        trimmed.to_string()
    } else {
        format!("{payload}{{")
    }
}

pub(crate) fn build_fixture_from_seed(
    profile: WebhookProfile,
    label: &str,
    payload_spec: WebhookPayloadSpec,
    seed: &[u8; 32],
) -> WebhookFixture {
    let mut rng = ChaCha20Rng::from_seed(*seed);
    let secret = build_secret(profile, &mut rng);
    let timestamp = 1_700_000_000_i64 + (rng.next_u32() as i64 % 200_000_000_i64);
    let payload = canonical_payload(profile, label, payload_spec, rng.next_u32());
    let (headers, signature_input) = sign(profile, &secret, &payload, timestamp);

    WebhookFixture {
        profile,
        secret,
        payload,
        headers,
        timestamp,
        signature_input,
    }
}
