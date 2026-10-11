package org.globalart.astrum.notifications.internal;

import javax.annotation.Nullable;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.google.common.hash.Hashing;

import static java.nio.charset.StandardCharsets.UTF_8;

/**
 * The request a JSON webhook gets for each component change, after changes of one component within the delivery
 * window have been combined; JSON webhooks never get the summary that chats get for many changes. The body is stable:
 *
 * <pre>
 * {
 *   "event": "component.published",
 *   "deliveryId": "0b4f7c9e-3d61-4a8e-9c1e-6f2d1a7b5e40",
 *   "timestamp": "2026-10-11T08:15:30.123Z",
 *   "repository": {"name": "maven-releases", "format": "maven2", "type": "hosted"},
 *   "component": {
 *     "id": "bWF2ZW4tcmVsZWFzZXM6MTM0NTg2Mzc",
 *     "group": "org.example",
 *     "name": "demo",
 *     "version": "1.0.0",
 *     "format": "maven2",
 *     "coordinates": "org.example:demo:1.0.0"
 *   },
 *   "initiator": "admin",
 *   "url": "https://repo.example.com/#browse/browse:maven-releases:org%2Fexample%2Fdemo%2F1.0.0",
 *   "dockerPull": null,
 *   "text": "📦 *Component published*\n`org.example:demo:1.0.0`\n…"
 * }
 * </pre>
 *
 * <ul>
 * <li>{@code event}: {@code component.published}, {@code component.updated}, {@code component.deleted}, or
 * {@code test} for a test message, whose {@code repository} and {@code component} are {@code null}.</li>
 * <li>{@code deliveryId}: the {@code X-Astrum-Delivery} header; it stays the same when a delivery is retried.</li>
 * <li>{@code timestamp}: when the message was written, ISO-8601 in UTC.</li>
 * <li>{@code repository.type}: {@code hosted}, {@code proxy} or {@code group}.</li>
 * <li>{@code component.id}: the component's id in the REST API ({@code /service/rest/v1/components/{id}}), or
 * {@code null} when it is not known; {@code group} and {@code version} are {@code null} when the format has none;
 * {@code coordinates} is how the format writes the component: {@code group:name:version}, {@code name:tag} for Docker,
 * {@code @scope/name@version} for npm, the path for raw.</li>
 * <li>{@code initiator}: the user who made the change, or {@code null} when it is not known.</li>
 * <li>{@code url}: the component in the browse tree, or {@code null} when the base URL is not known or the component
 * was deleted.</li>
 * <li>{@code dockerPull}: the {@code docker pull} command of a published or updated Docker image, otherwise
 * {@code null}.</li>
 * <li>{@code text}: the message a Slack-compatible webhook would show, in the channel's language.</li>
 * </ul>
 *
 * Requests are POSTed with {@code Content-Type: application/json; charset=UTF-8}, {@link #EVENT_HEADER} (the event),
 * {@link #DELIVERY_HEADER} (the delivery id), {@link #SIGNATURE_HEADER} ({@code sha256=} and the hex HMAC-SHA256 of
 * the raw body keyed with the channel's signing secret) when the channel has a signing secret, and the channel's
 * {@code Authorization} header when it has one. Any 2xx answer means the message was delivered; 429 and 5xx answers
 * and network failures are retried, after {@code Retry-After} when given.
 */
public final class WebhookPayload
{
  public static final String EVENT_HEADER = "X-Astrum-Event";

  public static final String DELIVERY_HEADER = "X-Astrum-Delivery";

  public static final String SIGNATURE_HEADER = "X-Astrum-Signature";

  private static final ObjectMapper MAPPER = new ObjectMapper();

  private WebhookPayload() {
  }

  /**
   * @return the body of the request for the message, in UTF-8
   */
  public static byte[] body(final NotificationMessage message) {
    ObjectNode body = MAPPER.createObjectNode();
    body.put("event", message.event());
    body.put("deliveryId", message.deliveryId());
    body.put("timestamp", message.timestamp().toString());
    ComponentNotice notice = message.notice();
    if (notice == null) {
      body.putNull("repository");
      body.putNull("component");
      body.putNull("initiator");
      body.putNull("url");
      body.putNull("dockerPull");
    }
    else {
      ObjectNode repository = body.putObject("repository");
      repository.put("name", notice.repository());
      repository.put("format", emptyToNull(notice.format()));
      repository.put("type", emptyToNull(notice.repositoryType()));
      ObjectNode component = body.putObject("component");
      component.put("id", notice.componentId());
      component.put("group", emptyToNull(notice.namespace()));
      component.put("name", notice.name());
      component.put("version", emptyToNull(notice.version()));
      component.put("format", emptyToNull(notice.format()));
      component.put("coordinates", NotificationMessages.coordinates(notice));
      body.put("initiator", emptyToNull(notice.initiator()));
      body.put("url", NotificationMessages.link(notice));
      body.put("dockerPull", NotificationMessages.pullCommand(notice));
    }
    body.put("text", message.text());
    try {
      return MAPPER.writeValueAsString(body).getBytes(UTF_8);
    }
    catch (JsonProcessingException e) {
      throw new IllegalStateException(e);
    }
  }

  /**
   * @return the value of {@link #SIGNATURE_HEADER} for the body
   */
  public static String signature(final String signingSecret, final byte[] body) {
    return "sha256=" + Hashing.hmacSha256(signingSecret.getBytes(UTF_8)).hashBytes(body);
  }

  @Nullable
  private static String emptyToNull(@Nullable final String value) {
    return value == null || value.isEmpty() ? null : value;
  }
}
