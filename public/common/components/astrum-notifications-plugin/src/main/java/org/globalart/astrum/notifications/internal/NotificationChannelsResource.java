package org.globalart.astrum.notifications.internal;

import java.net.URI;
import java.util.ArrayList;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Set;
import java.util.Objects;
import java.util.regex.Pattern;

import javax.annotation.Nullable;

import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.DELETE;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.POST;
import jakarta.ws.rs.PUT;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.PathParam;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.core.Response.Status;

import org.globalart.astrum.common.app.BaseUrlManager;
import org.globalart.astrum.notifications.internal.NotificationChannelStore.SecretChange;
import org.globalart.astrum.rapture.PasswordPlaceholder;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.rest.Resource;
import org.globalart.astrum.rest.WebApplicationMessageException;
import org.globalart.astrum.security.UserIdHelper;

import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Strings.isNullOrEmpty;
import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;
import static java.nio.charset.StandardCharsets.UTF_8;

/**
 * Manages the notification channels and sends test messages. The address of a channel's own proxy is chosen by the
 * administrator and may be in the local network; it is not checked against the SSRF protection settings, which apply
 * to the webhook hosts the messages are meant for.
 */
@Component
@Consumes(APPLICATION_JSON)
@Produces(APPLICATION_JSON)
@Path(NotificationChannelsResource.RESOURCE_PATH)
public class NotificationChannelsResource
    implements Resource
{
  static final String RESOURCE_PATH = "internal/ui/notifications";

  static final int MAX_NAME_LENGTH = 100;

  static final int MAX_PROXY_HOST_LENGTH = 255;

  static final int MAX_PROXY_CREDENTIAL_BYTES = 255;

  private static final Pattern CHAT_ID = Pattern.compile("-?\\d{1,20}|@[A-Za-z][A-Za-z0-9_]{3,31}");

  private static final Pattern PROXY_HOST =
      Pattern.compile("[A-Za-z0-9_](?:[A-Za-z0-9_.-]*[A-Za-z0-9_])?|[0-9A-Fa-f.]*:[0-9A-Fa-f:.]*");

  private final NotificationChannelStore store;

  private final NotificationTransport transport;

  private final RepositoryManager repositoryManager;

  private final BaseUrlManager baseUrlManager;

  @Autowired
  public NotificationChannelsResource(
      final NotificationChannelStore store,
      final NotificationTransport transport,
      final RepositoryManager repositoryManager,
      final BaseUrlManager baseUrlManager)
  {
    this.store = checkNotNull(store);
    this.transport = checkNotNull(transport);
    this.repositoryManager = checkNotNull(repositoryManager);
    this.baseUrlManager = checkNotNull(baseUrlManager);
  }

  @GET
  @RequiresAuthentication
  @RequiresPermissions("astrum:settings:read")
  public List<NotificationChannelXO> list() {
    return store.list().stream().map(NotificationChannelsResource::toXO).toList();
  }

  @POST
  @RequiresAuthentication
  @RequiresPermissions("astrum:settings:update")
  public NotificationChannelXO create(final NotificationChannelXO xo) {
    NotificationChannel channel = toChannel(require(xo), null, false);
    Secret secret = secret(xo, channel, null);
    channel.setTargetHost(secret.targetHost());
    SecretChange proxyPassword = proxyPassword(xo, channel, null);
    return toXO(store.create(channel, secret.change().value(), proxyPassword.value(), UserIdHelper.get()));
  }

  @PUT
  @Path("{id}")
  @RequiresAuthentication
  @RequiresPermissions("astrum:settings:update")
  public NotificationChannelXO update(@PathParam("id") final String id, final NotificationChannelXO xo) {
    NotificationChannel existing = find(id);
    NotificationChannel channel = toChannel(require(xo), existing, false);
    channel.setId(existing.getId());
    Secret secret = secret(xo, channel, existing);
    channel.setTargetHost(secret.targetHost());
    SecretChange proxyPassword = proxyPassword(xo, channel, existing);
    return toXO(store.update(channel, secret.change(), proxyPassword, UserIdHelper.get()));
  }

  @DELETE
  @Path("{id}")
  @RequiresAuthentication
  @RequiresPermissions("astrum:settings:update")
  public void delete(@PathParam("id") final String id) {
    if (!store.delete(id)) {
      throw notFound();
    }
  }

  /**
   * Sends a test message to a saved channel.
   */
  @POST
  @Path("{id}/test")
  @RequiresAuthentication
  @RequiresPermissions("astrum:settings:update")
  public NotificationTestResultXO test(@PathParam("id") final String id) {
    NotificationChannel channel = find(id);
    return sendTest(channel, store.secret(channel), store.proxyPassword(channel));
  }

  /**
   * Sends a test message with the settings of the form, which may not be saved yet. A secret or proxy password
   * placeholder stands for the secret or proxy password of the saved channel with the id of the form.
   */
  @POST
  @Path("test")
  @RequiresAuthentication
  @RequiresPermissions("astrum:settings:update")
  public NotificationTestResultXO testUnsaved(final NotificationChannelXO xo) {
    require(xo);
    NotificationChannel existing = isNullOrEmpty(xo.getId()) ? null : store.find(xo.getId()).orElse(null);
    NotificationChannel channel = toChannel(xo, existing, true);
    Secret secret = secret(xo, channel, existing);
    String value = secret.change().replace() ? secret.change().value() : store.secret(existing);
    SecretChange proxyPassword = proxyPassword(xo, channel, existing);
    String password =
        proxyPassword.replace() || existing == null ? proxyPassword.value() : store.proxyPassword(existing);
    return sendTest(channel, value, password);
  }

  private NotificationTestResultXO sendTest(
      final NotificationChannel channel,
      @Nullable final String secret,
      @Nullable final String proxyPassword)
  {
    if (secret == null) {
      throw badRequest("secret", secretRequired(channel));
    }
    try {
      transport.send(channel, new NotificationSecrets(secret, proxyPassword),
          NotificationMessages.test(channel, baseUrlManager.getOrDetect()));
      return new NotificationTestResultXO(true, null);
    }
    catch (NotificationDeliveryException e) {
      return new NotificationTestResultXO(false, e.getMessage());
    }
  }

  private NotificationChannel find(final String id) {
    return store.find(id).orElseThrow(NotificationChannelsResource::notFound);
  }

  private static NotificationChannelXO require(@Nullable final NotificationChannelXO xo) {
    if (xo == null) {
      throw badRequest("*", "The channel is missing");
    }
    return xo;
  }

  private NotificationChannel toChannel(
      final NotificationChannelXO xo,
      @Nullable final NotificationChannel existing,
      final boolean forTest)
  {
    NotificationChannel channel = new NotificationChannel();
    String type = trim(xo.getType());
    if (!NotificationChannel.TELEGRAM.equals(type) && !NotificationChannel.SLACK.equals(type)) {
      throw badRequest("type", "Type must be telegram or slack");
    }
    channel.setType(type);

    String given = trim(xo.getName());
    String name = given == null && forTest ? (channel.isTelegram() ? "Telegram" : "Slack") : given;
    if (name == null) {
      throw badRequest("name", "Name is required");
    }
    if (name.length() > MAX_NAME_LENGTH) {
      throw badRequest("name", "Name must not be longer than " + MAX_NAME_LENGTH + " characters");
    }
    if (!forTest && store.list().stream()
        .anyMatch(other -> other.getName().equalsIgnoreCase(name)
            && (existing == null || !other.getId().equals(existing.getId())))) {
      throw badRequest("name", "A channel with this name already exists");
    }
    channel.setName(name);
    channel.setEnabled(xo.isEnabled());

    if (channel.isTelegram()) {
      String chatId = trim(xo.getChatId());
      if (chatId == null) {
        throw badRequest("chatId", "Chat ID is required");
      }
      if (!CHAT_ID.matcher(chatId).matches()) {
        throw badRequest("chatId",
            "Chat ID must be a number such as -1001234567890 or a public channel name such as @releases");
      }
      channel.setChatId(chatId);
      Long threadId = xo.getMessageThreadId();
      if (threadId != null && threadId <= 0) {
        throw badRequest("messageThreadId", "Topic ID must be a positive number");
      }
      channel.setMessageThreadId(threadId);
    }

    String language = trim(xo.getLanguage());
    if (language == null) {
      language = NotificationChannel.RUSSIAN;
    }
    if (!NotificationChannel.RUSSIAN.equals(language) && !NotificationChannel.ENGLISH.equals(language)) {
      throw badRequest("language", "Language must be ru or en");
    }
    channel.setLanguage(language);

    proxy(xo, channel);

    if (!forTest) {
      channel.setEvents(events(xo.getEvents()));
      channel.setRepositories(repositories(xo.getRepositories()));
    }
    return channel;
  }

  private static void proxy(final NotificationChannelXO xo, final NotificationChannel channel) {
    String mode = trim(xo.getProxyMode());
    if (mode == null) {
      mode = NotificationChannel.PROXY_GLOBAL;
    }
    if (!List.of(NotificationChannel.PROXY_GLOBAL, NotificationChannel.PROXY_NONE, NotificationChannel.PROXY_CUSTOM)
        .contains(mode)) {
      throw badRequest("proxyMode", "Connection must be global, none or custom");
    }
    channel.setProxyMode(mode);
    if (!channel.isCustomProxy()) {
      return;
    }

    String type = trim(xo.getProxyType());
    if (!NotificationChannel.PROXY_HTTP.equals(type) && !NotificationChannel.PROXY_SOCKS5.equals(type)) {
      throw badRequest("proxyType", "Proxy type must be http or socks5");
    }
    channel.setProxyType(type);

    String host = trim(xo.getProxyHost());
    if (host == null) {
      throw badRequest("proxyHost", "Proxy host is required");
    }
    if (host.startsWith("[") && host.endsWith("]")) {
      host = host.substring(1, host.length() - 1);
    }
    if (host.length() > MAX_PROXY_HOST_LENGTH) {
      throw badRequest("proxyHost", "Proxy host must not be longer than " + MAX_PROXY_HOST_LENGTH + " characters");
    }
    if (!PROXY_HOST.matcher(host).matches()) {
      throw badRequest("proxyHost", "Proxy host must be a host name or an IP address, without a scheme or port");
    }
    channel.setProxyHost(host);

    Integer port = xo.getProxyPort();
    if (port == null) {
      throw badRequest("proxyPort", "Proxy port is required");
    }
    if (port < 1 || port > 65535) {
      throw badRequest("proxyPort", "Proxy port must be between 1 and 65535");
    }
    channel.setProxyPort(port);

    String username = trim(xo.getProxyUsername());
    if (username != null && username.getBytes(UTF_8).length > MAX_PROXY_CREDENTIAL_BYTES) {
      throw badRequest("proxyUsername",
          "Proxy username must not be longer than " + MAX_PROXY_CREDENTIAL_BYTES + " characters");
    }
    channel.setProxyUsername(username);
  }

  /**
   * The proxy password of the form: a new password, the stored one (placeholder or no value), or none (empty). The
   * stored password is kept only for the proxy it was entered for, so that it cannot be sent to another host.
   */
  private static SecretChange proxyPassword(
      final NotificationChannelXO xo,
      final NotificationChannel channel,
      @Nullable final NotificationChannel existing)
  {
    String value = xo.getProxyPassword();
    boolean keep = value == null || PasswordPlaceholder.is(value);
    boolean stored = existing != null && existing.getProxyPasswordSecretId() != null;
    if (!channel.isCustomProxy() || channel.getProxyUsername() == null) {
      if (channel.isCustomProxy() && !keep && !value.isEmpty()) {
        throw badRequest("proxyUsername", "Enter the username that goes with the proxy password");
      }
      return stored ? SecretChange.clear() : SecretChange.keep();
    }
    if (keep) {
      if (stored && !sameProxy(existing, channel)) {
        throw badRequest("proxyPassword", "Enter the proxy password again: the proxy has changed");
      }
      return SecretChange.keep();
    }
    if (value.isEmpty()) {
      return stored ? SecretChange.clear() : SecretChange.keep();
    }
    if (value.getBytes(UTF_8).length > MAX_PROXY_CREDENTIAL_BYTES) {
      throw badRequest("proxyPassword",
          "Proxy password must not be longer than " + MAX_PROXY_CREDENTIAL_BYTES + " characters");
    }
    return SecretChange.set(value);
  }

  private static boolean sameProxy(final NotificationChannel existing, final NotificationChannel channel) {
    return existing.isCustomProxy()
        && Objects.equals(existing.getProxyType(), channel.getProxyType())
        && existing.getProxyHost() != null && existing.getProxyHost().equalsIgnoreCase(channel.getProxyHost())
        && Objects.equals(existing.getProxyPort(), channel.getProxyPort());
  }

  private static List<String> events(@Nullable final List<String> events) {
    Set<NoticeAction> actions = new LinkedHashSet<>();
    if (events != null) {
      for (String event : events) {
        actions.add(NoticeAction.of(trim(event))
            .orElseThrow(() -> badRequest("events", "Unknown event: " + event)));
      }
    }
    if (actions.isEmpty()) {
      throw badRequest("events", "Choose at least one event");
    }
    List<String> ordered = new ArrayList<>();
    for (NoticeAction action : NoticeAction.values()) {
      if (actions.contains(action)) {
        ordered.add(action.id());
      }
    }
    return ordered;
  }

  private List<String> repositories(@Nullable final List<String> repositories) {
    Set<String> names = new LinkedHashSet<>();
    if (repositories != null) {
      for (String repository : repositories) {
        String name = trim(repository);
        if (name == null) {
          continue;
        }
        if (repositoryManager.get(name) == null) {
          throw badRequest("repositories", "Repository " + name + " does not exist");
        }
        names.add(name);
      }
    }
    return new ArrayList<>(names);
  }

  private record Secret(SecretChange change, @Nullable String targetHost)
  {
  }

  private static Secret secret(
      final NotificationChannelXO xo,
      final NotificationChannel channel,
      @Nullable final NotificationChannel existing)
  {
    String value = xo.getSecret();
    boolean blank = isNullOrEmpty(value) || value.isBlank() || PasswordPlaceholder.is(value);
    if (blank) {
      if (existing != null && existing.getSecretId() != null && existing.getType().equals(channel.getType())) {
        return new Secret(SecretChange.keep(), existing.getTargetHost());
      }
      throw badRequest("secret", secretRequired(channel));
    }
    String secret = value.strip();
    if (channel.isTelegram()) {
      if (!HttpNotificationTransport.BOT_TOKEN.matcher(secret).matches()) {
        throw badRequest("secret", "The bot token must look like 123456789:AAHdqTcvCH1vGWJxfSeofSAs0K5PALDsaw");
      }
      return new Secret(SecretChange.set(secret), null);
    }
    URI uri = HttpNotificationTransport.webhookUri(secret);
    if (uri == null) {
      throw badRequest("secret", "The webhook URL must be an http or https URL");
    }
    return new Secret(SecretChange.set(secret), uri.getHost());
  }

  private static String secretRequired(final NotificationChannel channel) {
    return channel.isTelegram() ? "Bot token is required" : "Webhook URL is required";
  }

  static NotificationChannelXO toXO(final NotificationChannel channel) {
    NotificationChannelXO xo = new NotificationChannelXO();
    xo.setId(channel.getId());
    xo.setName(channel.getName());
    xo.setType(channel.getType());
    xo.setEnabled(channel.isEnabled());
    xo.setChatId(channel.getChatId());
    xo.setMessageThreadId(channel.getMessageThreadId());
    xo.setSecret(channel.getSecretId() != null ? PasswordPlaceholder.get() : "");
    xo.setTarget(channel.isTelegram() ? channel.getChatId() : channel.getTargetHost());
    xo.setRepositories(new ArrayList<>(channel.getRepositories()));
    xo.setEvents(new ArrayList<>(channel.getEvents()));
    xo.setLanguage(channel.getLanguage());
    xo.setProxyMode(channel.getProxyMode());
    if (channel.isCustomProxy()) {
      xo.setProxyType(channel.getProxyType());
      xo.setProxyHost(channel.getProxyHost());
      xo.setProxyPort(channel.getProxyPort());
      xo.setProxyUsername(channel.getProxyUsername());
    }
    xo.setProxyPassword(channel.getProxyPasswordSecretId() != null ? PasswordPlaceholder.get() : "");
    return xo;
  }

  @Nullable
  private static String trim(@Nullable final String value) {
    if (value == null) {
      return null;
    }
    String trimmed = value.strip();
    return trimmed.isEmpty() ? null : trimmed;
  }

  private static WebApplicationMessageException badRequest(final String field, final String message) {
    return new WebApplicationMessageException(Status.BAD_REQUEST, field, message, APPLICATION_JSON);
  }

  private static WebApplicationMessageException notFound() {
    return new WebApplicationMessageException(Status.NOT_FOUND, "Notification channel not found", APPLICATION_JSON);
  }
}
