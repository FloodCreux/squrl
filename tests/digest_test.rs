use squrl::models::auth::digest::{Digest, DigestAlgorithm, DigestCharset, DigestError, DigestQop};

// --- Digest default values ---

#[test]
fn default_digest_has_empty_username() {
	let d = Digest::default();
	assert_eq!(d.username, "");
}

#[test]
fn default_digest_has_empty_password() {
	let d = Digest::default();
	assert_eq!(d.password, "");
}

#[test]
fn default_digest_has_empty_domains() {
	let d = Digest::default();
	assert_eq!(d.domains, "");
}

#[test]
fn default_digest_has_empty_realm() {
	let d = Digest::default();
	assert_eq!(d.realm, "");
}

#[test]
fn default_digest_has_empty_nonce() {
	let d = Digest::default();
	assert_eq!(d.nonce, "");
}

#[test]
fn default_digest_has_empty_opaque() {
	let d = Digest::default();
	assert_eq!(d.opaque, "");
}

#[test]
fn default_digest_is_not_stale() {
	let d = Digest::default();
	assert!(!d.stale);
}

#[test]
fn default_digest_user_hash_is_false() {
	let d = Digest::default();
	assert!(!d.user_hash);
}

#[test]
fn default_digest_nc_is_zero() {
	let d = Digest::default();
	assert_eq!(d.nc, 0);
}

// --- DigestAlgorithm defaults and Display ---

#[test]
fn default_digest_algorithm_is_md5() {
	assert_eq!(DigestAlgorithm::default().to_string(), "MD5");
}

#[test]
fn digest_algorithm_md5_display() {
	assert_eq!(DigestAlgorithm::MD5.to_string(), "MD5");
}

#[test]
fn digest_algorithm_md5_sess_display() {
	assert_eq!(DigestAlgorithm::MD5Sess.to_string(), "MD5-sess");
}

#[test]
fn digest_algorithm_sha256_display() {
	assert_eq!(DigestAlgorithm::SHA256.to_string(), "SHA-256");
}

#[test]
fn digest_algorithm_sha256_sess_display() {
	assert_eq!(DigestAlgorithm::SHA256Sess.to_string(), "SHA-256-sess");
}

#[test]
fn digest_algorithm_sha512_display() {
	assert_eq!(DigestAlgorithm::SHA512.to_string(), "SHA-512");
}

#[test]
fn digest_algorithm_sha512_sess_display() {
	assert_eq!(DigestAlgorithm::SHA512Sess.to_string(), "SHA-512-sess");
}

// --- DigestQop defaults and Display ---

#[test]
fn default_digest_qop_is_none() {
	assert_eq!(DigestQop::default().to_string(), "None");
}

#[test]
fn digest_qop_auth_display() {
	assert_eq!(DigestQop::Auth.to_string(), "auth");
}

#[test]
fn digest_qop_auth_int_display() {
	assert_eq!(DigestQop::AuthInt.to_string(), "auth-int");
}

// --- DigestCharset defaults and Display ---

#[test]
fn default_digest_charset_is_ascii() {
	assert_eq!(DigestCharset::default().to_string(), "ASCII");
}

#[test]
fn digest_charset_utf8_display() {
	assert_eq!(DigestCharset::UTF8.to_string(), "UTF8");
}

// --- Serialization ---

#[test]
fn digest_serialization_roundtrip() {
	let d = Digest {
		username: "user".to_string(),
		password: "pass".to_string(),
		realm: "test-realm".to_string(),
		nonce: "abc123".to_string(),
		..Default::default()
	};
	let json = serde_json::to_string(&d).unwrap();
	let deserialized: Digest = serde_json::from_str(&json).unwrap();
	assert_eq!(deserialized.username, "user");
	assert_eq!(deserialized.password, "pass");
	assert_eq!(deserialized.realm, "test-realm");
	assert_eq!(deserialized.nonce, "abc123");
}

#[test]
fn digest_nc_excluded_from_serialization() {
	let d = Digest {
		nc: 42,
		..Default::default()
	};
	let json = serde_json::to_string(&d).unwrap();
	assert!(!json.contains("\"nc\""));
}

#[test]
fn digest_nc_defaults_to_zero_on_deserialization() {
	let json = r#"{"username":"u","password":"p","domains":"","realm":"","nonce":"","opaque":"","stale":false,"algorithm":"MD5","qop":"None","user_hash":false,"charset":"ASCII"}"#;
	let d: Digest = serde_json::from_str(json).unwrap();
	assert_eq!(d.nc, 0);
}

#[test]
fn digest_roundtrip_preserves_all_fields() {
	let d = Digest {
		username: "admin".to_string(),
		password: "secret".to_string(),
		domains: "example.com".to_string(),
		realm: "protected".to_string(),
		nonce: "nonce123".to_string(),
		opaque: "opaque456".to_string(),
		stale: true,
		algorithm: DigestAlgorithm::SHA256,
		qop: DigestQop::Auth,
		user_hash: true,
		charset: DigestCharset::UTF8,
		nc: 0,
	};
	let json = serde_json::to_string(&d).unwrap();
	let deserialized: Digest = serde_json::from_str(&json).unwrap();
	assert_eq!(deserialized.username, "admin");
	assert_eq!(deserialized.password, "secret");
	assert_eq!(deserialized.domains, "example.com");
	assert_eq!(deserialized.realm, "protected");
	assert_eq!(deserialized.nonce, "nonce123");
	assert_eq!(deserialized.opaque, "opaque456");
	assert!(deserialized.stale);
	assert_eq!(deserialized.algorithm.to_string(), "SHA-256");
	assert_eq!(deserialized.qop.to_string(), "auth");
	assert!(deserialized.user_hash);
	assert_eq!(deserialized.charset.to_string(), "UTF8");
}

#[test]
fn digest_algorithm_serialization_roundtrip() {
	let variants = [
		DigestAlgorithm::MD5,
		DigestAlgorithm::MD5Sess,
		DigestAlgorithm::SHA256,
		DigestAlgorithm::SHA256Sess,
		DigestAlgorithm::SHA512,
		DigestAlgorithm::SHA512Sess,
	];
	for alg in variants {
		let json = serde_json::to_string(&alg).unwrap();
		let deserialized: DigestAlgorithm = serde_json::from_str(&json).unwrap();
		assert_eq!(alg.to_string(), deserialized.to_string());
	}
}

#[test]
fn digest_qop_serialization_roundtrip() {
	let variants = [DigestQop::None, DigestQop::Auth, DigestQop::AuthInt];
	for qop in variants {
		let json = serde_json::to_string(&qop).unwrap();
		let deserialized: DigestQop = serde_json::from_str(&json).unwrap();
		assert_eq!(qop.to_string(), deserialized.to_string());
	}
}

#[test]
fn digest_charset_serialization_roundtrip() {
	let variants = [DigestCharset::ASCII, DigestCharset::UTF8];
	for charset in variants {
		let json = serde_json::to_string(&charset).unwrap();
		let deserialized: DigestCharset = serde_json::from_str(&json).unwrap();
		assert_eq!(charset.to_string(), deserialized.to_string());
	}
}

// --- DigestError display messages ---

#[test]
fn digest_error_invalid_header_syntax_display() {
	let err = DigestError::InvalidHeaderSyntax("bad header".to_string());
	assert_eq!(err.to_string(), "Invalid header syntax: bad header");
}

#[test]
fn digest_error_missing_required_display() {
	let err = DigestError::MissingRequired("realm");
	assert_eq!(err.to_string(), "Missing required realm");
}

#[test]
fn digest_error_invalid_algorithm_display() {
	let err = DigestError::InvalidAlgorithm("BAD".to_string());
	assert_eq!(err.to_string(), "Invalid algorithm: BAD");
}

#[test]
fn digest_error_invalid_boolean_value_display() {
	let err = DigestError::InvalidBooleanValue("stale", "maybe".to_string());
	assert_eq!(err.to_string(), "Invalid boolean for stale: maybe");
}

#[test]
fn digest_error_invalid_charset_display() {
	let err = DigestError::InvalidCharset("LATIN1".to_string());
	assert_eq!(err.to_string(), "Invalid charset: LATIN1");
}

// --- Clone independence ---

#[test]
fn digest_clone_is_independent() {
	let original = Digest {
		username: "user1".to_string(),
		nonce: "nonce1".to_string(),
		..Default::default()
	};
	let mut cloned = original.clone();
	cloned.username = "user2".to_string();
	cloned.nonce = "nonce2".to_string();
	assert_eq!(original.username, "user1");
	assert_eq!(original.nonce, "nonce1");
}
