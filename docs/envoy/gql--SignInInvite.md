# SignInInvite

GraphQL mutation to check in to an existing invite.

## Request

```graphql
mutation SignInInvite($inviteID: ID!, $reservationID: ID) {
  signInInvite(inviteId: $inviteID, reservationId: $reservationID) {
    id
    signedInAt
  }
}
```

## Variables

```json
{
  "inviteID": "<your-invite-id>"
}
```

## Success Response

```json
{
  "data": {
    "signInInvite": [
      {
        "id": "123456789",
        "signedInAt": "2026-05-01T14:05:00.000Z"
      }
    ]
  }
}
```

## Error: Already signed in

```json
{
  "data": null,
  "errors": [{ "message": "Resource already exists" }]
}
```

## Error: Future invite

```json
{
  "data": null,
  "errors": [{ "message": "Cannot check in for a future invite" }]
}
```
