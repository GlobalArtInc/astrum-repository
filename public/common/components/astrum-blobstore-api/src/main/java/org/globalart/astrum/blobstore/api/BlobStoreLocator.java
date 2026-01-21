
package org.globalart.astrum.blobstore.api;

/**
 * @since 3.34
 */
public interface BlobStoreLocator
{
  BlobStore getBlobStore(String repositoryName) throws RepositoryDoesNotExistException;
}
