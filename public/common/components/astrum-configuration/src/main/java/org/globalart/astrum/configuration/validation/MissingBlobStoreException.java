
package org.globalart.astrum.configuration.validation;

/**
 * Thrown when a repository configuration references a blob store that does not exist during import.
 * This exception is propagated to stop the entire configuration import process.
 */
public class MissingBlobStoreException
    extends RuntimeException
{
  private final String repositoryName;

  private final String blobStoreName;

  public MissingBlobStoreException(final String repositoryName, final String blobStoreName) {
    super(String.format(
        "Cannot import repository '%s': Blob store '%s' does not exist. " +
            "Blob stores must be created manually before importing repositories.",
        repositoryName, blobStoreName));
    this.repositoryName = repositoryName;
    this.blobStoreName = blobStoreName;
  }

  public String getRepositoryName() {
    return repositoryName;
  }

  public String getBlobStoreName() {
    return blobStoreName;
  }
}
