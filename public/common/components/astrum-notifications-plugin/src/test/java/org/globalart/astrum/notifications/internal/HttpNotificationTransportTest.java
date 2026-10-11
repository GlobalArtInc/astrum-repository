package org.globalart.astrum.notifications.internal;

import java.io.IOException;
import java.io.OutputStream;
import java.net.InetSocketAddress;
import java.util.List;
import java.util.Map;
import java.util.concurrent.CopyOnWriteArrayList;

import jakarta.validation.ValidationException;

import org.globalart.astrum.httpclient.HttpClientManager;
import org.globalart.astrum.httpclient.HttpClientPlan;
import org.globalart.astrum.validation.ssrf.AntiSsrfService;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.sun.net.httpserver.HttpExchange;
import com.sun.net.httpserver.HttpServer;
import org.apache.http.impl.client.HttpClients;
import org.junit.After;
import org.junit.Before;
import org.junit.Test;

import static java.nio.charset.StandardCharsets.UTF_8;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsString;
import static org.hamcrest.Matchers.empty;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.not;
import static org.junit.Assert.assertThrows;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.doThrow;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

public class HttpNotificationTransportTest
{
  private static final String TOKEN = "123456:ABC-def_ghi";

  private final ObjectMapper mapper = new ObjectMapper();

  private final List<Map.Entry<String, String>> requests = new CopyOnWriteArrayList<>();

  private HttpServer server;

  private volatile int status = 200;

  private volatile String response = "{\"ok\":true}";

  private volatile String retryAfter;

  private AntiSsrfService antiSsrfService;

  private volatile Map<String, Object> planAttributes;

  private HttpNotificationTransport transport;

  @Before
  public void setUp() throws IOException {
    server = HttpServer.create(new InetSocketAddress("127.0.0.1", 0), 0);
    server.createContext("/", this::handle);
    server.start();
    HttpClientManager httpClientManager = mock(HttpClientManager.class);
    when(httpClientManager.create(any())).thenAnswer(invocation -> {
      HttpClientPlan plan = new HttpClientPlan();
      ((HttpClientPlan.Customizer) invocation.getArgument(0)).customize(plan);
      planAttributes = plan.getAttributes();
      return HttpClients.createDefault();
    });
    antiSsrfService = mock(AntiSsrfService.class);
    transport = new HttpNotificationTransport(httpClientManager, antiSsrfService, List.of(), baseUrl() + "/telegram/");
  }

  @After
  public void tearDown() {
    server.stop(0);
  }

  private String baseUrl() {
    return "http://127.0.0.1:" + server.getAddress().getPort();
  }

  private void handle(final HttpExchange exchange) throws IOException {
    String body = new String(exchange.getRequestBody().readAllBytes(), UTF_8);
    requests.add(Map.entry(exchange.getRequestURI().getPath(), body));
    byte[] bytes = response.getBytes(UTF_8);
    if (retryAfter != null) {
      exchange.getResponseHeaders().add("Retry-After", retryAfter);
    }
    exchange.getResponseHeaders().add("Content-Type", "application/json");
    exchange.sendResponseHeaders(status, bytes.length);
    try (OutputStream out = exchange.getResponseBody()) {
      out.write(bytes);
    }
  }

  private static NotificationSecrets secrets() {
    return NotificationSecrets.of(TOKEN);
  }

  private static NotificationChannel telegram() {
    NotificationChannel channel = new NotificationChannel();
    channel.setType(NotificationChannel.TELEGRAM);
    channel.setName("releases");
    channel.setChatId("-1001234567890");
    channel.setMessageThreadId(42L);
    return channel;
  }

  private static NotificationChannel slack() {
    NotificationChannel channel = new NotificationChannel();
    channel.setType(NotificationChannel.SLACK);
    channel.setName("releases");
    return channel;
  }

  @Test
  public void postsTelegramMessagesToTheBotApi() throws Exception {
    transport.send(telegram(), NotificationSecrets.of(TOKEN), "<b>hi</b>");

    assertThat(requests, hasSize(1));
    assertThat(requests.get(0).getKey(), is("/telegram/bot" + TOKEN + "/sendMessage"));
    JsonNode body = mapper.readTree(requests.get(0).getValue());
    assertThat(body.get("chat_id").asText(), is("-1001234567890"));
    assertThat(body.get("text").asText(), is("<b>hi</b>"));
    assertThat(body.get("parse_mode").asText(), is("HTML"));
    assertThat(body.get("disable_web_page_preview").asBoolean(), is(true));
    assertThat(body.get("message_thread_id").asLong(), is(42L));
  }

  @Test
  public void keepsRequestPathsOutOfTheOutboundLog() throws Exception {
    transport.send(telegram(), NotificationSecrets.of(TOKEN), "hi");

    assertThat(planAttributes.get(HttpClientManager.REDACT_REQUEST_PATH), is(true));
  }

  @Test
  public void leavesTheTopicOutWhenNotSet() throws Exception {
    NotificationChannel channel = telegram();
    channel.setMessageThreadId(null);

    transport.send(channel, NotificationSecrets.of(TOKEN), "hi");

    assertThat(mapper.readTree(requests.get(0).getValue()).has("message_thread_id"), is(false));
  }

  @Test
  public void telegramRateLimitIsRetryableAfterTheRequestedDelay() {
    status = 429;
    response = "{\"ok\":false,\"error_code\":429,\"description\":\"Too Many Requests: retry after 7\","
        + "\"parameters\":{\"retry_after\":7}}";

    NotificationDeliveryException e =
        assertThrows(NotificationDeliveryException.class, () -> transport.send(telegram(), secrets(), "hi"));

    assertThat(e.isRetryable(), is(true));
    assertThat(e.getRetryAfterMillis(), is(7000L));
    assertThat(e.getStatus(), is(429));
    assertThat(e.getMessage(), is("Telegram answered 429: Too Many Requests: retry after 7"));
  }

  @Test
  public void telegramClientErrorsAreFinal() {
    status = 400;
    response = "{\"ok\":false,\"error_code\":400,\"description\":\"Bad Request: chat not found\"}";

    NotificationDeliveryException e =
        assertThrows(NotificationDeliveryException.class, () -> transport.send(telegram(), secrets(), "hi"));

    assertThat(e.isRetryable(), is(false));
    assertThat(e.getMessage(), is("Telegram answered 400: Bad Request: chat not found"));
  }

  @Test
  public void malformedBotTokenIsNeverPutIntoTheUrl() {
    NotificationDeliveryException e = assertThrows(NotificationDeliveryException.class,
        () -> transport.send(telegram(), NotificationSecrets.of("123:abc/../../admin?x="), "hi"));

    assertThat(e.isRetryable(), is(false));
    assertThat(requests, is(empty()));
  }

  @Test
  public void unreachableTelegramIsRetryableAndDoesNotRevealTheToken() throws Exception {
    server.stop(0);

    NotificationDeliveryException e =
        assertThrows(NotificationDeliveryException.class, () -> transport.send(telegram(), secrets(), "hi"));

    assertThat(e.isRetryable(), is(true));
    assertThat(e.getMessage(), not(containsString(TOKEN)));
  }

  @Test
  public void postsSlackMessagesToTheWebhook() throws Exception {
    response = "ok";

    transport.send(slack(), NotificationSecrets.of(baseUrl() + "/services/T000/B000/XXXX"), "*hi*");

    assertThat(requests.get(0).getKey(), is("/services/T000/B000/XXXX"));
    assertThat(mapper.readTree(requests.get(0).getValue()).get("text").asText(), is("*hi*"));
    verify(antiSsrfService).validateHost("127.0.0.1");
  }

  @Test
  public void slackServerErrorsAreRetryableAndHonourRetryAfter() {
    status = 503;
    response = "busy";
    retryAfter = "3";

    NotificationDeliveryException e = assertThrows(NotificationDeliveryException.class,
        () -> transport.send(slack(), NotificationSecrets.of(baseUrl() + "/hooks/secret-path"), "hi"));

    assertThat(e.isRetryable(), is(true));
    assertThat(e.getRetryAfterMillis(), is(3000L));
    assertThat(e.getMessage(), is("The webhook answered 503: busy"));
  }

  @Test
  public void slackClientErrorsAreFinal() {
    status = 404;
    response = "no_team";

    NotificationDeliveryException e = assertThrows(NotificationDeliveryException.class,
        () -> transport.send(slack(), NotificationSecrets.of(baseUrl() + "/hooks/x"), "hi"));

    assertThat(e.isRetryable(), is(false));
    assertThat(e.getMessage(), is("The webhook answered 404: no_team"));
  }

  @Test
  public void webhookHostsBlockedBySsrfProtectionAreNotCalled() {
    doThrow(new ValidationException("Host resolves to private/local IP address(es)"))
        .when(antiSsrfService).validateHost(anyString());

    NotificationDeliveryException e = assertThrows(NotificationDeliveryException.class,
        () -> transport.send(slack(), NotificationSecrets.of(baseUrl() + "/hooks/x"), "hi"));

    assertThat(e.isRetryable(), is(false));
    assertThat(e.getMessage(), containsString("SSRF"));
    assertThat(requests, is(empty()));
  }

  @Test
  public void onlyHttpWebhookUrlsAreAccepted() {
    assertThat(HttpNotificationTransport.webhookUri("https://hooks.slack.com/services/T/B/X").getHost(),
        is("hooks.slack.com"));
    assertThat(HttpNotificationTransport.webhookUri("file:///etc/passwd"), is((Object) null));
    assertThat(HttpNotificationTransport.webhookUri("https://user:pass@example.com/x"), is((Object) null));
    assertThat(HttpNotificationTransport.webhookUri("not a url"), is((Object) null));
    assertThat(HttpNotificationTransport.webhookUri(""), is((Object) null));
  }
}
