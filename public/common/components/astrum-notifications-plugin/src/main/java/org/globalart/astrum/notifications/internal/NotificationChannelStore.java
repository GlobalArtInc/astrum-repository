package org.globalart.astrum.notifications.internal;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.UUID;

import javax.annotation.Nullable;

import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.crypto.secrets.Secret;
import org.globalart.astrum.crypto.secrets.SecretsService;
import org.globalart.astrum.kv.GlobalKeyValueStore;
import org.globalart.astrum.kv.KeyValueEvent;
import org.globalart.astrum.notifications.internal.NotificationChannelChangedEvent.Type;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Keeps the notification channels in the key-value store, and their bot tokens and webhook URLs in the secrets store.
 */
@Component
public class NotificationChannelStore
    implements EventAware
{
  static final String KEY = "astrum.notifications.channels";

  static final String SECRET_PURPOSE = "notifications";

  private static final Logger log = LoggerFactory.getLogger(NotificationChannelStore.class);

  private final GlobalKeyValueStore keyValueStore;

  private final SecretsService secretsService;

  private final EventManager eventManager;

  private volatile List<NotificationChannel> cached;

  @Autowired
  public NotificationChannelStore(
      final GlobalKeyValueStore keyValueStore,
      final SecretsService secretsService,
      final EventManager eventManager)
  {
    this.keyValueStore = checkNotNull(keyValueStore);
    this.secretsService = checkNotNull(secretsService);
    this.eventManager = checkNotNull(eventManager);
  }

  /**
   * @return copies of the channels, in the order they were created
   */
  public List<NotificationChannel> list() {
    return snapshot().stream().map(NotificationChannel::copy).toList();
  }

  /**
   * @return a copy of the channel
   */
  public Optional<NotificationChannel> find(final String id) {
    return snapshot().stream().filter(channel -> channel.getId().equals(id)).findFirst().map(NotificationChannel::copy);
  }

  /**
   * @return the enabled channels, shared and never to be changed by the caller
   */
  List<NotificationChannel> enabled() {
    return snapshot().stream().filter(NotificationChannel::isEnabled).toList();
  }

  /**
   * Whether a secret change keeps the stored value or replaces it.
   */
  public record SecretChange(boolean replace, String value)
  {
    public static SecretChange keep() {
      return new SecretChange(false, null);
    }

    public static SecretChange set(final String value) {
      return new SecretChange(true, checkNotNull(value));
    }
  }

  public synchronized NotificationChannel create(
      final NotificationChannel channel,
      final String secret,
      @Nullable final String userId)
  {
    NotificationChannel created = channel.copy();
    created.setId(UUID.randomUUID().toString());
    created.setSecretId(encrypt(secret, userId));
    List<NotificationChannel> channels = new ArrayList<>(snapshot());
    channels.add(created);
    save(channels);
    eventManager.post(new NotificationChannelChangedEvent(Type.CREATED, created));
    return created.copy();
  }

  public synchronized NotificationChannel update(
      final NotificationChannel channel,
      final SecretChange secret,
      @Nullable final String userId)
  {
    List<NotificationChannel> channels = new ArrayList<>(snapshot());
    int index = indexOf(channels, channel.getId());
    checkArgument(index >= 0, "Unknown notification channel %s", channel.getId());
    String previousSecretId = channels.get(index).getSecretId();
    NotificationChannel updated = channel.copy();
    updated.setSecretId(secret.replace() ? encrypt(secret.value(), userId) : previousSecretId);
    channels.set(index, updated);
    save(channels);
    if (secret.replace()) {
      removeSecret(previousSecretId);
    }
    eventManager.post(new NotificationChannelChangedEvent(Type.UPDATED, updated));
    return updated.copy();
  }

  public synchronized boolean delete(final String id) {
    List<NotificationChannel> channels = new ArrayList<>(snapshot());
    int index = indexOf(channels, id);
    if (index < 0) {
      return false;
    }
    NotificationChannel removed = channels.remove(index);
    save(channels);
    removeSecret(removed.getSecretId());
    eventManager.post(new NotificationChannelChangedEvent(Type.DELETED, removed));
    return true;
  }

  /**
   * @return the decrypted bot token or webhook URL of the channel, or {@code null} if it has none
   */
  @Nullable
  public String secret(final NotificationChannel channel) {
    if (channel.getSecretId() == null) {
      return null;
    }
    return new String(secretsService.from(channel.getSecretId()).decrypt());
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final KeyValueEvent event) {
    if (KEY.equals(event.getKey())) {
      cached = null;
    }
  }

  private List<NotificationChannel> snapshot() {
    List<NotificationChannel> current = cached;
    if (current == null) {
      current = keyValueStore.get(KEY, Channels.class)
          .map(Channels::getChannels)
          .map(channels -> channels.stream().map(NotificationChannel::copy).toList())
          .orElse(List.of());
      cached = current;
    }
    return current;
  }

  private void save(final List<NotificationChannel> channels) {
    List<NotificationChannel> copy = channels.stream().map(NotificationChannel::copy).toList();
    keyValueStore.setString(KEY, (Object) new Channels(new ArrayList<>(copy)));
    cached = copy;
  }

  private String encrypt(final String secret, @Nullable final String userId) {
    Secret encrypted = secretsService.encrypt(SECRET_PURPOSE, checkNotNull(secret).toCharArray(), userId);
    return encrypted.getId();
  }

  private void removeSecret(@Nullable final String secretId) {
    if (secretId == null) {
      return;
    }
    try {
      secretsService.remove(secretsService.from(secretId));
    }
    catch (RuntimeException e) {
      log.warn("Could not remove the notification channel secret {}", secretId, e);
    }
  }

  private static int indexOf(final List<NotificationChannel> channels, final String id) {
    for (int i = 0; i < channels.size(); i++) {
      if (channels.get(i).getId().equals(id)) {
        return i;
      }
    }
    return -1;
  }

  @JsonIgnoreProperties(ignoreUnknown = true)
  public static class Channels
  {
    private List<NotificationChannel> channels = new ArrayList<>();

    public Channels() {
    }

    public Channels(final List<NotificationChannel> channels) {
      this.channels = channels;
    }

    public List<NotificationChannel> getChannels() {
      return channels;
    }

    public void setChannels(final List<NotificationChannel> channels) {
      this.channels = channels != null ? channels : new ArrayList<>();
    }
  }
}
