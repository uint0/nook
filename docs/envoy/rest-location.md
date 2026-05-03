# GET /a/visitors/api/v2/locations/<location>

Get details about a location

## Request

```sh
curl https://dashboard.envoy.com/a/visitors/api/v2/locations/<location-id> \
  -H 'Authorization: Bearer <access-token>'
```

## Response

```json
{
  "data": {
    "id": "<location-id>",
    "type": "locations",
    "attributes": {
      "name": "<name>",
      "google-place-id": "[REDACTED]",
      "address": "[REDACTED]",
      "address-line-one": "[REDACTED]",
      "address-line-two": null,
      "auto-sign-out-at-midnight": false,
      "auto-sign-out-at-minutes-since-midnight": 0,
      "city": "[REDACTED]",
      "employee-screening-enabled": false,
      "state": "[REDACTED]",
      "zip": "[REDACTED]",
      "country": "[REDACTED]",
      "longitude": 0.0,
      "latitude": 0.0,
      "device-notifications-enabled": false,
      "pre-registration-required-enabled": false,
      "host-approval-enabled": false,
      "timezone": "[REDACTED]",
      "disabled": false,
      "disabled-to-employees-at": null,
      "locale": "[REDACTED]",
      "mvt-enabled": false,
      "created-at": "[REDACTED]",
      "updated-at": "[REDACTED]",
      "last-entry-at": "[REDACTED]",
      "capacity-limit": 0,
      "touchless-signin-enabled": false,
      "registration-eligibility-start-offset": 0,
      "registration-eligibility-end-offset": 0,
      "near-visit-screening-enabled": false,
      "visitor-registration-eligibility-start-offset": 0,
      "visitor-reminder-threshold-minutes": 0,
      "employee-schedule-enabled": false,
      "schedule-eligibility-offset": 0,
      "multi-tenancy-visitor-notifications-enabled": false,
      "logo-url": "[REDACTED]",
      "logo-small-url": "[REDACTED]",
      "logo-thumb-url": "[REDACTED]",
      "button-color": "[REDACTED]",
      "button-text-color": "[REDACTED]",
      "employees-sync-message": "[REDACTED]",
      "employees-csv-upload-status": "[REDACTED]",
      "blacklist-filters-csv-upload-status": null,
      "logo": {
        "deprecated": true,
        "url": "[REDACTED]",
        "smallUrl": "[REDACTED]",
        "thumbUrl": "[REDACTED]"
      },
      "saml-id": 0,
      "saml-location-id": "[REDACTED]",
      "office365-webhook-url": null,
      "admin-id": 0,
      "watchlist": {
        "deprecated": true,
        "list": "[REDACTED]",
        "notifyEmail": "[REDACTED]",
        "notifyPhoneNumber": "[REDACTED]"
      },
      "blacklist-enabled": false,
      "installed-version": "[REDACTED]",
      "current-version": "[REDACTED]",
      "current-os-version": "[REDACTED]",
      "pubnub-cipher-key": "[REDACTED]",
      "pubnub-channel-entry": "[REDACTED]",
      "pubnub-channel-invite": "[REDACTED]",
      "pubnub-channel-integrations": "[REDACTED]",
      "pubnub-channel-devices": "[REDACTED]",
      "primary": false,
      "color": "[REDACTED]",
      "pre-registration-enabled": false,
      "pre-registration-notes": null,
      "cc-receptionist-enabled": false,
      "host-notifications-enabled": false,
      "printer-notifications-enabled": false,
      "email-notification-enabled": false,
      "slack-notification-enabled": false,
      "call-notification-enabled": false,
      "sms-notification-enabled": false,
      "office365-notification-enabled": false,
      "fallback-notifications-enabled": false,
      "notify-receptionists-on-host-reply-enabled": false,
      "visitor-guide-enabled": false,
      "visitor-survey-enabled": false,
      "visual-compliance-enabled": false,
      "dashboard-fields": [
        {
          "deprecated": true,
          "name": "[REDACTED]",
          "componentName": "[REDACTED]"
        },
        {
          "deprecated": true,
          "name": "[REDACTED]",
          "componentName": "[REDACTED]"
        },
        {
          "deprecated": true,
          "name": "[REDACTED]",
          "componentName": "[REDACTED]"
        }
      ],
      "invite-dashboard-fields": [
        {
          "deprecated": true,
          "name": "[REDACTED]",
          "componentName": "[REDACTED]"
        },
        {
          "deprecated": true,
          "name": "[REDACTED]",
          "componentName": "[REDACTED]"
        }
      ],
      "multiple-languages-enabled": false,
      "enabled-locales": [
        "[REDACTED]"
      ],
      "welcome-email-preference": "[REDACTED]",
      "deliveries-onboarding-complete": false,
      "visitors-onboarding-complete": false,
      "default-notifications": [
        {
          "for": "[REDACTED]",
          "message": "[REDACTED]",
          "host-message": "[REDACTED]",
          "delegated-message": "[REDACTED]"
        },
        {
          "for": "[REDACTED]",
          "message": "[REDACTED]",
          "host-message": "[REDACTED]",
          "delegated-message": "[REDACTED]"
        }
      ],
      "custom-notifications": [
        {
          "for": "[REDACTED]",
          "variables": [
            "[REDACTED]"
          ]
        },
        {
          "for": "[REDACTED]",
          "variables": [
            "[REDACTED]"
          ]
        }
      ]
    },
    "relationships": {
      "company": {
        "data": {
          "type": "companies",
          "id": "[REDACTED]"
        }
      },
      "gdpr-configuration": {
        "data": {
          "type": "gdpr-configurations",
          "id": "[REDACTED]"
        }
      },
      "visual-compliance-configuration": {
        "data": {
          "type": "visual-compliance-configurations",
          "id": "[REDACTED]"
        }
      },
      "admin": {
        "data": {
          "type": "admins",
          "id": "[REDACTED]"
        }
      },
      "config": {
        "data": {
          "type": "configs",
          "id": "[REDACTED]"
        }
      },
      "printer": {
        "data": {
          "type": "printers",
          "id": "[REDACTED]"
        }
      },
      "printers": {
        "data": [
          {
            "type": "printers",
            "id": "[REDACTED]"
          },
          {
            "type": "printers",
            "id": "[REDACTED]"
          }
        ]
      },
      "badge": {
        "data": {
          "type": "badges",
          "id": "[REDACTED]"
        }
      },
      "tv-config": {
        "data": {
          "type": "tv-configs",
          "id": "[REDACTED]"
        }
      },
      "blacklist-contacts": {
        "data": [
          {
            "type": "users",
            "id": "[REDACTED]"
          },
          {
            "type": "users",
            "id": "[REDACTED]"
          },
          {
            "type": "users",
            "id": "[REDACTED]"
          }
        ]
      },
      "id-scan-contacts": {
        "data": []
      },
      "fallback-contacts": {
        "links": {
          "href": "[REDACTED]"
        }
      },
      "capacity-contacts": {
        "data": []
      },
      "device-contacts": {
        "data": []
      },
      "groups": {
        "data": [
          {
            "type": "groups",
            "id": "[REDACTED]"
          }
        ]
      },
      "multi-tenancy-visitor-contacts": {
        "data": []
      },
      "connect-walk-in-approval-contacts": {
        "data": []
      },
      "printer-contacts": {
        "data": [
          {
            "type": "users",
            "id": "[REDACTED]"
          },
          {
            "type": "users",
            "id": "[REDACTED]"
          },
          {
            "type": "users",
            "id": "[REDACTED]"
          }
        ]
      },
      "flows": {
        "data": [
          {
            "type": "flows",
            "id": "[REDACTED]"
          },
          {
            "type": "flows",
            "id": "[REDACTED]"
          },
          {
            "type": "flows",
            "id": "[REDACTED]"
          },
          {
            "type": "flows",
            "id": "[REDACTED]"
          },
          {
            "type": "flows",
            "id": "[REDACTED]"
          },
          {
            "type": "flows",
            "id": "[REDACTED]"
          }
        ]
      },
      "locations-setup-guide-steps": {
        "links": {
          "related": "[REDACTED]"
        }
      },
      "location-subscriptions": {
        "links": {
          "related": "[REDACTED]"
        }
      },
      "visitor-guide": {
        "data": {
          "type": "visitor-guides",
          "id": "[REDACTED]"
        }
      },
      "visitor-survey-configuration": {
        "data": {
          "type": "visitor-survey-configurations",
          "id": "[REDACTED]"
        }
      },
      "ticket-configuration": {
        "data": {
          "type": "ticket-configurations",
          "id": "[REDACTED]"
        }
      }
    }
  },
  "links": {
    "configs": "[REDACTED]"
  }
}