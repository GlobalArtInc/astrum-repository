package org.globalart.astrum.notifications.internal;

import java.util.List;
import java.util.Optional;

import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.crypto.secrets.Secret;
import org.globalart.astrum.crypto.secrets.SecretsService;
import org.globalart.astrum.kv.GlobalKeyValueStore;
import org.globalart.astrum.kv.KeyValueEvent;
import org.globalart.astrum.notifications.internal.NotificationChannelStore.Channels;
import org.globalart.astrum.notifications.internal.NotificationChannelStore.SecretChange;
import org.globalart.astrum.notifications.internal.NotificationChannelStore.SecretChanges;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.junit.Before;
import org.junit.Test;
import org.mockito.ArgumentCaptor;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.empty;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.containsString;
import static org.hamcrest.Matchers.not;
import static org.hamcrest.Matchers.notNullValue;
import static org.hamcrest.Matchers.nullValue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.argThat;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

public class NotificationChannelStoreTest
{
  private static final NotificationSecrets TOKEN_AND_PASSWORD = new NotificationSecrets("1:token", "pass", null, null);

  private GlobalKeyValueStore keyValueStore;

  private SecretsService secretsService;

  private EventManager eventManager;

  private NotificationChannelStore store;

  private int secrets;

  @Before
  public void setUp() {
    keyValueStore = mock(GlobalKeyValueStore.class);
    secretsService = mock(SecretsService.class);
    eventManager = mock(EventManager.class);
    when(keyValueStore.get(NotificationChannelStore.KEY, Channels.class)).thenReturn(Optional.empty());
    when(secretsService.encrypt(eq(NotificationChannelStore.SECRET_PURPOSE), any(char[].class), any()))
        .thenAnswer(invocation -> secret("_" + ++secrets, new String((char[]) invocation.getArgument(1))));
    store = new NotificationChannelStore(keyValueStore, secretsService, eventManager);
  }

  private Secret secret(final String id, final String value) {
    Secret secret = mock(Secret.class);
    when(secret.getId()).thenReturn(id);
    when(secret.decrypt()).thenReturn(value.toCharArray());
    when(secretsService.from(id)).thenReturn(secret);
    return secret;
  }

  private static SecretChanges change(final SecretChange secret, final SecretChange proxyPassword) {
    return new SecretChanges(secret, proxyPassword, SecretChange.keep(), SecretChange.keep());
  }

  private static NotificationChannel channel(final String name) {
    NotificationChannel channel = new NotificationChannel();
    channel.setName(name);
    channel.setType(NotificationChannel.TELEGRAM);
    channel.setChatId("-100");
    channel.setEvents(List.of("published"));
    return channel;
  }

  @Test
  public void createKeepsTheTokenInTheSecretsStore() {
    NotificationChannel created = store.create(channel("releases"), NotificationSecrets.of("1:token"), "admin");

    assertThat(created.getId(), is(notNullValue()));
    assertThat(created.getSecretId(), is("_1"));
    assertThat(store.secret(created), is("1:token"));
    ArgumentCaptor<Object> saved = ArgumentCaptor.forClass(Object.class);
    verify(keyValueStore).setString(eq(NotificationChannelStore.KEY), saved.capture());
    Channels channels = (Channels) saved.getValue();
    assertThat(channels.getChannels(), hasSize(1));
    assertThat(channels.getChannels().get(0).getSecretId(), is("_1"));
    verify(eventManager).post(argThat(event -> event instanceof NotificationChannelChangedEvent changed
        && changed.getType() == NotificationChannelChangedEvent.Type.CREATED));
  }

  @Test
  public void updateWithThePlaceholderKeepsTheSecret() {
    NotificationChannel created = store.create(channel("releases"), NotificationSecrets.of("1:token"), "admin");
    NotificationChannel changed = created.copy();
    changed.setName("builds");
    changed.setSecretId(null);

    NotificationChannel updated = store.update(changed, SecretChanges.keepAll(), "admin");

    assertThat(updated.getName(), is("builds"));
    assertThat(updated.getSecretId(), is("_1"));
    assertThat(store.secret(updated), is("1:token"));
    verify(secretsService, never()).remove(any());
  }

  @Test
  public void updateWithANewSecretReplacesAndRemovesTheOldOne() {
    NotificationChannel created = store.create(channel("releases"), NotificationSecrets.of("1:token"), "admin");
    Secret old = secretsService.from("_1");

    NotificationChannel updated = store.update(created, change(SecretChange.set("2:other"), SecretChange.keep()),
        "admin");

    assertThat(updated.getSecretId(), is("_2"));
    assertThat(store.secret(updated), is("2:other"));
    verify(secretsService).remove(old);
  }

  @Test
  public void deleteRemovesTheChannelAndItsSecret() {
    NotificationChannel created = store.create(channel("releases"), NotificationSecrets.of("1:token"), "admin");
    Secret secret = secretsService.from("_1");

    assertThat(store.delete(created.getId()), is(true));
    assertThat(store.delete(created.getId()), is(false));

    assertThat(store.list(), is(empty()));
    verify(secretsService).remove(secret);
    verify(eventManager, times(2)).post(any(NotificationChannelChangedEvent.class));
  }

  @Test
  public void listReturnsCopies() {
    NotificationChannel created = store.create(channel("releases"), NotificationSecrets.of("1:token"), "admin");

    store.list().get(0).setName("changed");
    store.find(created.getId()).get().getEvents().add("deleted");

    assertThat(store.list().get(0).getName(), is("releases"));
    assertThat(store.list().get(0).getEvents(), contains("published"));
  }

  @Test
  public void enabledSkipsDisabledChannels() {
    NotificationChannel first = channel("first");
    NotificationChannel second = channel("second");
    second.setEnabled(false);
    store.create(first, NotificationSecrets.of("1:a"), "admin");
    store.create(second, NotificationSecrets.of("1:b"), "admin");

    assertThat(store.enabled(), hasSize(1));
    assertThat(store.enabled().get(0).getName(), is("first"));
  }

  @Test
  public void reloadsAfterTheKeyChangesElsewhere() {
    NotificationChannel stored = channel("from-another-node");
    stored.setId("x");
    when(keyValueStore.get(NotificationChannelStore.KEY, Channels.class))
        .thenReturn(Optional.of(new Channels(List.of(stored))));

    store.on(new KeyValueEvent(NotificationChannelStore.KEY, null));

    assertThat(store.list().get(0).getName(), is("from-another-node"));
  }

  private static NotificationChannel proxied(final String name) {
    NotificationChannel channel = channel(name);
    channel.setProxyMode(NotificationChannel.PROXY_CUSTOM);
    channel.setProxyType(NotificationChannel.PROXY_HTTP);
    channel.setProxyHost("proxy.internal");
    channel.setProxyPort(3128);
    channel.setProxyUsername("robot");
    return channel;
  }

  @Test
  public void createKeepsTheProxyPasswordInTheSecretsStore() {
    NotificationChannel created = store.create(proxied("releases"), TOKEN_AND_PASSWORD, "admin");

    assertThat(created.getProxyPasswordSecretId(), is("_2"));
    assertThat(store.proxyPassword(created), is("pass"));
    NotificationSecrets secrets = store.secrets(created);
    assertThat(secrets.secret(), is("1:token"));
    assertThat(secrets.proxyPassword(), is("pass"));
    assertThat(secrets.toString(), not(containsString("pass")));
  }

  @Test
  public void channelsWithoutAProxyPasswordHaveNone() {
    NotificationChannel created = store.create(proxied("releases"), NotificationSecrets.of("1:token"), "admin");

    assertThat(created.getProxyPasswordSecretId(), is(nullValue()));
    assertThat(store.secrets(created).proxyPassword(), is(nullValue()));
    verify(secretsService, times(1)).encrypt(any(), any(char[].class), any());
  }

  @Test
  public void keepingTheProxyPasswordLeavesItAlone() {
    NotificationChannel created = store.create(proxied("releases"), TOKEN_AND_PASSWORD, "admin");
    NotificationChannel changed = created.copy();
    changed.setProxyPasswordSecretId(null);
    changed.setProxyPort(8080);

    NotificationChannel updated = store.update(changed, SecretChanges.keepAll(), "admin");

    assertThat(updated.getProxyPasswordSecretId(), is("_2"));
    assertThat(store.proxyPassword(updated), is("pass"));
    verify(secretsService, never()).remove(any());
  }

  @Test
  public void aNewProxyPasswordReplacesAndRemovesTheOldOne() {
    NotificationChannel created = store.create(proxied("releases"), TOKEN_AND_PASSWORD, "admin");
    Secret old = secretsService.from("_2");

    NotificationChannel updated =
        store.update(created, change(SecretChange.keep(), SecretChange.set("other")), "admin");

    assertThat(store.proxyPassword(updated), is("other"));
    assertThat(updated.getSecretId(), is("_1"));
    verify(secretsService).remove(old);
    verify(secretsService, never()).remove(secretsService.from("_1"));
  }

  @Test
  public void clearingTheProxyPasswordRemovesIt() {
    NotificationChannel created = store.create(proxied("releases"), TOKEN_AND_PASSWORD, "admin");
    Secret old = secretsService.from("_2");
    NotificationChannel direct = created.copy();
    direct.setProxyMode(NotificationChannel.PROXY_NONE);

    NotificationChannel updated = store.update(direct, change(SecretChange.keep(), SecretChange.clear()), "admin");

    assertThat(updated.getProxyPasswordSecretId(), is(nullValue()));
    assertThat(store.proxyPassword(updated), is(nullValue()));
    verify(secretsService).remove(old);
  }

  @Test
  public void deleteRemovesTheProxyPasswordToo() {
    NotificationChannel created = store.create(proxied("releases"), TOKEN_AND_PASSWORD, "admin");
    Secret token = secretsService.from("_1");
    Secret password = secretsService.from("_2");

    store.delete(created.getId());

    verify(secretsService).remove(token);
    verify(secretsService).remove(password);
  }

  @Test
  public void channelsStoredBeforeTheConnectionCouldBeChosenUseTheGlobalSettings() throws Exception {
    Channels stored = new ObjectMapper().readValue("{\"channels\":[{\"id\":\"x\",\"name\":\"old\","
        + "\"type\":\"telegram\",\"chatId\":\"-100\",\"secretId\":\"_9\",\"events\":[\"published\"]}]}",
        Channels.class);
    NotificationChannel legacy = stored.getChannels().get(0);

    assertThat(legacy.getProxyMode(), is(NotificationChannel.PROXY_GLOBAL));
    assertThat(legacy.isCustomProxy(), is(false));
    assertThat(legacy.isDirect(), is(false));
    assertThat(legacy.getProxyPasswordSecretId(), is(nullValue()));

    NotificationChannel explicitNull = new ObjectMapper()
        .readValue("{\"id\":\"y\",\"proxyMode\":null}", NotificationChannel.class);
    assertThat(explicitNull.getProxyMode(), is(NotificationChannel.PROXY_GLOBAL));
  }

  @Test
  public void storedChannelsKeepTheirConnectionButNotTheDerivedFlags() throws Exception {
    NotificationChannel channel = proxied("releases");
    channel.setProxyType(NotificationChannel.PROXY_SOCKS5);

    String json = new ObjectMapper().writeValueAsString(channel);

    assertThat(json, containsString("\"proxyMode\":\"custom\""));
    assertThat(json, containsString("\"proxyType\":\"socks5\""));
    assertThat(json, not(containsString("socksProxy")));
    assertThat(json, not(containsString("customProxy")));
    NotificationChannel read = new ObjectMapper().readValue(json, NotificationChannel.class);
    assertThat(read.isSocksProxy(), is(true));
    assertThat(read.copy().getProxyUsername(), is("robot"));
  }

  private static NotificationChannel webhook(final String name) {
    NotificationChannel channel = channel(name);
    channel.setType(NotificationChannel.WEBHOOK);
    channel.setChatId(null);
    channel.setTargetHost("hooks.internal");
    return channel;
  }

  @Test
  public void webhookSecretsLiveInTheSecretsStore() {
    NotificationChannel created = store.create(webhook("ci"),
        new NotificationSecrets("https://hooks.internal/x", null, "signing", "Bearer abc"), "admin");

    assertThat(created.getSigningSecretId(), is("_2"));
    assertThat(created.getAuthorizationSecretId(), is("_3"));
    NotificationSecrets secrets = store.secrets(created);
    assertThat(secrets.secret(), is("https://hooks.internal/x"));
    assertThat(secrets.signingSecret(), is("signing"));
    assertThat(secrets.authorization(), is("Bearer abc"));
    assertThat(secrets.proxyPassword(), is(nullValue()));
  }

  @Test
  public void webhookSecretsAreKeptReplacedRemovedAndDeleted() {
    NotificationChannel created = store.create(webhook("ci"),
        new NotificationSecrets("https://hooks.internal/x", null, "signing", "Bearer abc"), "admin");
    Secret signing = secretsService.from("_2");
    Secret authorization = secretsService.from("_3");

    NotificationChannel kept = store.update(created, SecretChanges.keepAll(), "admin");
    assertThat(store.secrets(kept).signingSecret(), is("signing"));
    verify(secretsService, never()).remove(any());

    NotificationChannel changed = store.update(created, new SecretChanges(SecretChange.keep(), SecretChange.keep(),
        SecretChange.set("other"), SecretChange.clear()), "admin");
    assertThat(store.secrets(changed).signingSecret(), is("other"));
    assertThat(changed.getAuthorizationSecretId(), is(nullValue()));
    assertThat(store.secrets(changed).authorization(), is(nullValue()));
    verify(secretsService).remove(signing);
    verify(secretsService).remove(authorization);

    Secret newSigning = secretsService.from(changed.getSigningSecretId());
    store.delete(changed.getId());
    verify(secretsService).remove(newSigning);
  }

  @Test(expected = IllegalArgumentException.class)
  public void theBotTokenOrWebhookUrlCannotBeRemoved() {
    NotificationChannel created = store.create(channel("releases"), NotificationSecrets.of("1:token"), "admin");

    store.update(created, change(SecretChange.clear(), SecretChange.keep()), "admin");
  }
}
