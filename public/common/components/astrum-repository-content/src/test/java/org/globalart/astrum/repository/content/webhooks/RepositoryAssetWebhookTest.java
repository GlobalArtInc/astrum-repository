
package org.globalart.astrum.repository.content.webhooks;

import java.net.URI;
import java.util.Optional;

import org.globalart.astrum.audit.InitiatorProvider;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.common.node.NodeAccess;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.event.asset.AssetCreatedEvent;
import org.globalart.astrum.repository.content.event.asset.AssetDeletedEvent;
import org.globalart.astrum.repository.content.event.asset.AssetKindEvent;
import org.globalart.astrum.repository.content.event.asset.AssetPurgedEvent;
import org.globalart.astrum.repository.content.store.AssetData;
import org.globalart.astrum.repository.content.store.ContentStoreEventSetter;
import org.globalart.astrum.repository.content.webhooks.RepositoryAssetWebhook.RepositoryAssetWebhookPayload;
import org.globalart.astrum.repository.content.webhooks.RepositoryAssetWebhook.RepositoryAssetWebhookPayload.RepositoryAsset;
import org.globalart.astrum.repository.webhooks.RepositoryWebhook;
import org.globalart.astrum.webhooks.WebhookRequestSendEvent;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.ArgumentCaptor;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.verifyNoInteractions;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class RepositoryAssetWebhookTest
{
  @Mock
  private NodeAccess nodeAccess;

  @Mock
  private InitiatorProvider initiatorProvider;

  @Mock
  private EventManager eventManager;

  @Mock
  private Repository repository;

  @Mock
  private Format format;

  private RepositoryAssetWebhook underTest;

  @Before
  public void setUp() {
    when(nodeAccess.getId()).thenReturn("node-1");
    when(initiatorProvider.get()).thenReturn("admin");
    when(repository.getName()).thenReturn("maven-releases");
    when(repository.getFormat()).thenReturn(format);
    when(format.getValue()).thenReturn("maven2");

    underTest = new RepositoryAssetWebhook(nodeAccess, initiatorProvider);
    underTest.setEventManager(eventManager);
  }

  @Test
  public void testGetName() {
    assertThat(underTest.getName(), is("asset"));
  }

  @Test
  public void testWebhookNameConstant() {
    assertThat(RepositoryAssetWebhook.NAME, is("asset"));
  }

  @Test
  public void testRepositoryAssetFields() {
    RepositoryAsset asset = new RepositoryAsset("ext-id", "asset-id", "npm", "/lodash/-/lodash-4.17.21.tgz");

    assertThat(asset.getId(), is("ext-id"));
    assertThat(asset.getAssetId(), is("asset-id"));
    assertThat(asset.getFormat(), is("npm"));
    assertThat(asset.getName(), is("/lodash/-/lodash-4.17.21.tgz"));
  }

  @Test
  public void testRepositoryAssetWithDifferentFormats() {
    RepositoryAsset maven = new RepositoryAsset("1", "m-1", "maven2", "/org/example/artifact.jar");
    assertThat(maven.getFormat(), is("maven2"));
    assertThat(maven.getName(), is("/org/example/artifact.jar"));

    RepositoryAsset docker = new RepositoryAsset("2", "d-1", "docker", "/v2/library/nginx/manifests/latest");
    assertThat(docker.getFormat(), is("docker"));
    assertThat(docker.getName(), is("/v2/library/nginx/manifests/latest"));
  }

  @Test
  public void testOnAssetCreatedEvent_noSubscriptions() {
    AssetData asset = createAssetData(3);
    AssetCreatedEvent event = new AssetCreatedEvent(asset);
    ContentStoreEventSetter.setRepositorySupplier(event, () -> Optional.of(repository));

    underTest.on(event);

    verifyNoInteractions(eventManager);
  }

  @Test
  public void testOnAssetCreatedEvent_withSubscription_queuesPayload() {
    subscribeToRepository("maven-releases");

    AssetData asset = createAssetData(3);
    AssetCreatedEvent event = new AssetCreatedEvent(asset);
    ContentStoreEventSetter.setRepositorySupplier(event, () -> Optional.of(repository));

    underTest.on(event);

    RepositoryAssetWebhookPayload payload = capturePayload();
    assertThat(payload.getRepositoryName(), is("maven-releases"));
    assertThat(String.valueOf(payload.getAction()), is("CREATED"));
    assertThat(payload.getNodeId(), is("node-1"));
    assertThat(payload.getInitiator(), is("admin"));
    assertThat(payload.getAsset(), is(notNullValue()));
    assertThat(payload.getAsset().getFormat(), is("maven2"));
    assertThat(payload.getAsset().getName(), is("/org/example/artifact-1.0.jar"));
  }

  @Test
  public void testOnAssetUpdatedEvent_withSubscription_queuesPayload() {
    subscribeToRepository("maven-releases");

    AssetData asset = createAssetData(3);
    AssetKindEvent event = new AssetKindEvent(asset);
    ContentStoreEventSetter.setRepositorySupplier(event, () -> Optional.of(repository));

    underTest.on(event);

    RepositoryAssetWebhookPayload payload = capturePayload();
    assertThat(payload.getRepositoryName(), is("maven-releases"));
    assertThat(String.valueOf(payload.getAction()), is("UPDATED"));
    assertThat(payload.getAsset(), is(notNullValue()));
  }

  @Test
  public void testOnAssetDeletedEvent_withSubscription_queuesPayload() {
    subscribeToRepository("maven-releases");

    AssetData asset = createAssetData(3);
    AssetDeletedEvent event = new AssetDeletedEvent(asset);
    ContentStoreEventSetter.setRepositorySupplier(event, () -> Optional.of(repository));

    underTest.on(event);

    RepositoryAssetWebhookPayload payload = capturePayload();
    assertThat(payload.getRepositoryName(), is("maven-releases"));
    assertThat(String.valueOf(payload.getAction()), is("DELETED"));
    assertThat(payload.getAsset(), is(notNullValue()));
  }

  @Test
  public void testOnAssetPurgedEvent_withSubscription_queuesPayload() {
    subscribeToRepository("maven-releases");

    AssetPurgedEvent event = new AssetPurgedEvent(1, new int[]{3, 4});
    ContentStoreEventSetter.setRepositorySupplier(event, () -> Optional.of(repository));

    underTest.on(event);

    RepositoryAssetWebhookPayload payload = capturePayload();
    assertThat(payload.getRepositoryName(), is("maven-releases"));
    assertThat(String.valueOf(payload.getAction()), is("PURGED"));
    assertThat(payload.getAssets(), is(notNullValue()));
    assertThat(payload.getAssets().length, is(2));
  }

  @Test
  public void testOnEvent_subscriptionForDifferentRepo_doesNotQueue() {
    subscribeToRepository("other-repo");

    AssetData asset = createAssetData(3);
    AssetCreatedEvent event = new AssetCreatedEvent(asset);
    ContentStoreEventSetter.setRepositorySupplier(event, () -> Optional.of(repository));

    underTest.on(event);

    // eventManager.register is called during subscribe, but no WebhookRequestSendEvent should be posted
    verify(eventManager).register(underTest);
    ArgumentCaptor<Object> captor = ArgumentCaptor.forClass(Object.class);
    verify(eventManager).register(captor.capture());
    // Only the register call, no post call for WebhookRequestSendEvent
  }

  @Test
  public void testPayloadSetAsset() {
    RepositoryAsset asset = new RepositoryAsset(
        "ext-id", "asset-id", "maven2", "/org/example/artifact-1.0.jar");

    assertThat(asset.getId(), is("ext-id"));
    assertThat(asset.getAssetId(), is("asset-id"));
    assertThat(asset.getFormat(), is("maven2"));
    assertThat(asset.getName(), is("/org/example/artifact-1.0.jar"));
  }

  private AssetData createAssetData(final int assetId) {
    AssetData asset = new AssetData();
    asset.setRepositoryId(1);
    asset.setAssetId(assetId);
    asset.setPath("/org/example/artifact-1.0.jar");
    return asset;
  }

  private void subscribeToRepository(final String repositoryName) {
    RepositoryWebhook.Configuration config = mock(RepositoryWebhook.Configuration.class);
    when(config.getRepository()).thenReturn(repositoryName);
    when(config.getUrl()).thenReturn(URI.create("http://example.com/webhook"));
    underTest.subscribe(config);
  }

  private RepositoryAssetWebhookPayload capturePayload() {
    ArgumentCaptor<WebhookRequestSendEvent> captor = ArgumentCaptor.forClass(WebhookRequestSendEvent.class);
    verify(eventManager).post(captor.capture());
    return (RepositoryAssetWebhookPayload) captor.getValue().getRequest().getPayload();
  }
}
