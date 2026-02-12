
package org.globalart.astrum.repository.content.store;

import java.security.SecureRandom;
import java.util.Optional;
import java.util.OptionalInt;
import java.util.function.IntSupplier;

import org.globalart.astrum.common.entity.EntityId;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.AssetBlob;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.ContentRepository;
import org.globalart.astrum.repository.content.facet.ContentFacetDependencies;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.fluent.internal.FluentAssetImpl;
import org.globalart.astrum.repository.content.fluent.internal.FluentComponentImpl;
import org.globalart.astrum.repository.move.RepositoryMoveService;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.Mockito;
import org.mockito.junit.MockitoJUnitRunner;

import static java.util.OptionalInt.empty;
import static org.hamcrest.Matchers.is;
import static org.junit.Assert.assertFalse;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.junit.Assert.fail;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.repository.content.store.InternalIds.internalAssetBlobId;
import static org.globalart.astrum.repository.content.store.InternalIds.internalAssetId;
import static org.globalart.astrum.repository.content.store.InternalIds.internalComponentId;
import static org.globalart.astrum.repository.content.store.InternalIds.contentRepositoryId;

/**
 * Test {@link InternalIds}.
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class InternalIdsTest
{
  @Mock
  private ContentFacetSupport contentFacet;

  @Mock
  private ContentFacetDependencies dependencies;

  @Before
  public void setUp() {
    when(contentFacet.dependencies()).thenReturn(dependencies);
    when(dependencies.getMoveService()).thenReturn(Optional.of(mock(RepositoryMoveService.class)));
  }

  @Test
  public void testMissingInternalIds() {
    ContentRepository repository = new ContentRepositoryData();
    Component component = new ComponentData();
    Asset asset = new AssetData();
    AssetBlob assetBlob = new AssetBlobData();

    Component fluentComponent = new FluentComponentImpl(contentFacet, component);
    Asset fluentAsset = new FluentAssetImpl(contentFacet, asset);

    checkIllegalState(() -> contentRepositoryId(repository));
    checkIllegalState(() -> contentRepositoryId(component));
    checkIllegalState(() -> contentRepositoryId(fluentComponent));
    checkIllegalState(() -> internalComponentId(component));
    checkIllegalState(() -> internalComponentId(fluentComponent));
    checkIllegalState(() -> contentRepositoryId(asset));
    checkIllegalState(() -> contentRepositoryId(fluentAsset));
    checkIllegalState(() -> internalAssetId(asset));
    checkIllegalState(() -> internalAssetId(fluentAsset));
    checkIllegalState(() -> internalAssetBlobId(assetBlob));

    checkNotPresent(internalComponentId(asset));
    checkNotPresent(internalComponentId(fluentAsset));
    checkNotPresent(internalAssetBlobId(asset));
    checkNotPresent(internalAssetBlobId(fluentAsset));
  }

  @Test
  public void testInternalIds() {
    ContentRepositoryData repository = new ContentRepositoryData();
    repository.repositoryId = 1;
    ComponentData component = new ComponentData();
    component.componentId = 2;
    AssetData asset = new AssetData();
    asset.assetId = 3;
    AssetBlobData assetBlob = new AssetBlobData();
    assetBlob.assetBlobId = 4;

    component.repositoryId = 5;
    asset.repositoryId = 6;
    asset.componentId = 7;
    asset.assetBlobId = 8;

    Component fluentComponent = new FluentComponentImpl(contentFacet, component);
    Asset fluentAsset = new FluentAssetImpl(contentFacet, asset);

    assertThat(contentRepositoryId(repository), is(1));
    assertThat(contentRepositoryId(component), is(5));
    assertThat(contentRepositoryId(fluentComponent), is(5));
    assertThat(internalComponentId(component), is(2));
    assertThat(internalComponentId(fluentComponent), is(2));
    assertThat(contentRepositoryId(asset), is(6));
    assertThat(contentRepositoryId(fluentAsset), is(6));
    assertThat(internalAssetId(asset), is(3));
    assertThat(internalAssetId(fluentAsset), is(3));
    assertThat(internalAssetBlobId(assetBlob), is(4));

    assertThat(internalComponentId(asset).getAsInt(), is(7));
    assertThat(internalComponentId(fluentAsset).getAsInt(), is(7));
    assertThat(internalAssetBlobId(asset).getAsInt(), is(8));
    assertThat(internalAssetBlobId(fluentAsset).getAsInt(), is(8));
  }

  @Test
  public void shouldFetchComponentIdFromComponentWhenComponentIdIsNullOnAsset() {
    ComponentData component = new ComponentData();
    component.componentId = 2;
    AssetData asset = new AssetData();
    asset.assetId = 3;

    Asset fluentAsset = new FluentAssetImpl(contentFacet, asset);

    assertThat(internalComponentId(asset), is(empty()));
    assertThat(internalComponentId(fluentAsset), is(empty()));

    asset.setComponent(component);
    asset.componentId = null;
    assertThat(internalComponentId(asset).getAsInt(), is(component.componentId));
    assertThat(internalComponentId(fluentAsset).getAsInt(), is(component.componentId));
  }

  @Test
  public void shouldUnwrapFluentWrappersToTheirBackingData() {
    ComponentData component = new ComponentData();
    AssetData asset = new AssetData();

    Asset fluentAsset = new FluentAssetImpl(contentFacet, asset);
    Component fluentComponent = new FluentComponentImpl(contentFacet, component);

    assertThat(InternalIds.unwrap(fluentAsset), is(asset));
    assertThat(InternalIds.unwrap(fluentComponent), is(component));
    assertThat(InternalIds.unwrap(asset), is(asset));
  }

  private static void checkIllegalState(final IntSupplier underTest) {
    try {
      underTest.getAsInt();
      fail("Expected IllegalStateException");
    }
    catch (IllegalStateException e) {
      assertThat(e.getMessage(), is("Entity does not have an internal id; is it detached?"));
    }
  }

  private static void checkNotPresent(final OptionalInt underTest) {
    assertFalse(underTest.isPresent());
  }

  @Test
  public void testExternalIdRoundTrip() {
    SecureRandom random = new SecureRandom();
    for (int numIds = 0; numIds < 1_000_000; numIds++) {
      assertExternalIdRoundTrip(1 + random.nextInt(Integer.MAX_VALUE));
    }
  }

  @Test
  public void shouldReturnParsedIdWhenNumberFormatException() {
    EntityId externalId = Mockito.mock(EntityId.class);
    when(externalId.getValue()).thenReturn("103562827");
    int internalId = InternalIds.toInternalId(externalId);
    assertThat(internalId, is(103562827));
  }

  @Test
  public void shouldReturnValidIdWhenBase16ExternalId() {
    EntityId externalId = Mockito.mock(EntityId.class);
    when(externalId.getValue()).thenReturn("88d9dabf");
    int internalId = InternalIds.toInternalId(externalId);
    assertThat(internalId, is(103562827));
  }

  private void assertExternalIdRoundTrip(int expected) {
    EntityId externalId = InternalIds.toExternalId(expected);
    int internalId = InternalIds.toInternalId(externalId);
    assertThat(internalId, is(expected));
  }
}
