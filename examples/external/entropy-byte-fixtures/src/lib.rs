use uselesskey_core::Factory;
use uselesskey_entropy::EntropyFactoryExt;

#[test]
fn entropy_fixtures_exercise_deterministic_byte_paths() {
    let fx = Factory::deterministic_from_str("external-entropy-byte-fixtures");
    let fixture = fx.entropy("api-token-placeholder");

    let first = fixture.bytes(32);
    let second = fixture.bytes(32);
    let longer = fixture.bytes(64);

    assert_eq!(fixture.label(), "api-token-placeholder");
    assert_eq!(fixture.variant(), "good");
    assert_eq!(first, second);
    assert_eq!(first.len(), 32);
    assert_eq!(longer.len(), 64);
    assert_ne!(first.as_slice(), &longer[..32]);
}

#[test]
fn entropy_fixtures_exercise_variants_and_fill_paths() {
    let fx = Factory::deterministic_from_str("external-entropy-variants");
    let fixture = fx.entropy("session-cookie-shape");

    let good = fixture.bytes(48);
    let rotated = fixture.bytes_with_variant(48, "rotated");
    let mut filled = [0u8; 48];

    fixture.fill_bytes(&mut filled);

    assert_eq!(good.as_slice(), &filled);
    assert_ne!(good, rotated);

    let mut rotated_filled = [0u8; 48];
    fixture.fill_bytes_with_variant(&mut rotated_filled, "rotated");
    assert_eq!(rotated.as_slice(), &rotated_filled);
}

#[test]
fn entropy_fixture_debug_output_omits_generated_bytes() {
    let fx = Factory::deterministic_from_str("external-entropy-debug");
    let fixture = fx.entropy("debug-placeholder");
    let bytes = fixture.bytes(16);
    let debug = format!("{fixture:?}");

    assert!(debug.contains("EntropyFixture"));
    assert!(debug.contains("debug-placeholder"));
    assert!(!debug.contains(&format!("{bytes:?}")));
}
