# /me Endpoint

Fetch the currently authenticated user's profile.

## Request

```sh
curl https://app.envoy.com/a/visitors/api/v2/users/me \
  -H 'Authorization: Bearer <access-token>'
```

## Response

```json
{
  "data": {
    "id": "123456",
    "type": "users",
    "attributes": {
      "full-name": "Your Name",
      "email": "you@example.com"
    }
  }
}
```
