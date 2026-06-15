
package org.globalart.astrum.coreui.internal.blobstore;

import javax.annotation.Nullable;

import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.blobstore.api.BlobStoreMetrics;

import static com.google.common.base.Preconditions.checkNotNull;

public class BlobStoreUIResponse
{
  private final String name;

  private final String typeId;

  private final String typeName;

  private final String path;

  private final boolean unavailable;

  private final long blobCount;

  private final long totalSizeInBytes;

  private final long availableSpaceInBytes;

  private final boolean unlimited;

  public BlobStoreUIResponse(
      final String typeId,
      final BlobStoreConfiguration configuration,
      @Nullable final BlobStoreMetrics metrics,
      final String path)
  {
    if (metrics != null) {
      unavailable = metrics.isUnavailable();
      blobCount = metrics.getBlobCount();
      totalSizeInBytes = metrics.getTotalSize();
      availableSpaceInBytes = metrics.getAvailableSpace();
      unlimited = metrics.isUnlimited();
    }
    else {
      unavailable = true;
      blobCount = 0;
      totalSizeInBytes = 0;
      availableSpaceInBytes = 0;
      unlimited = false;
    }

    name = checkNotNull(configuration.getName());

    this.typeId = checkNotNull(typeId);
    typeName = configuration.getType();
    this.path = checkNotNull(path);
  }

  public String getName() {
    return name;
  }

  public String getTypeId() {
    return typeId;
  }

  public String getTypeName() {
    return typeName;
  }

  public String getPath() {
    return path;
  }

  public boolean isUnavailable() {
    return unavailable;
  }

  public long getBlobCount() {
    return blobCount;
  }

  public long getTotalSizeInBytes() {
    return totalSizeInBytes;
  }

  public long getAvailableSpaceInBytes() {
    return availableSpaceInBytes;
  }

  public boolean isUnlimited() {
    return unlimited;
  }
}
