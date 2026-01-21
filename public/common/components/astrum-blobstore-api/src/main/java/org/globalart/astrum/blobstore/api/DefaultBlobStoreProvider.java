
package org.globalart.astrum.blobstore.api;

import java.util.function.Supplier;

/**
 * Implementations of {@link BlobStoreManager} may choose to use one or more implementations of this interface
 * to create the initial blob stores.
 *
 * @since 3.37
 */
public interface DefaultBlobStoreProvider
{
  BlobStoreConfiguration get(Supplier<BlobStoreConfiguration> configurationSupplier);
}
