
package org.globalart.astrum.coreui.internal.blobstore;

import java.util.Map;

import org.globalart.astrum.blobstore.quota.BlobStoreQuota;

public class BlobStoreQuotaTypesUIResponse
{
  private final String id;

  private final String name;

  public BlobStoreQuotaTypesUIResponse(final Map.Entry<String, BlobStoreQuota> entry) {
    this.id = entry.getKey();
    this.name = entry.getValue().getDisplayName();
  }

  public String getId() {
    return id;
  }

  public String getName() {
    return name;
  }
}
