package org.globalart.astrum.notifications.internal;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.concurrent.CopyOnWriteArrayList;
import java.util.concurrent.atomic.AtomicLong;

import org.globalart.astrum.common.app.BaseUrlManager;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.browse.node.BrowsePath;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.browse.BrowseNodeGenerator;
import org.globalart.astrum.repository.content.event.asset.AssetCreatedEvent;
import org.globalart.astrum.repository.content.event.asset.AssetUploadedEvent;
import org.globalart.astrum.repository.content.event.component.ComponentCreatedEvent;
import org.globalart.astrum.repository.content.event.component.ComponentDeletedEvent;
import org.globalart.astrum.repository.content.event.component.ComponentsPurgedAuditEvent;
import org.globalart.astrum.repository.rest.api.AssetXODescriptor;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.types.ProxyType;

import org.junit.After;
import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.containsString;
import static org.hamcrest.Matchers.empty;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.startsWith;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

public class NotificationDispatcherTest
{
  private final AtomicLong clock = new AtomicLong(1_000_000);

  private final List<String> sent = new CopyOnWriteArrayList<>();

  private final List<NotificationMessage> messages = new CopyOnWriteArrayList<>();

  private final List<NotificationChannel> channels = new ArrayList<>();

  private final Repository releases = repository("maven-releases", "maven2", new HostedType());

  private final Repository central = repository("maven-central", "maven2", new ProxyType());

  private final Repository docker = repository("docker-hosted", "docker", new HostedType());

  private NotificationDispatcher dispatcher;

  @Before
  public void setUp() throws Exception {
    NotificationChannelStore store = mock(NotificationChannelStore.class);
    when(store.enabled()).thenAnswer(invocation -> channels.stream().filter(NotificationChannel::isEnabled).toList());
    when(store.find(anyString())).thenAnswer(invocation -> channels.stream()
        .filter(channel -> channel.getId().equals(invocation.getArgument(0)))
        .findFirst());
    when(store.secrets(any())).thenReturn(NotificationSecrets.of("1:token"));

    BaseUrlManager baseUrlManager = mock(BaseUrlManager.class);
    when(baseUrlManager.getUrl()).thenReturn("https://repo.example.com");

    BrowseNodeGenerator maven = mock(BrowseNodeGenerator.class, "maven2");
    when(maven.computeComponentPaths(any())).thenAnswer(invocation -> {
      Component component = ((Asset) invocation.getArgument(0)).component().get();
      List<BrowsePath> paths = new ArrayList<>();
      for (String segment : component.namespace().split("\\.")) {
        paths.add(new BrowsePath(segment, segment));
      }
      paths.add(new BrowsePath(component.name(), component.name()));
      paths.add(new BrowsePath(component.version(), component.version()));
      return paths;
    });
    AssetXODescriptor dockerDescriptor = mock(AssetXODescriptor.class, "docker");
    when(dockerDescriptor.computeRegistryUrl(any())).thenReturn("repo.example.com:8085");

    NotificationTransport transport = (channel, secret, message) -> {
      messages.add(message);
      sent.add(channel.getName() + "|" + message.text());
    };

    dispatcher = new NotificationDispatcher(store, transport, baseUrlManager, List.of(maven), List.of(dockerDescriptor),
        new NotificationBuffer(10_000, 60_000, NotificationDispatcher.BUFFER_CAPACITY), clock::get, () -> "roman", 10);
    dispatcher.start();
  }

  @After
  public void tearDown() throws Exception {
    dispatcher.stop();
  }

  private static Repository repository(final String name, final String format, final Type type) {
    Repository repository = mock(Repository.class);
    when(repository.getName()).thenReturn(name);
    when(repository.getFormat()).thenReturn(new Format(format)
    {
    });
    when(repository.getType()).thenReturn(type);
    return repository;
  }

  private NotificationChannel channel(final String name, final String... events) {
    NotificationChannel channel = new NotificationChannel();
    channel.setId(name);
    channel.setName(name);
    channel.setType(NotificationChannel.TELEGRAM);
    channel.setChatId("-100");
    channel.setLanguage(NotificationChannel.ENGLISH);
    channel.setEvents(events.length == 0 ? List.of("published", "updated", "deleted") : List.of(events));
    channels.add(channel);
    return channel;
  }

  private static Component component(final String namespace, final String name, final String version) {
    Component component = mock(Component.class);
    when(component.namespace()).thenReturn(namespace);
    when(component.name()).thenReturn(name);
    when(component.version()).thenReturn(version);
    return component;
  }

  private void created(final Repository repository, final Component component) {
    ComponentCreatedEvent event = mock(ComponentCreatedEvent.class);
    when(event.getComponent()).thenReturn(component);
    when(event.getRepository()).thenReturn(Optional.of(repository));
    dispatcher.on(event);
  }

  private void deleted(final Repository repository, final Component component) {
    ComponentDeletedEvent event = mock(ComponentDeletedEvent.class);
    when(event.getComponent()).thenReturn(component);
    when(event.getRepository()).thenReturn(Optional.of(repository));
    dispatcher.on(event);
  }

  private void uploaded(final Repository repository, final Component component) {
    Asset asset = mock(Asset.class);
    when(asset.component()).thenReturn(Optional.of(component));
    AssetUploadedEvent event = mock(AssetUploadedEvent.class);
    when(event.getAsset()).thenReturn(asset);
    when(event.getRepository()).thenReturn(Optional.of(repository));
    dispatcher.on(event);
  }

  private void createdAsset(final Repository repository, final Component component) {
    Asset asset = mock(Asset.class);
    when(asset.component()).thenReturn(Optional.ofNullable(component));
    AssetCreatedEvent event = mock(AssetCreatedEvent.class);
    when(event.getAsset()).thenReturn(asset);
    when(event.getRepository()).thenReturn(Optional.of(repository));
    dispatcher.on(event);
  }

  private void advance(final long millis) {
    clock.addAndGet(millis);
    dispatcher.flushReady();
  }

  private List<String> awaitSent(final int count) throws InterruptedException {
    long deadline = System.currentTimeMillis() + 10_000;
    while (sent.size() < count && System.currentTimeMillis() < deadline) {
      Thread.sleep(10);
    }
    Thread.sleep(150);
    return sent;
  }

  @Test
  public void aMavenDeploymentBecomesOneMessage() throws Exception {
    channel("releases");
    Component demo = component("org.example", "demo", "1.0.0");

    createdAsset(releases, null);
    created(releases, demo);
    for (int i = 0; i < 6; i++) {
      createdAsset(releases, demo);
      uploaded(releases, demo);
      advance(1_000);
    }
    assertThat(sent, is(empty()));

    advance(10_000);

    List<String> messages = awaitSent(1);
    assertThat(messages, hasSize(1));
    assertThat(messages.get(0), startsWith("releases|📦 <b>Component published</b>\n<b>org.example:demo:1.0.0</b>"));
    assertThat(messages.get(0), containsString("By: roman"));
    assertThat(messages.get(0),
        containsString("https://repo.example.com/#browse/browse:maven-releases:org%2Fexample%2Fdemo%2F1.0.0"));
  }

  @Test
  public void filesStoredIntoAnExistingComponentAreAnUpdate() throws Exception {
    channel("releases");

    uploaded(releases, component("org.example", "demo", "1.0.0"));
    advance(10_000);

    assertThat(awaitSent(1).get(0), containsString("🔄 <b>Component updated</b>"));
  }

  @Test
  public void manyComponentsInOneWindowBecomeOneSummary() throws Exception {
    channel("releases");

    for (int i = 0; i < 7; i++) {
      created(releases, component("org.example", "lib-" + i, "2.0"));
    }
    advance(10_000);

    List<String> messages = awaitSent(1);
    assertThat(messages, hasSize(1));
    assertThat(messages.get(0), startsWith("releases|📋 <b>Component changes: 7</b>"));
  }

  @Test
  public void channelsWithoutRepositoriesFollowHostedRepositoriesOnly() throws Exception {
    channel("all-hosted");
    channel("central-only").setRepositories(List.of("maven-central"));

    created(central, component("org.apache", "commons", "1.0"));
    created(releases, component("org.example", "demo", "1.0"));
    advance(10_000);

    List<String> messages = awaitSent(2);
    assertThat(messages, hasSize(2));
    assertThat(messages.stream().filter(m -> m.startsWith("all-hosted|")).toList().get(0),
        containsString("org.example:demo:1.0"));
    assertThat(messages.stream().filter(m -> m.startsWith("central-only|")).toList().get(0),
        containsString("org.apache:commons:1.0"));
  }

  @Test
  public void channelsOnlyGetTheEventsTheyChose() throws Exception {
    channel("deletions", "deleted");
    channel("disabled").setEnabled(false);

    created(releases, component("org.example", "new", "1.0"));
    deleted(releases, component("org.example", "old", "0.9"));
    advance(10_000);

    List<String> messages = awaitSent(1);
    assertThat(messages, hasSize(1));
    assertThat(messages.get(0), startsWith("deletions|🗑️ <b>Component deleted</b>\n<b>org.example:old:0.9</b>"));
  }

  @Test
  public void dockerImagesCarryThePullCommand() throws Exception {
    channel("images");

    created(docker, component("", "team/app", "1.4"));
    advance(10_000);

    assertThat(awaitSent(1).get(0), containsString("<code>docker pull repo.example.com:8085/team/app:1.4</code>"));
  }

  @Test
  public void purgedComponentsAreDeleted() throws Exception {
    channel("releases");
    List<Component> purged = List.of(component("g", "a", "1"), component("g", "b", "1"));
    ComponentsPurgedAuditEvent event = mock(ComponentsPurgedAuditEvent.class);
    when(event.getComponents()).thenReturn(purged);
    when(event.getRepository()).thenReturn(Optional.of(releases));

    dispatcher.on(event);
    advance(10_000);

    List<String> messages = awaitSent(2);
    assertThat(messages, hasSize(2));
    assertThat(messages.get(0), containsString("Component deleted"));
  }

  @Test
  public void pendingChangesAreSentOnStop() throws Exception {
    channel("releases");

    created(releases, component("org.example", "demo", "1.0"));
    dispatcher.stop();

    assertThat(sent, hasSize(1));
    dispatcher.start();
  }

  @Test
  public void jsonWebhooksGetEveryComponentChangeOnItsOwn() throws Exception {
    NotificationChannel webhook = channel("ci");
    webhook.setType(NotificationChannel.WEBHOOK);
    webhook.setChatId(null);
    Component demo = component("org.example", "demo", "1.0.0");

    created(releases, demo);
    uploaded(releases, demo);
    for (int i = 0; i < 6; i++) {
      created(releases, component("org.example", "lib-" + i, "1.0"));
    }
    advance(10_000);

    assertThat(awaitSent(7), hasSize(7));
    assertThat(messages.stream().map(NotificationMessage::event).distinct().toList(),
        contains("component.published"));
    NotificationMessage demoMessage = messages.stream()
        .filter(message -> message.notice().name().equals("demo"))
        .findFirst()
        .orElseThrow();
    assertThat(demoMessage.notice().repository(), is("maven-releases"));
    assertThat(demoMessage.notice().repositoryType(), is("hosted"));
    assertThat(demoMessage.notice().initiator(), is("roman"));
    assertThat(demoMessage.text(), startsWith("📦 *Component published*\n`org.example:demo:1.0.0`"));
  }
}
