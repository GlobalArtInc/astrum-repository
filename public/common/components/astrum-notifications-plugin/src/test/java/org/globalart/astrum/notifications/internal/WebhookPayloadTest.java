package org.globalart.astrum.notifications.internal;

import java.time.Instant;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.junit.Test;

import static java.nio.charset.StandardCharsets.UTF_8;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

public class WebhookPayloadTest
{
  private static final Instant TIME = Instant.parse("2026-10-11T08:15:30.123Z");

  private static final String DELIVERY = "0b4f7c9e-3d61-4a8e-9c1e-6f2d1a7b5e40";

  private final ObjectMapper mapper = new ObjectMapper();

  @Test
  public void signatureIsTheHexHmacSha256OfTheBody() {
    assertThat(WebhookPayload.signature("Jefe", "what do ya want for nothing?".getBytes(UTF_8)),
        is("sha256=5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"));
  }

  @Test
  public void bodyOfAComponentChangeIsStable() {
    ComponentNotice notice = new ComponentNotice("maven-releases", "maven2", "org.example", "demo", "1.0.0",
        NoticeAction.PUBLISHED, "admin", "org/example/demo/1.0.0", null, "https://repo.example.com/", "hosted",
        "bWF2ZW4tcmVsZWFzZXM6MTM0NTg2Mzc");
    NotificationMessage message = new NotificationMessage("component.published", "📦 *Component published*", notice,
        DELIVERY, TIME);

    String body = new String(WebhookPayload.body(message), UTF_8);

    assertThat(body, is("{\"event\":\"component.published\",\"deliveryId\":\"" + DELIVERY + "\","
        + "\"timestamp\":\"2026-10-11T08:15:30.123Z\","
        + "\"repository\":{\"name\":\"maven-releases\",\"format\":\"maven2\",\"type\":\"hosted\"},"
        + "\"component\":{\"id\":\"bWF2ZW4tcmVsZWFzZXM6MTM0NTg2Mzc\",\"group\":\"org.example\",\"name\":\"demo\","
        + "\"version\":\"1.0.0\",\"format\":\"maven2\",\"coordinates\":\"org.example:demo:1.0.0\"},"
        + "\"initiator\":\"admin\","
        + "\"url\":\"https://repo.example.com/#browse/browse:maven-releases:org%2Fexample%2Fdemo%2F1.0.0\","
        + "\"dockerPull\":null,\"text\":\"📦 *Component published*\"}"));
  }

  @Test
  public void dockerImagesComeWithTheirPullCommandAndWithoutAGroup() throws Exception {
    ComponentNotice notice = new ComponentNotice("docker-hosted", "docker", "", "team/app", "1.4",
        NoticeAction.UPDATED, null, null, "repo.example.com:8085", null, "hosted", null);

    JsonNode body = mapper.readTree(WebhookPayload.body(NotificationMessage.of(notice, "text")));

    assertThat(body.get("event").asText(), is("component.updated"));
    assertThat(body.get("component").get("group").isNull(), is(true));
    assertThat(body.get("component").get("id").isNull(), is(true));
    assertThat(body.get("component").get("coordinates").asText(), is("team/app:1.4"));
    assertThat(body.get("dockerPull").asText(), is("docker pull repo.example.com:8085/team/app:1.4"));
    assertThat(body.get("initiator").isNull(), is(true));
    assertThat(body.get("url").isNull(), is(true));
  }

  @Test
  public void testMessagesTellAboutNoComponent() throws Exception {
    JsonNode body = mapper.readTree(WebhookPayload.body(NotificationMessage.test("✅ Test")));

    assertThat(body.get("event").asText(), is("test"));
    assertThat(body.get("repository").isNull(), is(true));
    assertThat(body.get("component").isNull(), is(true));
    assertThat(body.get("text").asText(), is("✅ Test"));
    assertThat(Instant.parse(body.get("timestamp").asText()).isAfter(TIME), is(true));
    assertThat(body.get("deliveryId").asText().length(), is(36));
  }
}
