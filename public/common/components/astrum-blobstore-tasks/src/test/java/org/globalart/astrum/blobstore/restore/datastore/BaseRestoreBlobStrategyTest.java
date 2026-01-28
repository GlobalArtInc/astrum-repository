
package org.globalart.astrum.blobstore.restore.datastore;

import java.io.IOException;
import java.io.UncheckedIOException;
import java.util.HashMap;
import java.util.Map;
import java.util.Properties;

import javax.annotation.Nonnull;

import org.globalart.astrum.blobstore.api.Blob;
import org.globalart.astrum.blobstore.api.BlobAttributes;
import org.globalart.astrum.blobstore.api.BlobId;
import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.common.log.DryRunPrefix;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentAssetBuilder;
import org.globalart.astrum.repository.content.fluent.FluentAssets;
import org.globalart.astrum.repository.content.handlers.LastDownloadedAttributeHandler;
import org.globalart.astrum.repository.manager.RepositoryManager;

import com.google.common.base.Throwables;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.instanceOf;
import static org.hamcrest.Matchers.is;
import static org.junit.Assert.assertThrows;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.blobstore.api.BlobAttributesConstants.HEADER_PREFIX;
import static org.globalart.astrum.blobstore.api.BlobStore.BLOB_NAME_HEADER;
import static org.globalart.astrum.blobstore.api.BlobStore.REPO_NAME_HEADER;
import static org.globalart.astrum.repository.config.ConfigurationConstants.BLOB_STORE_NAME;
import static org.globalart.astrum.repository.config.ConfigurationConstants.STORAGE;

@ExtendWith(MockitoExtension.class)
class BaseRestoreBlobStrategyTest

{
  private static final String TEST_BLOB_STORE_NAME = "test-blobstore";

  private static final String REPO_NAME = "test-repo";

  private static final String BLOB_NAME = "/test/blob.jar";

  private static final String ASSET_PATH = "/test/blob.jar";

  @Mock
  private DryRunPrefix dryRunPrefix;

  @Mock
  private LastDownloadedAttributeHandler lastDownloadedAttributeHandler;

  @Mock
  private Blob blob;

  @Mock
  private BlobStore blobStore;

  @Mock
  private BlobStoreConfiguration blobStoreConfiguration;

  @Mock
  private Repository repository;

  @Mock
  private RepositoryManager repositoryManager;

  @Mock
  private ContentFacet contentFacet;

  @Mock
  private BlobAttributes blobAttributes;

  @Mock
  private BlobId blobId;

  @Mock
  private Configuration repositoryConfiguration;

  private Properties properties;

  private TestRestoreBlobStrategy underTest;

  @BeforeEach
  void setup() {
    underTest = new TestRestoreBlobStrategy(dryRunPrefix);
    underTest.injectDependencies(lastDownloadedAttributeHandler);

    properties = new Properties();
    properties.setProperty(HEADER_PREFIX + REPO_NAME_HEADER, REPO_NAME);
    properties.setProperty(HEADER_PREFIX + BLOB_NAME_HEADER, BLOB_NAME);

    when(blobStore.getBlobStoreConfiguration()).thenReturn(blobStoreConfiguration);
    when(blobStoreConfiguration.getName()).thenReturn(TEST_BLOB_STORE_NAME);
    when(blob.getId()).thenReturn(blobId);
    when(repository.getName()).thenReturn(REPO_NAME);
    when(repository.getConfiguration()).thenReturn(repositoryConfiguration);
    Map<String, Map<String, Object>> attributes = new HashMap<>();
    Map<String, Object> storageAttributes = new HashMap<>();
    storageAttributes.put(BLOB_STORE_NAME, TEST_BLOB_STORE_NAME);
    attributes.put(STORAGE, storageAttributes);
    when(repositoryConfiguration.getAttributes()).thenReturn(attributes);
    when(repositoryManager.get(REPO_NAME)).thenReturn(repository);
    when(blobStore.getBlobAttributes(blobId)).thenReturn(blobAttributes);
    when(blobAttributes.isDeleted()).thenReturn(false);
    when(lastDownloadedAttributeHandler.readLastDownloadedAttribute(anyString(), any(Blob.class))).thenReturn(null);
  }

  @Test
  void testRestore_propagatesIOException() {
    FluentAssets fluentAssets = mock(FluentAssets.class);
    FluentAssetBuilder fluentAssetBuilder = mock();

    when(repository.facet(ContentFacet.class)).thenReturn(contentFacet);
    when(contentFacet.assets()).thenReturn(fluentAssets);
    when(fluentAssets.path(ASSET_PATH)).thenReturn(fluentAssetBuilder);

    underTest.exception = new IOException("Test IO exception");

    UncheckedIOException exception = assertThrows(
        UncheckedIOException.class,
        () -> underTest.restore(properties, blob, blobStore, false));

    assertThat(exception.getCause(), instanceOf(IOException.class));
    assertThat(exception.getCause().getMessage(), is("Test IO exception"));
  }

  @Test
  void testRestore_propagatesNPE() {
    FluentAssets fluentAssets = mock(FluentAssets.class);
    FluentAssetBuilder fluentAssetBuilder = mock();

    when(repository.facet(ContentFacet.class)).thenReturn(contentFacet);
    when(contentFacet.assets()).thenReturn(fluentAssets);
    when(fluentAssets.path(ASSET_PATH)).thenReturn(fluentAssetBuilder);

    underTest.exception = new NullPointerException("NPE");

    NullPointerException exception =
        assertThrows(NullPointerException.class, () -> underTest.restore(properties, blob, blobStore, false));

    assertThat(exception.getMessage(), is("NPE"));
  }

  /**
   * Concrete test implementation of BaseRestoreBlobStrategy for testing purposes
   */
  private class TestRestoreBlobStrategy
      extends BaseRestoreBlobStrategy<DataStoreRestoreBlobData>
  {
    Exception exception;

    protected TestRestoreBlobStrategy(final DryRunPrefix dryRunPrefix) {
      super(dryRunPrefix);
    }

    @Override
    protected boolean canAttemptRestore(@Nonnull final DataStoreRestoreBlobData data) {
      return true;
    }

    @Override
    protected void createAssetFromData(final DataStoreRestoreBlobData data) throws IOException {
      Throwables.propagateIfPossible(exception, IOException.class);
    }

    @Override
    protected String getAssetPath(@Nonnull final DataStoreRestoreBlobData data) {
      return ASSET_PATH;
    }

    @Override
    protected DataStoreRestoreBlobData createRestoreData(
        final Properties properties,
        final Blob blob,
        final BlobStore blobStore)
    {
      return new DataStoreRestoreBlobData(blob, properties, blobStore, repositoryManager);
    }

    @Override
    protected boolean isComponentRequired(final DataStoreRestoreBlobData data) {
      return false;
    }

    @Override
    public void after(final boolean updateAssets, final Repository repository) {

    }
  }
}
