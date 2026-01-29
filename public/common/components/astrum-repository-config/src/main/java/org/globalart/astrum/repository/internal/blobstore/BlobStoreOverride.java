
package org.globalart.astrum.repository.internal.blobstore;

/**
 * Allow the application of "overrides" to blob stores during initialization of {@link BaseBlobStoreManager}.
 *
 * @since 3.31
 */
public interface BlobStoreOverride
{
  void apply();
}
