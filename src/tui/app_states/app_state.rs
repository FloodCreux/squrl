use strum::Display;

#[derive(Copy, Clone, PartialEq, Default, Display)]
pub enum AppState {
	#[default]
	#[strum(to_string = "Main menu")]
	Normal,

	/* Env */
	#[strum(to_string = "Displaying environment editor")]
	DisplayingEnvEditor,

	#[strum(to_string = "Editing env variable")]
	EditingEnvVariable,

	/* Cookies */
	#[strum(to_string = "Displaying cookies")]
	DisplayingCookies,

	/* Logs */
	#[strum(to_string = "Displaying logs")]
	DisplayingLogs,

	/* Collections */
	#[strum(to_string = "Choosing an element to create")]
	ChoosingElementToCreate,

	#[strum(to_string = "Creating new collection")]
	CreatingNewCollection,

	#[strum(to_string = "Creating new request")]
	CreatingNewRequest,

	#[strum(to_string = "Deleting collection")]
	DeletingCollection,

	#[strum(to_string = "Deleting request")]
	DeletingRequest,

	#[strum(to_string = "Renaming collection")]
	RenamingCollection,

	#[strum(to_string = "Renaming request")]
	RenamingRequest,

	/* Request */
	#[strum(to_string = "Request menu")]
	SelectedRequest,

	#[strum(to_string = "Editing request URL")]
	EditingRequestUrl,

	#[strum(to_string = "Editing request param")]
	EditingRequestParam,

	#[strum(to_string = "Editing request auth username")]
	EditingRequestAuthBasicUsername,

	#[strum(to_string = "Editing request auth password")]
	EditingRequestAuthBasicPassword,

	#[strum(to_string = "Editing request auth bearer token")]
	EditingRequestAuthBearerToken,

	#[strum(to_string = "Editing request JWT secret")]
	EditingRequestAuthJwtSecret,

	#[strum(to_string = "Editing request JWT payload")]
	EditingRequestAuthJwtPayload,

	#[strum(to_string = "Editing request digest username")]
	EditingRequestAuthDigestUsername,

	#[strum(to_string = "Editing request digest password")]
	EditingRequestAuthDigestPassword,

	#[strum(to_string = "Editing request digest domains")]
	EditingRequestAuthDigestDomains,

	#[strum(to_string = "Editing request digest realm")]
	EditingRequestAuthDigestRealm,

	#[strum(to_string = "Editing request digest nonce")]
	EditingRequestAuthDigestNonce,

	#[strum(to_string = "Editing request digest opaque")]
	EditingRequestAuthDigestOpaque,

	#[strum(to_string = "Editing request header")]
	EditingRequestHeader,

	#[strum(to_string = "Editing request body (Form)")]
	EditingRequestBodyTable,

	#[strum(to_string = "Editing request body (File)")]
	EditingRequestBodyFile,

	#[strum(to_string = "Editing request body (Text)")]
	EditingRequestBodyString,

	#[strum(to_string = "Editing request message")]
	EditingRequestMessage,

	#[strum(to_string = "Editing pre-request script")]
	EditingPreRequestScript,

	#[strum(to_string = "Editing post-request script")]
	EditingPostRequestScript,

	#[strum(to_string = "Editing request settings")]
	EditingRequestSettings,

	#[strum(to_string = "Choosing request export format")]
	ChoosingRequestExportFormat,

	#[strum(to_string = "Displaying request export")]
	DisplayingRequestExport,
}
