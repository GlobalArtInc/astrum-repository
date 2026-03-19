
package org.globalart.astrum.blobstore.restore.raw.internal;

import java.io.IOException;
import java.time.OffsetDateTime;
import java.util.HashMap;
import java.util.Map;
import java.util.Optional;
import java.util.Properties;

import org.globalart.astrum.blobstore.api.Blob;
import org.globalart.astrum.blobstore.api.BlobAttributes;
import org.globalart.astrum.blobstore.api.BlobId;
import org.globalart.astrum.blobstore.api.BlobMetrics;
import org.globalart.astrum.blobstore.api.BlobRef;
import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.common.log.DryRunPrefix;
import org.globalart.astrum.content.raw.RawContentFacet;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.content.AssetBlob;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentBlobs;
import org.globalart.astrum.repository.content.fluent.FluentAssetBuilder;
import org.globalart.astrum.repository.content.fluent.FluentAssets;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.handlers.LastDownloadedAttributeHandler;
import org.globalart.astrum.repository.manager.RepositoryManager;

import org.joda.time.DateTime;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static java.util.Optional.empty;
import static java.util.Optional.of;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.verifyNoMoreInteractions;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.blobstore.api.BlobAttributesConstants.HEADER_PREFIX;
import static org.globalart.astrum.blobstore.api.BlobStore.BLOB_NAME_HEADER;
import static org.globalart.astrum.blobstore.api.BlobStore.CONTENT_TYPE_HEADER;
import static org.globalart.astrum.repository.config.ConfigurationConstants.BLOB_STORE_NAME;
import static org.globalart.astrum.repository.config.ConfigurationConstants.STORAGE;

@RunWith(MockitoJUnitRunner.Silent.class)
public class RawRestoreBlobStrategyTest

{
  private static final String TEST_BLOB_STORE_NAME = "test";

  private static final String REPOSITORY_NAME = "theRepository";

  private static final String BLOB_PATH = "/blob/path/end";

  private static final boolean DRY_RUN = true;

  @Mock
  private RawContentFacet rawContentFacet;

  @Mock
  private RepositoryManager repositoryManager;

  @Mock
  private Repository repository;

  @Mock
  private Blob blob;

  @Mock
  private BlobId blobId;

  @Mock
  private BlobAttributes blobAttributes;

  @Mock
  private AssetBlob assetBlob;

  @Mock
  private BlobRef assetBlobRef;

  @Mock
  private BlobMetrics blobMetrics;

  @Mock
  private BlobStore blobStore;

  @Mock
  private BlobStoreConfiguration blobStoreConfiguration;

  @Mock
  private FluentAssets assets;

  @Mock
  private FluentAssetBuilder fluentAssetBuilder;

  @Mock
  private FluentAsset asset;

  @Mock
  private FluentComponent component;

  @Mock
  private FluentBlobs assetBlobs;

  @Mock
  private Configuration repositoryConfiguration;

  private final DryRunPrefix dryRunPrefix = new DryRunPrefix("DRY RUN");

  private Properties properties;

  private RawRestoreBlobStrategy underTest;

  @Before
  public void setup() {
    when(repositoryManager.get(REPOSITORY_NAME)).thenReturn(repository);

    when(repository.getConfiguration()).thenReturn(repositoryConfiguration);
    Map<String, Map<String, Object>> attributes = new HashMap<>();
    Map<String, Object> storageAttributes = new HashMap<>();
    storageAttributes.put(BLOB_STORE_NAME, TEST_BLOB_STORE_NAME);
    attributes.put(STORAGE, storageAttributes);
    when(repositoryConfiguration.getAttributes()).thenReturn(attributes);

    when(repository.optionalFacet(RawContentFacet.class)).thenReturn(of(rawContentFacet));
    when(repository.facet(RawContentFacet.class)).thenReturn(rawContentFacet);
    when(repository.facet(ContentFacet.class)).thenReturn(rawContentFacet);

    when(rawContentFacet.assets()).thenReturn(assets);
    when(assets.path(anyString())).thenReturn(fluentAssetBuilder);
    when(fluentAssetBuilder.find()).thenReturn(Optional.of(asset));

    when(rawContentFacet.blobs()).thenReturn(assetBlobs);
    when(assetBlobs.blob(any(BlobRef.class))).thenReturn(Optional.of(blob));

    when(asset.component()).thenReturn(empty());
    when(asset.blob()).thenReturn(Optional.of(assetBlob));
    when(assetBlob.blobRef()).thenReturn(assetBlobRef);

    when(blob.getId()).thenReturn(blobId);
    when(blob.getMetrics()).thenReturn(blobMetrics);

    when(blobAttributes.isDeleted()).thenReturn(false);

    when(blobStore.getBlobStoreConfiguration()).thenReturn(blobStoreConfiguration);
    when(blobStore.getBlobAttributes(blobId)).thenReturn(blobAttributes);

    when(blobStoreConfiguration.getName()).thenReturn(TEST_BLOB_STORE_NAME);

    properties = new Properties();
    properties.put(HEADER_PREFIX + BlobStore.REPO_NAME_HEADER, REPOSITORY_NAME);
    properties.put(HEADER_PREFIX + BLOB_NAME_HEADER, BLOB_PATH);
    properties.put(HEADER_PREFIX + CONTENT_TYPE_HEADER, "testContentType");

    underTest = new RawRestoreBlobStrategy(dryRunPrefix, repositoryManager);
    underTest.injectDependencies(mock(LastDownloadedAttributeHandler.class));
  }

  @Test
  public void testRestoreWhenNoContentFacet() {
    when(repository.optionalFacet(RawContentFacet.class)).thenReturn(empty());
    when(repository.facet(RawContentFacet.class)).thenReturn(null);
    when(repository.facet(ContentFacet.class)).thenReturn(null);

    underTest.restore(properties, blob, blobStore, !DRY_RUN);
    verifyNoMoreInteractions(rawContentFacet);
  }

  @Test
  public void testRestoreWhenNoAssetExists() throws IOException {
    when(fluentAssetBuilder.find()).thenReturn(empty());

    underTest.restore(properties, blob, blobStore, !DRY_RUN);
    verify(asset, never()).delete();
    verify(rawContentFacet, times(1)).put(eq(BLOB_PATH), any());
  }

  @Test
  public void testRestoreWhenComponentAndAssetExist() throws IOException {
    when(asset.component()).thenReturn(of(component));

    underTest.restore(properties, blob, blobStore, !DRY_RUN);
    verify(asset, never()).delete();
    verify(rawContentFacet, never()).put(eq(BLOB_PATH), any());
  }

  @Test
  public void testRestoreWhenDryRun() throws IOException {
    underTest.restore(properties, blob, blobStore, DRY_RUN);
    verify(asset, never()).delete();
    verify(rawContentFacet, never()).put(anyString(), any());
  }

  @Test
  public void testRestoreCreatesAsset() throws Exception {
    underTest.restore(properties, blob, blobStore, !DRY_RUN);

    verify(asset, times(1)).delete();
    verify(rawContentFacet, times(1)).put(eq(BLOB_PATH), any());
  }

  @Test
  public void shouldSkipDeletedBlob() throws Exception {
    when(blobAttributes.isDeleted()).thenReturn(true);
    underTest.restore(properties, blob, blobStore, false);
    verifyNoMoreInteractions(rawContentFacet);
    verify(asset, never()).delete();
    verify(rawContentFacet, never()).put(eq(BLOB_PATH), any());
  }

  @Test
  public void shouldSkipOlderBlob() throws Exception {
    when(asset.component()).thenReturn(of(component));
    when(assetBlob.blobCreated()).thenReturn(OffsetDateTime.now());
    when(blobMetrics.getCreationTime()).thenReturn(DateTime.now().minusDays(1));
    underTest.restore(properties, blob, blobStore, false);
    verify(asset, never()).delete();
    verify(rawContentFacet, never()).put(eq(BLOB_PATH), any());
  }

  @Test
  public void shouldRestoreMoreRecentBlob() throws Exception {
    when(asset.component()).thenReturn(of(component));
    when(assetBlob.blobCreated()).thenReturn(OffsetDateTime.now().minusDays(1));
    when(blobMetrics.getCreationTime()).thenReturn(DateTime.now());
    underTest.restore(properties, blob, blobStore, false);
    verify(asset, times(1)).delete();
    verify(rawContentFacet, times(1)).put(eq(BLOB_PATH), any());
  }
}
