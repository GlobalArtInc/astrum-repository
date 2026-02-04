
package org.globalart.astrum.repository.content;

import java.time.OffsetDateTime;
import java.util.Map;

/**
 * Information about asset and it's blob.
 *
 * @since 3.41
 */
public interface AssetInfo
    extends RepositoryContent
{
  Integer assetId();

  Integer componentId();

  String path();

  String contentType();

  String createdBy();

  String createdByIp();

  OffsetDateTime lastUpdated();

  OffsetDateTime lastDownloaded();

  Long blobSize();

  Map<String, String> checksums();

  OffsetDateTime blobCreated();

  OffsetDateTime addedToRepository();
}
