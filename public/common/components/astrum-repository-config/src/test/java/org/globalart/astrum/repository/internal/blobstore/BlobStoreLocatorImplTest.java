
package org.globalart.astrum.repository.internal.blobstore;

import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.blobstore.api.BlobStoreManager;
import org.globalart.astrum.blobstore.api.RepositoryDoesNotExistException;
import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.manager.RepositoryManager;

import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.mockito.Mockito.when;
import static org.hamcrest.Matchers.is;
import static org.globalart.astrum.repository.config.ConfigurationConstants.BLOB_STORE_NAME;
import static org.globalart.astrum.repository.config.ConfigurationConstants.STORAGE;
import static org.hamcrest.MatcherAssert.assertThat;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class BlobStoreLocatorImplTest
{
  private BlobStoreLocatorImpl underTest;

  @Mock
  private RepositoryManager repositoryManager;

  @Mock
  private BlobStoreManager blobStoreManager;

  @Mock
  private Repository sourceRepository;

  @Mock
  private BlobStore sourceBlobStore;

  @Mock
  private BlobStoreConfiguration sourceBlobStoreConfiguration;

  @Mock
  private Configuration sourceRepositoryConfiguration;

  @Mock
  NestedAttributesMap sourceRepoStorageAttributes;

  @Before
  public void setup() throws RepositoryDoesNotExistException {
    when(blobStoreManager.get("blobStoreName")).thenReturn(sourceBlobStore);
    when(sourceRepository.getConfiguration()).thenReturn(sourceRepositoryConfiguration);
    when(repositoryManager.get("repoName")).thenReturn(sourceRepository);
    when(sourceRepoStorageAttributes.require(BLOB_STORE_NAME, String.class)).thenReturn("blobStoreName");
    when(sourceRepoStorageAttributes.get(BLOB_STORE_NAME, String.class)).thenReturn("blobStoreName");
    when(sourceRepositoryConfiguration.attributes(STORAGE)).thenReturn(sourceRepoStorageAttributes);
    underTest = new BlobStoreLocatorImpl(blobStoreManager, repositoryManager);
  }

  @Test
  public void getBlobStore() throws RepositoryDoesNotExistException {
    BlobStore blobStore = underTest.getBlobStore("repoName");
    assertThat(blobStore, is(sourceBlobStore));
  }

  @Test(expected = RepositoryDoesNotExistException.class)
  public void getBlobStoreRepositoryNotPresent() throws RepositoryDoesNotExistException {
    when(repositoryManager.get("repoName")).thenReturn(null);
    underTest.getBlobStore("repoName");
  }
}
