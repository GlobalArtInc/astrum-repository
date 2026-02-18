
package org.globalart.astrum.repository.move;

import java.time.OffsetDateTime;

/**
 * @since 3.28
 */
public interface ChangeRepositoryBlobStoreConfiguration
{
  String getName();

  void setName(final String name);

  String getTargetBlobStoreName();

  void setTargetBlobStoreName(final String targetBlobStoreName);

  String getSourceBlobStoreName();

  void setSourceBlobStoreName(final String sourceBlobStoreName);

  OffsetDateTime getStarted();

  void setStarted(final OffsetDateTime processStartDate);
}
