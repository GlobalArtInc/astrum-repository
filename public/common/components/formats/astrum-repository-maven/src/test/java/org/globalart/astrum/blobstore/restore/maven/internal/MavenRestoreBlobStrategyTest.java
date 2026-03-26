
package org.globalart.astrum.blobstore.restore.maven.internal;

import java.io.ByteArrayInputStream;
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
import org.globalart.astrum.content.maven.MavenContentFacet;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.content.AssetBlob;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentAssetBuilder;
import org.globalart.astrum.repository.content.fluent.FluentAssets;
import org.globalart.astrum.repository.content.fluent.FluentBlobs;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.handlers.LastDownloadedAttributeHandler;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.maven.MavenPath;
import org.globalart.astrum.repository.maven.MavenPath.Coordinates;
import org.globalart.astrum.repository.maven.MavenPathParser;
import org.globalart.astrum.repository.view.Payload;

import org.joda.time.DateTime;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static java.util.Optional.empty;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.ArgumentMatchers.nullable;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.verifyNoMoreInteractions;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.blobstore.api.BlobAttributesConstants.HEADER_PREFIX;
import static org.globalart.astrum.blobstore.api.BlobStore.BLOB_NAME_HEADER;
import static org.globalart.astrum.blobstore.api.BlobStore.CONTENT_TYPE_HEADER;
import static org.globalart.astrum.blobstore.api.BlobStore.REPO_NAME_HEADER;
import static org.globalart.astrum.repository.config.ConfigurationConstants.BLOB_STORE_NAME;
import static org.globalart.astrum.repository.config.ConfigurationConstants.STORAGE;

@RunWith(MockitoJUnitRunner.Silent.class)
public class MavenRestoreBlobStrategyTest

{
  private static final String REPO_NAME = "test-repo";

  private static final String TEST_BLOB_STORE_NAME = "test";

  private static final String BLOB_NAME = "/org/codehaus/plexus/plexus/3.1/plexus-3.1.pom";

  @Mock
  private MavenPathParser mavenPathParser;

  @Mock
  private RepositoryManager repositoryManager;

  @Mock
  private MavenPath mavenPath;

  @Mock
  private Coordinates coordinates;

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
  private Repository repository;

  @Mock
  private ContentFacet contentFacet;

  @Mock
  private MavenContentFacet mavenFacet;

  @Mock
  private DryRunPrefix dryRunPrefix;

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

  Properties properties;

  byte[] blobBytes = "blobbytes".getBytes();

  MavenRestoreBlobStrategy underTest;

  @Before
  public void setup() {
    properties = new Properties();
    properties.setProperty(HEADER_PREFIX + REPO_NAME_HEADER, REPO_NAME);
    properties.setProperty(HEADER_PREFIX + BLOB_NAME_HEADER, BLOB_NAME);
    properties.setProperty(HEADER_PREFIX + CONTENT_TYPE_HEADER, "application/xml");
    properties.setProperty("size", "1000");
    properties.setProperty("sha1", "b64de86ceaa4f0e4d8ccc44a26c562c6fb7fb230");

    when(repositoryManager.get(REPO_NAME)).thenReturn(repository);

    when(repository.getConfiguration()).thenReturn(repositoryConfiguration);
    Map<String, Map<String, Object>> attributes = new HashMap<>();
    Map<String, Object> storageAttributes = new HashMap<>();
    storageAttributes.put(BLOB_STORE_NAME, TEST_BLOB_STORE_NAME);
    attributes.put(STORAGE, storageAttributes);
    when(repositoryConfiguration.getAttributes()).thenReturn(attributes);

    when(contentFacet.assets()).thenReturn(assets);
    when(assets.path(nullable(String.class))).thenReturn(fluentAssetBuilder);
    when(fluentAssetBuilder.find()).thenReturn(Optional.of(asset));

    when(contentFacet.blobs()).thenReturn(assetBlobs);
    when(assetBlobs.blob(any(BlobRef.class))).thenReturn(Optional.of(blob));

    when(asset.component()).thenReturn(empty());
    when(asset.blob()).thenReturn(Optional.of(assetBlob));
    when(assetBlob.blobRef()).thenReturn(assetBlobRef);

    when(repository.facet(ContentFacet.class)).thenReturn(contentFacet);
    when(repository.optionalFacet(MavenContentFacet.class)).thenReturn(Optional.of(mavenFacet));
    when(repository.facet(MavenContentFacet.class)).thenReturn(mavenFacet);

    when(blob.getInputStream()).thenReturn(new ByteArrayInputStream(blobBytes));
    when(blob.getId()).thenReturn(blobId);
    when(blob.getMetrics()).thenReturn(blobMetrics);

    when(blobAttributes.isDeleted()).thenReturn(false);

    when(mavenPathParser.parsePath(BLOB_NAME)).thenReturn(mavenPath);
    when(mavenPathParser.isRepositoryMetadata(mavenPath)).thenReturn(false);
    when(mavenPathParser.isRepositoryIndex(mavenPath)).thenReturn(false);
    when(mavenPath.getCoordinates()).thenReturn(coordinates);

    when(blobStoreConfiguration.getName()).thenReturn(TEST_BLOB_STORE_NAME);

    when(blobStore.getBlobStoreConfiguration()).thenReturn(blobStoreConfiguration);
    when(blobStore.getBlobAttributes(blobId)).thenReturn(blobAttributes);

    underTest = new MavenRestoreBlobStrategy(dryRunPrefix, repositoryManager, mavenPathParser);
    underTest.injectDependencies(mock(LastDownloadedAttributeHandler.class));
  }

  @SuppressWarnings("deprecation")
  @Test
  public void testRestore() throws Exception {
    underTest.restore(properties, blob, blobStore);
    verify(mavenFacet).put(eq(mavenPath), any(Payload.class));
    verifyNoMoreInteractions(mavenFacet);
  }

  @Test
  public void testRestoreDryRun() throws Exception {
    underTest.restore(properties, blob, blobStore, true);
    verifyNoMoreInteractions(mavenFacet);
  }

  @SuppressWarnings("deprecation")
  @Test
  public void testRestoreSkipNotFacet() {
    when(repository.optionalFacet(MavenContentFacet.class)).thenReturn(Optional.empty());
    underTest.restore(properties, blob, blobStore);
    verifyNoMoreInteractions(mavenFacet);
  }

  @SuppressWarnings("deprecation")
  @Test
  public void testRestoreSkipExistingContent() throws Exception {
    when(asset.component()).thenReturn(Optional.of(component));
    underTest.restore(properties, blob, blobStore);
    verifyNoMoreInteractions(mavenFacet);
  }

  @Test
  public void missingCoordinates() throws Exception {
    when(mavenPath.getCoordinates()).thenReturn(null);
    underTest.restore(properties, blob, blobStore, false);
    verifyNoMoreInteractions(mavenFacet);
  }

  @Test
  public void pathIsIndex() throws Exception {
    when(mavenPathParser.isRepositoryIndex(mavenPath)).thenReturn(true);
    underTest.restore(properties, blob, blobStore, false);
    verifyNoMoreInteractions(mavenFacet);
  }

  @Test
  public void pathIsMetadata() throws Exception {
    when(mavenPathParser.isRepositoryMetadata(mavenPath)).thenReturn(true);
    underTest.restore(properties, blob, blobStore, false);
    verifyNoMoreInteractions(mavenFacet);
  }

  @Test
  public void testMissingContentFacet() throws Exception {
    when(repository.optionalFacet(MavenContentFacet.class)).thenReturn(Optional.empty());
    underTest.restore(properties, blob, blobStore, false);
    verifyNoMoreInteractions(mavenFacet);
  }

  @Test
  public void shouldSkipDeletedBlob() throws Exception {
    when(blobAttributes.isDeleted()).thenReturn(true);
    underTest.restore(properties, blob, blobStore, false);
    verifyNoMoreInteractions(mavenFacet);
  }

  @Test
  public void shouldSkipOlderBlob() throws Exception {
    when(asset.component()).thenReturn(Optional.of(component));
    when(assetBlob.blobCreated()).thenReturn(OffsetDateTime.now());
    when(blobMetrics.getCreationTime()).thenReturn(DateTime.now().minusDays(1));
    underTest.restore(properties, blob, blobStore, false);
    verifyNoMoreInteractions(mavenFacet);
  }

  @Test
  public void shouldRestoreMoreRecentBlob() throws Exception {
    when(asset.component()).thenReturn(Optional.of(component));
    when(assetBlob.blobCreated()).thenReturn(OffsetDateTime.now().minusDays(1));
    when(blobMetrics.getCreationTime()).thenReturn(DateTime.now());
    underTest.restore(properties, blob, blobStore, false);
    verify(mavenFacet).put(eq(mavenPath), any(Payload.class));
    verifyNoMoreInteractions(mavenFacet);
  }
}
