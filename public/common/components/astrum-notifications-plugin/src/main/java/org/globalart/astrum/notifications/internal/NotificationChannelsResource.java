package org.globalart.astrum.notifications.internal;

import java.net.URI;
import java.util.ArrayList;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Set;
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

/**
 * Manages the notification channels and sends test messages.
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

  private static final Pattern CHAT_ID = Pattern.compile("-?\\d{1,20}|@[A-Za-z][A-Za-z0-9_]{3,31}");

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
    return toXO(store.create(channel, secret.change().value(), UserIdHelper.get()));
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
    return toXO(store.update(channel, secret.change(), UserIdHelper.get()));
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
    return sendTest(channel, store.secret(channel));
  }

  /**
   * Sends a test message with the settings of the form, which may not be saved yet. A secret placeholder stands for
   * the secret of the saved channel with the id of the form.
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
    return sendTest(channel, value);
  }

  private NotificationTestResultXO sendTest(final NotificationChannel channel, @Nullable final String secret) {
    if (secret == null) {
      throw badRequest("secret", secretRequired(channel));
    }
    try {
      transport.send(channel, secret, NotificationMessages.test(channel, baseUrlManager.getOrDetect()));
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

    if (!forTest) {
      channel.setEvents(events(xo.getEvents()));
      channel.setRepositories(repositories(xo.getRepositories()));
    }
    return channel;
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
