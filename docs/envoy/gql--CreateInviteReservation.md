# CreateInviteReservation

GraphQL mutation to create a new booking (invite + desk reservation).

## Request

```graphql
mutation CreateInviteReservation($deskId: ID, $invite: NewInviteInput, $inviteId: ID) {
  createInviteReservation(deskId: $deskId, invite: $invite, inviteId: $inviteId) {
    invite {
      id
      expectedArrivalTime
      location { name }
    }
    reservation {
      desk {
        id
        name
      }
    }
  }
}
```

## Variables

```json
{
  "deskId": "<optional-desk-id>",
  "invite": {
    "fullName": "Your Name",
    "email": "you@example.com",
    "location": "<your-location-id>",
    "userData": [
      { "field": "Purpose of visit", "value": "Employee registration" }
    ],
    "expectedArrivalTime": "2026-05-01T14:00:00.000Z"
  }
}
```

## Success Response

```json
{
  "data": {
    "createInviteReservation": {
      "invite": {
        "id": "123456789",
        "expectedArrivalTime": "2026-05-01T14:00:00.000Z",
        "location": { "name": "Office Name" }
      },
      "reservation": {
        "desk": {
          "id": "987654321",
          "name": "1.001"
        }
      }
    }
  }
}
```

## Error Response (date outside bookable window)

```json
{
  "data": null,
  "errors": [
    {
      "message": "Scheduling Limit Check: date is outside the bookable window"
    }
  ]
}
```
