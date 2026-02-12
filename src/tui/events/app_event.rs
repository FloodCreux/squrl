use crate::get_key_bindings;
use crate::tui::event_key_bindings::EventKeyBinding;

get_key_bindings! {
	#[derive(Debug, Clone)]
	pub enum AppEvent {
		/* Main Page */

		ExitApp(EventKeyBinding),

		MoveCollectionCursorUp(EventKeyBinding),
		MoveCollectionCursorDown(EventKeyBinding),

		SelectRequestOrExpandCollection(EventKeyBinding),
		ExpandCollection(EventKeyBinding),
		UnselectRequest(EventKeyBinding),

		CreateElement(EventKeyBinding),
		DeleteElement(EventKeyBinding),
		RenameElement(EventKeyBinding),
		DuplicateElement(EventKeyBinding),

		MoveElementUp(EventKeyBinding),
		MoveElementDown(EventKeyBinding),

		NextEnvironment(EventKeyBinding),
		DisplayEnvEditor(EventKeyBinding),
		DisplayCookies(EventKeyBinding),
		DisplayLogs(EventKeyBinding),

		GoBackToLastState(EventKeyBinding),

		/* Env */

		EditEnvVariable(EventKeyBinding),
		EnvVariablesMoveUp(EventKeyBinding),
		EnvVariablesMoveDown(EventKeyBinding),
		EnvVariablesMoveLeft(EventKeyBinding),
		EnvVariablesMoveRight(EventKeyBinding),
		CreateEnvVariable(EventKeyBinding),
		DeleteEnvVariable(EventKeyBinding),

		ModifyEnvVariable(EventKeyBinding),
		CancelModifyEnvVariable(EventKeyBinding),
		KeyEventModifyEnvVariable(EventKeyBinding),

		/* Cookies */

		CookiesMoveUp(EventKeyBinding),
		CookiesMoveDown(EventKeyBinding),
		CookiesMoveLeft(EventKeyBinding),
		CookiesMoveRight(EventKeyBinding),
		DeleteCookie(EventKeyBinding),

		/* Logs */

		ScrollLogsUp(EventKeyBinding),
		ScrollLogsDown(EventKeyBinding),
		ScrollLogsLeft(EventKeyBinding),
		ScrollLogsRight(EventKeyBinding),

		/* Collections */

		ChooseElementToCreateMoveCursorLeft(EventKeyBinding),
		ChooseElementToCreateMoveCursorRight(EventKeyBinding),
		SelectElementToCreate(EventKeyBinding),

		CreateNewCollection(EventKeyBinding),
		CancelCreateNewCollection(EventKeyBinding),
		KeyEventCreateNewCollection(EventKeyBinding),

		CreateNewRequest(EventKeyBinding),
		CancelCreateNewRequest(EventKeyBinding),
		CreatingRequestSelectInputUp(EventKeyBinding),
		CreatingRequestSelectInputDown(EventKeyBinding),
		CreatingRequestInputLeft(EventKeyBinding),
		CreatingRequestInputRight(EventKeyBinding),
		KeyEventCreateNewRequest(EventKeyBinding),

		DeletingCollectionMoveCursorLeft(EventKeyBinding),
		DeletingCollectionMoveCursorRight(EventKeyBinding),
		DeleteCollection(EventKeyBinding),

		DeletingRequestMoveCursorLeft(EventKeyBinding),
		DeletingRequestMoveCursorRight(EventKeyBinding),
		DeleteRequest(EventKeyBinding),

		RenameCollection(EventKeyBinding),
		CancelRenameCollection(EventKeyBinding),
		KeyEventRenameCollection(EventKeyBinding),

		RenameRequest(EventKeyBinding),
		CancelRenameRequest(EventKeyBinding),
		KeyEventRenameRequest(EventKeyBinding),

		/* Request */

		GoBackToRequestMenu(EventKeyBinding),

		EditUrl(EventKeyBinding),
		EditMethod(EventKeyBinding),

		EditSettings(EventKeyBinding),

		NextView(EventKeyBinding),

		SendRequest(EventKeyBinding),

		/* Param tabs */

		NextParamTab(EventKeyBinding),
		ModifyRequestAuthMethod(EventKeyBinding),
		ModifyRequestBodyContentType(EventKeyBinding),
		ModifyRequestMessageType(EventKeyBinding),

		EditRequestQueryParam(EventKeyBinding),
		RequestQueryParamsMoveUp(EventKeyBinding),
		RequestQueryParamsMoveDown(EventKeyBinding),
		RequestQueryParamsMoveLeft(EventKeyBinding),
		RequestQueryParamsMoveRight(EventKeyBinding),
		CreateRequestQueryParam(EventKeyBinding),
		DeleteRequestQueryParam(EventKeyBinding),
		ToggleRequestQueryParam(EventKeyBinding),
		DuplicateRequestQueryParam(EventKeyBinding),

		EditRequestAuth(EventKeyBinding),
		RequestAuthMoveUp(EventKeyBinding),
		RequestAuthMoveDown(EventKeyBinding),
		RequestAuthMoveLeft(EventKeyBinding),
		RequestAuthMoveRight(EventKeyBinding),

		EditRequestHeader(EventKeyBinding),
		RequestHeadersMoveUp(EventKeyBinding),
		RequestHeadersMoveDown(EventKeyBinding),
		RequestHeadersMoveLeft(EventKeyBinding),
		RequestHeadersMoveRight(EventKeyBinding),
		CreateRequestHeader(EventKeyBinding),
		DeleteRequestHeader(EventKeyBinding),
		ToggleRequestHeader(EventKeyBinding),
		DuplicateRequestHeader(EventKeyBinding),

		EditRequestBody(EventKeyBinding),
		RequestBodyTableMoveUp(EventKeyBinding),
		RequestBodyTableMoveDown(EventKeyBinding),
		RequestBodyTableMoveLeft(EventKeyBinding),
		RequestBodyTableMoveRight(EventKeyBinding),
		CreateRequestBodyTableElement(EventKeyBinding),
		DeleteRequestBodyTableElement(EventKeyBinding),
		ToggleRequestBodyTableElement(EventKeyBinding),
		DuplicateRequestBodyTableElement(EventKeyBinding),

		EditRequestMessage(EventKeyBinding),

		EditRequestScript(EventKeyBinding),
		// Move up or down
		RequestScriptMove(EventKeyBinding),

		/* Result tabs */

		NextResultTab(EventKeyBinding),

		ScrollResultUp(EventKeyBinding),
		ScrollResultDown(EventKeyBinding),
		ScrollResultLeft(EventKeyBinding),
		ScrollResultRight(EventKeyBinding),

		/* Others */

		CopyResponsePart(EventKeyBinding),

		/* Request export */

		ExportRequest(EventKeyBinding),
		RequestExportFormatMoveCursorLeft(EventKeyBinding),
		RequestExportFormatMoveCursorRight(EventKeyBinding),
		SelectRequestExportFormat(EventKeyBinding),

		ScrollRequestExportUp(EventKeyBinding),
		ScrollRequestExportDown(EventKeyBinding),
		ScrollRequestExportLeft(EventKeyBinding),
		ScrollRequestExportRight(EventKeyBinding),
		CopyRequestExport(EventKeyBinding),

		/* Request Text inputs */

		ModifyRequestUrl(EventKeyBinding),
		CancelEditRequestUrl(EventKeyBinding),
		KeyEventEditRequestUrl(EventKeyBinding),

		ModifyRequestQueryParam(EventKeyBinding),
		CancelEditRequestQueryParam(EventKeyBinding),
		KeyEventEditRequestQueryParam(EventKeyBinding),

		/* Auth */

		ModifyRequestAuthBasicUsername(EventKeyBinding),
		CancelEditRequestAuthBasicUsername(EventKeyBinding),
		KeyEventEditRequestAuthBasicUsername(EventKeyBinding),

		ModifyRequestAuthBasicPassword(EventKeyBinding),
		CancelEditRequestAuthBasicPassword(EventKeyBinding),
		KeyEventEditRequestAuthBasicPassword(EventKeyBinding),

		ModifyRequestAuthBearerToken(EventKeyBinding),
		CancelEditRequestAuthBearerToken(EventKeyBinding),
		KeyEventEditRequestAuthBearerToken(EventKeyBinding),

		ModifyRequestAuthJwtSecret(EventKeyBinding),
		CancelEditRequestAuthJwtSecret(EventKeyBinding),
		KeyEventEditRequestAuthJwtSecret(EventKeyBinding),

		ModifyRequestAuthJwtPayload(EventKeyBinding),
		CancelEditRequestAuthJwtPayload(EventKeyBinding),
		KeyEventEditRequestAuthJwtPayload(EventKeyBinding),

		ModifyRequestAuthDigestUsername(EventKeyBinding),
		CancelEditRequestAuthDigestUsername(EventKeyBinding),
		KeyEventEditRequestAuthDigestUsername(EventKeyBinding),

		ModifyRequestAuthDigestPassword(EventKeyBinding),
		CancelEditRequestAuthDigestPassword(EventKeyBinding),
		KeyEventEditRequestAuthDigestPassword(EventKeyBinding),

		ModifyRequestAuthDigestDomains(EventKeyBinding),
		CancelEditRequestAuthDigestDomains(EventKeyBinding),
		KeyEventEditRequestAuthDigestDomains(EventKeyBinding),

		ModifyRequestAuthDigestRealm(EventKeyBinding),
		CancelEditRequestAuthDigestRealm(EventKeyBinding),
		KeyEventEditRequestAuthDigestRealm(EventKeyBinding),

		ModifyRequestAuthDigestNonce(EventKeyBinding),
		CancelEditRequestAuthDigestNonce(EventKeyBinding),
		KeyEventEditRequestAuthDigestNonce(EventKeyBinding),

		ModifyRequestAuthDigestOpaque(EventKeyBinding),
		CancelEditRequestAuthDigestOpaque(EventKeyBinding),
		KeyEventEditRequestAuthDigestOpaque(EventKeyBinding),

		/* Headers */

		ModifyRequestHeader(EventKeyBinding),
		CancelEditRequestHeader(EventKeyBinding),
		KeyEventEditRequestHeader(EventKeyBinding),

		/* Body */

		ModifyRequestBodyTable(EventKeyBinding),
		CancelEditRequestBodyTable(EventKeyBinding),
		KeyEventEditRequestBodyTable(EventKeyBinding),

		ModifyRequestBodyFile(EventKeyBinding),
		CancelEditRequestBodyFile(EventKeyBinding),
		KeyEventEditRequestBodyFile(EventKeyBinding),

		ModifyRequestBodyString(EventKeyBinding),
		CancelEditRequestBodyString(EventKeyBinding),
		KeyEventEditRequestBodyString(EventKeyBinding),

		/* Websocket */

		ModifyRequestMessage(EventKeyBinding),
		CancelEditRequestMessage(EventKeyBinding),
		KeyEventEditRequestMessage(EventKeyBinding),

		/* Scripts */

		ModifyRequestPreRequestScript(EventKeyBinding),
		CancelEditRequestPreRequestScript(EventKeyBinding),
		KeyEventEditRequestPreRequestScript(EventKeyBinding),

		ModifyRequestPostRequestScript(EventKeyBinding),
		CancelEditRequestPostRequestScript(EventKeyBinding),
		KeyEventEditRequestPostRequestScript(EventKeyBinding),

		/* Settings */

		RequestSettingsMoveUp(EventKeyBinding),
		RequestSettingsMoveDown(EventKeyBinding),
		RequestSettingsToggleSettingLeft(EventKeyBinding),
		RequestSettingsToggleSettingRight(EventKeyBinding),
		ModifyRequestSettings(EventKeyBinding),

		/* Others */

		Documentation(EventKeyBinding),
	}
}
