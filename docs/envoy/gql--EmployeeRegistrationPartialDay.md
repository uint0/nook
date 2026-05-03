# EmployeeRegistrationPartialDay

GraphQL query to fetch registration dates for a location within a date range.

## Request

```graphql
query EmployeeRegistrationPartialDay($locationId: ID!, $startDate: DateTime!, $endDate: DateTime!) {
  employeeRegistrationPartialDay(locationId: $locationId, startDate: $startDate, endDate: $endDate) {
    registrationDates {
      date
      peopleRegistered
      reservations {
        desk {
          name
          floor { name }
        }
      }
      screeningCard {
        __typename
        ... on Invite {
          id
          location { name }
        }
        ... on SelfCertify {
          __typename
        }
      }
    }
  }
}
```

## Variables

```json
{
  "locationId": "<your-location-id>",
  "startDate": "2026-05-01T00:00:00.000Z",
  "endDate": "2026-05-14T23:59:00.000Z"
}
```

## Example Response

```json
{
  "data": {
    "employeeRegistrationPartialDay": {
      "registrationDates": [
        {
          "date": "2026-05-01T00:00:00.000Z",
          "peopleRegistered": 42,
          "reservations": [
            {
              "desk": {
                "name": "1.001",
                "floor": { "name": "Level 1" }
              }
            }
          ],
          "screeningCard": {
            "__typename": "Invite",
            "id": "123456789",
            "location": { "name": "Office Name" }
          }
        },
        {
          "date": "2026-05-02T00:00:00.000Z",
          "peopleRegistered": 38,
          "reservations": [],
          "screeningCard": null
        }
      ]
    }
  }
}
```
