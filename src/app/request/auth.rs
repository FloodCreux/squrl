use crate::app::app::App;
use crate::models::auth::auth::Auth;
use tracing::info;

impl App<'_> {
	pub fn modify_request_auth(
		&mut self,
		collection_index: usize,
		request_index: usize,
		auth: Auth,
	) -> anyhow::Result<()> {
		let local_selected_request =
			self.get_request_as_local_from_indexes(&(collection_index, request_index));

		{
			let mut selected_request = local_selected_request.write();

			info!("Auth method set to \"{}\"", auth);

			selected_request.auth = auth;
		}

		self.save_collection_to_file(collection_index);

		Ok(())
	}

	pub fn modify_request_auth_basic_username(
		&mut self,
		collection_index: usize,
		request_index: usize,
		username: String,
	) {
		self.modify_auth_field(
			collection_index,
			request_index,
			"basic username",
			&username,
			|auth| {
				if let Auth::BasicAuth(basic) = auth {
					basic.username = username.clone();
				}
			},
		);
	}

	pub fn modify_request_auth_basic_password(
		&mut self,
		collection_index: usize,
		request_index: usize,
		password: String,
	) {
		self.modify_auth_field(
			collection_index,
			request_index,
			"basic password",
			&password,
			|auth| {
				if let Auth::BasicAuth(basic) = auth {
					basic.password = password.clone();
				}
			},
		);
	}

	pub fn modify_request_auth_bearer_token(
		&mut self,
		collection_index: usize,
		request_index: usize,
		token: String,
	) {
		self.modify_auth_field(
			collection_index,
			request_index,
			"bearer token",
			&token,
			|auth| {
				if let Auth::BearerToken(bearer) = auth {
					bearer.token = token.clone();
				}
			},
		);
	}

	pub fn modify_request_auth_jwt_secret(
		&mut self,
		collection_index: usize,
		request_index: usize,
		secret: String,
	) {
		self.modify_auth_field(
			collection_index,
			request_index,
			"JWT secret",
			&secret,
			|auth| {
				if let Auth::JwtToken(token) = auth {
					token.secret = secret.clone();
				}
			},
		);
	}

	pub fn modify_request_auth_jwt_payload(
		&mut self,
		collection_index: usize,
		request_index: usize,
		payload: String,
	) {
		self.modify_auth_field(
			collection_index,
			request_index,
			"JWT payload",
			&payload,
			|auth| {
				if let Auth::JwtToken(token) = auth {
					token.payload = payload.clone();
				}
			},
		);
	}

	pub fn modify_request_auth_digest_username(
		&mut self,
		collection_index: usize,
		request_index: usize,
		username: String,
	) {
		self.modify_auth_field(
			collection_index,
			request_index,
			"Digest username",
			&username,
			|auth| {
				if let Auth::Digest(digest) = auth {
					digest.username = username.clone();
				}
			},
		);
	}

	pub fn modify_request_auth_digest_password(
		&mut self,
		collection_index: usize,
		request_index: usize,
		password: String,
	) {
		self.modify_auth_field(
			collection_index,
			request_index,
			"Digest password",
			&password,
			|auth| {
				if let Auth::Digest(digest) = auth {
					digest.password = password.clone();
				}
			},
		);
	}

	pub fn modify_request_auth_digest_domains(
		&mut self,
		collection_index: usize,
		request_index: usize,
		domains: String,
	) {
		self.modify_auth_field(
			collection_index,
			request_index,
			"Digest domains",
			&domains,
			|auth| {
				if let Auth::Digest(digest) = auth {
					digest.domains = domains.clone();
				}
			},
		);
	}

	pub fn modify_request_auth_digest_realm(
		&mut self,
		collection_index: usize,
		request_index: usize,
		realm: String,
	) {
		self.modify_auth_field(
			collection_index,
			request_index,
			"Digest realm",
			&realm,
			|auth| {
				if let Auth::Digest(digest) = auth {
					digest.realm = realm.clone();
				}
			},
		);
	}

	pub fn modify_request_auth_digest_nonce(
		&mut self,
		collection_index: usize,
		request_index: usize,
		nonce: String,
	) {
		self.modify_auth_field(
			collection_index,
			request_index,
			"Digest nonce",
			&nonce,
			|auth| {
				if let Auth::Digest(digest) = auth {
					digest.nonce = nonce.clone();
				}
			},
		);
	}

	pub fn modify_request_auth_digest_opaque(
		&mut self,
		collection_index: usize,
		request_index: usize,
		opaque: String,
	) {
		self.modify_auth_field(
			collection_index,
			request_index,
			"Digest opaque",
			&opaque,
			|auth| {
				if let Auth::Digest(digest) = auth {
					digest.opaque = opaque.clone();
				}
			},
		);
	}

	fn modify_auth_field(
		&mut self,
		collection_index: usize,
		request_index: usize,
		field_name: &str,
		value: &str,
		mutate: impl FnOnce(&mut Auth),
	) {
		self.with_request_write(collection_index, request_index, |req| {
			info!("Auth {field_name} set to \"{value}\"");
			mutate(&mut req.auth);
		});
	}
}
