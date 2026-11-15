# Notifications

A notification channel sends a message to people or to a system when something happens on the server.
Channels are set up under **Settings → System → Notifications**. The web interface names the kinds
Telegram, Slack, Mattermost, Rocket.Chat, email and your own webhook ([web interface](../use/web-interface.md));
the server stores four kinds: `telegram`, `slack`, `webhook` and `email`.

A channel has:

- a **name** and an **enabled** switch;
- **events**: the events it hears about; at least one is required;
- **repositories**: the repositories it hears about. Left empty, a channel hears about the hosted
  repositories only, not about proxies or groups;
- a **target**: the chat for Telegram, the recipients for email, and the URL for Slack and webhooks, which is
  stored as a secret;
- for webhooks only, an optional **signing secret** and an optional **Authorization** header, both
  stored sealed and never returned by the REST API;
- a **Connection**, which decides whether the channel uses the server's [outbound HTTP](outbound-http.md)
  settings or a proxy of its own. Webhook and Slack addresses are subject to the same allowed addresses and
  domains as the other outgoing connections.

Changes are collected per channel and sent in batches: a batch goes out after 10 seconds without a new
change, and at the latest 60 seconds after its first change. When a batch holds more than five changes, a
channel other than a webhook gets one summary message instead of one message per change. A failed delivery
is retried a few times with a growing pause.

## Events

A channel chooses events by the names in the first column. A webhook receives the name in the second column
in its `X-Astrum-Event` header and in the `event` field of the body. The third column says which feature adds
the event; the events of a feature exist only on servers that have that feature.

| Channel event | Webhook `event` | Added by | When it is sent |
|---|---|---|---|
| `published` | `component.published` | the server | A component is stored. |
| `updated` | `component.updated` | the server | A component changes, or one of its files is stored again. |
| `deleted` | `component.deleted` | the server | A component is deleted. |
| (summary) | `component.summary` | the server | More than five changes in one batch, for channels other than webhooks. |
| `promoted` | `component.promoted` | promotion | A component arrives by [promotion](../repositories/overview.md) from another hosted repository. It replaces `published` and `updated` for that component, so a channel that should hear about promoted components has to choose `promoted` as well. A channel that had `published`, `updated` and `deleted` gets `promoted` added by the migration. |
| `cooldown.released` | `cooldown.released` | release cooldown | A version that a proxy held back for the release cooldown, and that somebody asked for, becomes available. It is sent by the task `security.purge-expired` ([tasks](../operate/tasks.md)), so that task must be scheduled for it to be sent at all. Other changes are never batched with it. |
| `vulnerability.found` | `vulnerability.found` | vulnerability scanning | A scan finds an advisory that the content policy of the repository flags, and no policy exception covers it. |
| `policy.blocked` | `policy.blocked` | content policies | A download is refused by a content policy. At most one such message is sent per component and repository in 24 hours. |
| `advisories.stale` | `advisories.stale` | vulnerability scanning | An advisory feed has not been updated for too long. It repeats every 24 hours while the feed stays stale, and goes to the channels that choose `vulnerability.found`. |
| (blob store alerts) | `blobstore.alert`, `blobstore.recovered` | the server | Free space, quota or reachability problems of a blob store, and their end. See [blob store space alerts](../operate/monitoring.md#blob-store-space-alerts). |

The component events carry the repository, the component, the initiator and, when the server knows them, links
to the component and a `docker pull` command. The blob store alerts carry the message in `text`, and
`repository` and `component` are `null`.

## Webhook requests

A webhook receives a `POST` with a JSON body and these headers:

| Header | Value |
|---|---|
| `X-Astrum-Event` | The event name, as in the table. |
| `X-Astrum-Delivery` | An identifier of the message. The same message sent again after a failure keeps it. |
| `X-Astrum-Signature` | `sha256=` and the hexadecimal HMAC-SHA256 of the body with the signing secret. Sent only when a signing secret is set. |
| `Authorization` | The value of the channel's Authorization header, when one is set. |

The body has these fields: `event`, `deliveryId`, `timestamp` (UTC, milliseconds), `repository` (`name`,
`format`, `type`), `component` (`id`, `group`, `name`, `version`, `format`, `coordinates`), `initiator`,
`url`, `dockerPull`, `text`. A field without a value is `null`. The `component` block and the `repository`
block are `null` for the blob store alerts.

The two security events add fields to the body:

- `policy` (`vulnerability.found` and `policy.blocked`): the name of the content policy;
- `vulnerability` (`vulnerability.found` only): `advisory`, the advisory id, and `severity`.

To check a request, compute the HMAC-SHA256 of the raw body with the signing secret and compare it with the
value after `sha256=` in constant time. The body must be checked as received, before it is parsed.
