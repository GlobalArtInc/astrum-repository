
package org.globalart.astrum.repository.internal.blobstore;

import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;

/**
 * {@link BlobStoreConfiguration} event.
 *
 * @since 3.1
 */
public interface BlobStoreConfigurationEvent
{
  boolean isLocal();

  String getName();

  BlobStoreConfiguration getConfiguration();
}
