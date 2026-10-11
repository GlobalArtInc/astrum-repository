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
 * Keeps the notification channels in the key-value store, and their bot tokens, webhook URLs, proxy passwords, signing
 * secrets and Authorization headers in the secrets store.
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
   * Whether a secret change keeps the stored value or replaces it; a replacement without a value removes the secret.
   */
  public record SecretChange(boolean replace, @Nullable String value)
  {
    public static SecretChange keep() {
      return new SecretChange(false, null);
    }

    public static SecretChange set(final String value) {
      return new SecretChange(true, checkNotNull(value));
    }

    public static SecretChange clear() {
      return new SecretChange(true, null);
    }
  }

  /**
   * How each secret of a channel changes.
   */
  public record SecretChanges(
      SecretChange secret,
      SecretChange proxyPassword,
      SecretChange signingSecret,
      SecretChange authorization)
  {
    public static SecretChanges keepAll() {
      return new SecretChanges(SecretChange.keep(), SecretChange.keep(), SecretChange.keep(), SecretChange.keep());
    }
  }

  public synchronized NotificationChannel create(
      final NotificationChannel channel,
      final NotificationSecrets secrets,
      @Nullable final String userId)
  {
    NotificationChannel created = channel.copy();
    created.setId(UUID.randomUUID().toString());
    created.setSecretId(encrypt(secrets.secret(), userId));
    created.setProxyPasswordSecretId(encryptOptional(secrets.proxyPassword(), userId));
    created.setSigningSecretId(encryptOptional(secrets.signingSecret(), userId));
    created.setAuthorizationSecretId(encryptOptional(secrets.authorization(), userId));
    List<NotificationChannel> channels = new ArrayList<>(snapshot());
    channels.add(created);
    save(channels);
    eventManager.post(new NotificationChannelChangedEvent(Type.CREATED, created));
    return created.copy();
  }

  public synchronized NotificationChannel update(
      final NotificationChannel channel,
      final SecretChanges changes,
      @Nullable final String userId)
  {
    List<NotificationChannel> channels = new ArrayList<>(snapshot());
    int index = indexOf(channels, channel.getId());
    checkArgument(index >= 0, "Unknown notification channel %s", channel.getId());
    checkArgument(!changes.secret().replace() || changes.secret().value() != null, "The secret cannot be removed");
    NotificationChannel previous = channels.get(index);
    List<String> replaced = new ArrayList<>();
    NotificationChannel updated = channel.copy();
    updated.setSecretId(apply(changes.secret(), previous.getSecretId(), replaced, userId));
    updated.setProxyPasswordSecretId(
        apply(changes.proxyPassword(), previous.getProxyPasswordSecretId(), replaced, userId));
    updated.setSigningSecretId(apply(changes.signingSecret(), previous.getSigningSecretId(), replaced, userId));
    updated.setAuthorizationSecretId(
        apply(changes.authorization(), previous.getAuthorizationSecretId(), replaced, userId));
    channels.set(index, updated);
    save(channels);
    replaced.forEach(this::removeSecret);
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
    removeSecret(removed.getProxyPasswordSecretId());
    removeSecret(removed.getSigningSecretId());
    removeSecret(removed.getAuthorizationSecretId());
    eventManager.post(new NotificationChannelChangedEvent(Type.DELETED, removed));
    return true;
  }

  /**
   * @return the decrypted bot token or webhook URL of the channel, or {@code null} if it has none
   */
  @Nullable
  public String secret(final NotificationChannel channel) {
    return decrypt(channel.getSecretId());
  }

  /**
   * @return the decrypted secrets of the channel, or {@code null} if it has no bot token or webhook URL
   */
  @Nullable
  public NotificationSecrets secrets(final NotificationChannel channel) {
    String secret = secret(channel);
    if (secret == null) {
      return null;
    }
    return new NotificationSecrets(secret, proxyPassword(channel), decrypt(channel.getSigningSecretId()),
        decrypt(channel.getAuthorizationSecretId()));
  }

  /**
   * @return the decrypted password of the channel's own proxy, or {@code null} if it has none
   */
  @Nullable
  public String proxyPassword(final NotificationChannel channel) {
    return decrypt(channel.getProxyPasswordSecretId());
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

  @Nullable
  private String decrypt(@Nullable final String secretId) {
    if (secretId == null) {
      return null;
    }
    return new String(secretsService.from(secretId).decrypt());
  }

  /**
   * @return the id of the secret after the change; the id of a secret that is replaced or removed is added to
   *         {@code replaced}
   */
  @Nullable
  private String apply(
      final SecretChange change,
      @Nullable final String previousId,
      final List<String> replaced,
      @Nullable final String userId)
  {
    if (!change.replace()) {
      return previousId;
    }
    if (previousId != null) {
      replaced.add(previousId);
    }
    return encryptOptional(change.value(), userId);
  }

  @Nullable
  private String encryptOptional(@Nullable final String secret, @Nullable final String userId) {
    return secret != null ? encrypt(secret, userId) : null;
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
