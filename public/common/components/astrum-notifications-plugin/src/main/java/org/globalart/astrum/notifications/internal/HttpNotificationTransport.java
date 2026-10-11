package org.globalart.astrum.notifications.internal;

import java.io.IOException;
import java.net.URI;
import java.net.URISyntaxException;
import java.util.List;
import java.util.regex.Pattern;

import javax.annotation.Nullable;

import jakarta.validation.ValidationException;

import org.globalart.astrum.httpclient.HttpClientManager;
import org.globalart.astrum.httpclient.SSLContextSelector;
import org.globalart.astrum.internal.httpclient.AstrumSSLConnectionSocketFactory;
import org.globalart.astrum.validation.ssrf.AntiSsrfService;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ObjectNode;
import org.apache.http.Header;
import org.apache.http.HttpEntity;
import org.apache.http.client.methods.CloseableHttpResponse;
import org.apache.http.client.methods.HttpPost;
import org.apache.http.client.protocol.HttpClientContext;
import org.apache.http.conn.socket.LayeredConnectionSocketFactory;
import org.apache.http.entity.ContentType;
import org.apache.http.entity.StringEntity;
import org.apache.http.impl.client.CloseableHttpClient;
import org.apache.http.util.EntityUtils;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.nio.charset.StandardCharsets.UTF_8;

/**
 * Sends messages through the Telegram Bot API and through Slack-compatible incoming webhooks, using the global HTTP
 * settings (proxy, timeouts, user agent) or the connection chosen for the channel (see
 * {@link NotificationConnection}). Webhook hosts are checked against the SSRF protection settings; the Telegram API
 * address is set by the operator with {@code astrum.notifications.telegram.apiUrl}.
 */
@Component
public class HttpNotificationTransport
    implements NotificationTransport
{
  static final String DEFAULT_TELEGRAM_API_URL = "https://api.telegram.org";

  static final Pattern BOT_TOKEN = Pattern.compile("\\d+:[A-Za-z0-9_-]+");

  private static final int MAX_ERROR_LENGTH = 300;

  private static final long MAX_RETRY_AFTER_SECONDS = 3600;

  private final HttpClientManager httpClientManager;

  private final AntiSsrfService antiSsrfService;

  private final String telegramApiUrl;

  private final LayeredConnectionSocketFactory tls;

  private final ObjectMapper mapper = new ObjectMapper();

  @Autowired
  public HttpNotificationTransport(
      final HttpClientManager httpClientManager,
      final AntiSsrfService antiSsrfService,
      final List<SSLContextSelector> sslContextSelectors,
      @Value("${astrum.notifications.telegram.apiUrl:" + DEFAULT_TELEGRAM_API_URL + "}") final String telegramApiUrl)
  {
    this.httpClientManager = checkNotNull(httpClientManager);
    this.antiSsrfService = checkNotNull(antiSsrfService);
    this.tls = new AstrumSSLConnectionSocketFactory(checkNotNull(sslContextSelectors));
    this.telegramApiUrl = checkNotNull(telegramApiUrl).replaceAll("/+$", "");
  }

  @Override
  public void send(final NotificationChannel channel, final NotificationSecrets secrets, final String text)
      throws NotificationDeliveryException
  {
    checkNotNull(secrets);
    checkNotNull(text);
    NotificationConnection connection = new NotificationConnection(channel, secrets.proxyPassword(), tls);
    if (channel.isTelegram()) {
      sendTelegram(channel, connection, secrets.secret(), text);
    }
    else {
      sendWebhook(connection, secrets.secret(), text);
    }
  }

  private void sendTelegram(
      final NotificationChannel channel,
      final NotificationConnection connection,
      final String token,
      final String text)
      throws NotificationDeliveryException
  {
    if (!BOT_TOKEN.matcher(token).matches()) {
      throw NotificationDeliveryException.permanent("The bot token is malformed");
    }
    ObjectNode body = mapper.createObjectNode();
    body.put("chat_id", channel.getChatId());
    body.put("text", text);
    body.put("parse_mode", "HTML");
    body.put("disable_web_page_preview", true);
    if (channel.getMessageThreadId() != null) {
      body.put("message_thread_id", channel.getMessageThreadId());
    }
    HttpPost post = new HttpPost(telegramApiUrl + "/bot" + token + "/sendMessage");
    post.setEntity(new StringEntity(body.toString(), ContentType.APPLICATION_JSON));
    HttpClientContext context = connection.context();

    try (CloseableHttpClient client = httpClientManager.create(connection);
        CloseableHttpResponse response = client.execute(post, context)) {
      refuseProxyAnswer(connection, response, context, post, token);
      int status = response.getStatusLine().getStatusCode();
      String content = content(response.getEntity());
      JsonNode json = parse(content);
      boolean ok = json == null || json.path("ok").asBoolean(true);
      if (status >= 200 && status < 300 && ok) {
        return;
      }
      String description = json != null ? json.path("description").asText(null) : null;
      long retryAfter = json != null ? json.path("parameters").path("retry_after").asLong(0) : 0;
      if (retryAfter <= 0) {
        retryAfter = retryAfterHeader(response);
      }
      throw new NotificationDeliveryException(
          sanitize("Telegram answered " + status + (description != null ? ": " + description : ""), token),
          status,
          isRetryable(status),
          Math.min(retryAfter, MAX_RETRY_AFTER_SECONDS) * 1000);
    }
    catch (IOException e) {
      throw sanitize(connection.unreachable("Telegram", e), token);
    }
  }

  private void sendWebhook(final NotificationConnection connection, final String url, final String text)
      throws NotificationDeliveryException
  {
    URI uri = webhookUri(url);
    if (uri == null) {
      throw NotificationDeliveryException.permanent("The webhook URL is not a valid http or https URL");
    }
    try {
      antiSsrfService.validateHost(uri.getHost());
    }
    catch (ValidationException e) {
      throw NotificationDeliveryException.permanent(
          "The webhook host is blocked by the SSRF protection: " + sanitize(e.getMessage(), url));
    }
    ObjectNode body = mapper.createObjectNode();
    body.put("text", text);
    HttpPost post = new HttpPost(uri);
    post.setEntity(new StringEntity(body.toString(), ContentType.APPLICATION_JSON));
    HttpClientContext context = connection.context();

    try (CloseableHttpClient client = httpClientManager.create(connection);
        CloseableHttpResponse response = client.execute(post, context)) {
      refuseProxyAnswer(connection, response, context, post, url);
      int status = response.getStatusLine().getStatusCode();
      String content = content(response.getEntity());
      if (status >= 200 && status < 300) {
        return;
      }
      String reason = content != null && !content.isBlank() && !content.contains("<html")
          ? content.strip()
          : response.getStatusLine().getReasonPhrase();
      throw new NotificationDeliveryException(
          sanitize("The webhook answered " + status + (reason != null && !reason.isBlank() ? ": " + reason : ""), url),
          status,
          isRetryable(status),
          Math.min(retryAfterHeader(response), MAX_RETRY_AFTER_SECONDS) * 1000);
    }
    catch (IOException e) {
      throw sanitize(connection.unreachable("the webhook", e), url);
    }
  }

  private static void refuseProxyAnswer(
      final NotificationConnection connection,
      final CloseableHttpResponse response,
      final HttpClientContext context,
      final HttpPost post,
      final String secret) throws NotificationDeliveryException
  {
    String target = post.getURI().getHost() + (post.getURI().getPort() > 0 ? ":" + post.getURI().getPort() : "");
    NotificationDeliveryException refusal = connection.refusal(response, context, target);
    if (refusal != null) {
      EntityUtils.consumeQuietly(response.getEntity());
      throw sanitize(refusal, secret);
    }
  }

  /**
   * @return the URI of an http or https webhook URL with a host, or {@code null} for anything else
   */
  @Nullable
  static URI webhookUri(@Nullable final String url) {
    if (url == null || url.isBlank()) {
      return null;
    }
    try {
      URI uri = new URI(url.strip());
      String scheme = uri.getScheme();
      if (("https".equalsIgnoreCase(scheme) || "http".equalsIgnoreCase(scheme)) && uri.getHost() != null
          && uri.getRawUserInfo() == null) {
        return uri;
      }
      return null;
    }
    catch (URISyntaxException e) {
      return null;
    }
  }

  private static boolean isRetryable(final int status) {
    return status == 429 || status >= 500;
  }

  private static long retryAfterHeader(final CloseableHttpResponse response) {
    Header header = response.getFirstHeader("Retry-After");
    if (header == null) {
      return 0;
    }
    try {
      return Math.max(0, Long.parseLong(header.getValue().trim()));
    }
    catch (NumberFormatException e) {
      return 0;
    }
  }

  @Nullable
  private static String content(@Nullable final HttpEntity entity) throws IOException {
    if (entity == null) {
      return null;
    }
    try {
      return EntityUtils.toString(entity, UTF_8);
    }
    finally {
      EntityUtils.consumeQuietly(entity);
    }
  }

  @Nullable
  private JsonNode parse(@Nullable final String content) {
    if (content == null || content.isBlank()) {
      return null;
    }
    try {
      JsonNode node = mapper.readTree(content);
      return node != null && node.isObject() ? node : null;
    }
    catch (IOException e) {
      return null;
    }
  }

  private static NotificationDeliveryException sanitize(final NotificationDeliveryException e, final String secret) {
    return new NotificationDeliveryException(sanitize(e.getMessage(), secret), e.getStatus(), e.isRetryable(),
        e.getRetryAfterMillis());
  }

  private static String sanitize(@Nullable final String message, final String secret) {
    String text = message == null ? "" : message;
    if (!secret.isEmpty()) {
      text = text.replace(secret, "***");
    }
    return text.length() > MAX_ERROR_LENGTH ? text.substring(0, MAX_ERROR_LENGTH) + "…" : text;
  }
}
