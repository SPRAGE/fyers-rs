# Fixtures

Fixtures are added before implementation and are loaded from the repository-root
`fixtures/` directory by integration-test helpers under `tests/support/`.

Rules:

- Use fake credentials/tokens only.
- Keep request and response examples close to the Fyers docs.
- Every fixture should map to a local Fyers docs inventory row.
- Do not store real account data.
- Prefer one endpoint/message variant per directory once a row grows beyond a
	single request/response pair.

Top-level layout:

- `rest/` — REST request/query/response fixtures.
- `ws/` — WebSocket command, text-frame, JSON-event, and binary-event fixtures.

Suggested names:

- `request.json`
- `request_query.txt`
- `response_success.json`
- `response_error.json`
- `command_subscribe.json`
- `command_unsubscribe.json`
- `ping.txt`
- `event_*.json`
- `event_*.bin`
