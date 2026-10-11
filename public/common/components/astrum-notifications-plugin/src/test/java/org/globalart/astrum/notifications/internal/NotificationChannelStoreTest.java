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

import org.junit.Before;
import org.junit.Test;
import org.mockito.ArgumentCaptor;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.empty;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
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
    NotificationChannel created = store.create(channel("releases"), "1:token", "admin");

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
    NotificationChannel created = store.create(channel("releases"), "1:token", "admin");
    NotificationChannel changed = created.copy();
    changed.setName("builds");
    changed.setSecretId(null);

    NotificationChannel updated = store.update(changed, SecretChange.keep(), "admin");

    assertThat(updated.getName(), is("builds"));
    assertThat(updated.getSecretId(), is("_1"));
    assertThat(store.secret(updated), is("1:token"));
    verify(secretsService, never()).remove(any());
  }

  @Test
  public void updateWithANewSecretReplacesAndRemovesTheOldOne() {
    NotificationChannel created = store.create(channel("releases"), "1:token", "admin");
    Secret old = secretsService.from("_1");

    NotificationChannel updated = store.update(created, SecretChange.set("2:other"), "admin");

    assertThat(updated.getSecretId(), is("_2"));
    assertThat(store.secret(updated), is("2:other"));
    verify(secretsService).remove(old);
  }

  @Test
  public void deleteRemovesTheChannelAndItsSecret() {
    NotificationChannel created = store.create(channel("releases"), "1:token", "admin");
    Secret secret = secretsService.from("_1");

    assertThat(store.delete(created.getId()), is(true));
    assertThat(store.delete(created.getId()), is(false));

    assertThat(store.list(), is(empty()));
    verify(secretsService).remove(secret);
    verify(eventManager, times(2)).post(any(NotificationChannelChangedEvent.class));
  }

  @Test
  public void listReturnsCopies() {
    NotificationChannel created = store.create(channel("releases"), "1:token", "admin");

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
    store.create(first, "1:a", "admin");
    store.create(second, "1:b", "admin");

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
}
