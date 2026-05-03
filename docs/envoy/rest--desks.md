# GET /a/rms/desks?filter[location-id]=<location-id>

## Request

```sh
curl https://app.envoy.com/a/rms/desks?filter[location-id]=<location-id> \
  -H 'Authorization: Bearer <access-token>'
```

## Response

```json
{
  "jsonapi": {
    "version": "1.0"
  },
  "data": [
    {
      "relationships": {
        "company": {
          "data": {
            "type": "companies",
            "id": "<company-id>"
          }
        },
        "location": {
          "data": {
            "type": "locations",
            "id": "<location-id>"
          }
        },
        "neighborhood": {
          "data": {
            "type": "neighborhoods",
            "id": "<neightborhood-id>"
          }
        },
        "floor": {
          "data": {
            "type": "floors",
            "id": "<floor-id>"
          }
        }
      },
      "attributes": {
        "assignment-source-type": null,
        "updated-at": 0,
        "is-assignable": false,
        "assigned-to": null,
        "neighborhood-id": 0,
        "availability": "AVAILABLE",
        "enabled": false,
        "x-pos": 0,
        "parent-desk-id": null,
        "name": "00.000",
        "created-at": 0,
        "neighborhood": "Neighborhood Name",
        "0li0nt-i0": "00000000-0000-0000-0000-000000000000",
        "y-pos": 0
      },
      "id": "<desk-id>",
      "type": "desks"
    }
  ]
}
```