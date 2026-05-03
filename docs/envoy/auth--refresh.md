# Token Refresh

Exchange a refresh token for a new access token.

## Request

```sh
curl https://app.envoy.com/a/auth/v0/token \
  -d 'grant_type=refresh_token' \
  -d 'refresh_token=<your-refresh-token>' \
  -d 'client_id=<your-client-id>'
```

## Response

```json
{
  "token_type": "Bearer",
  "access_token": "<new-access-token>",
  "expires_in": 86400,
  "refresh_token": "<new-refresh-token>",
  "refresh_token_expires_in": 2592000
}
```
