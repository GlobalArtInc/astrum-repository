
package org.globalart.astrum.blobstore.restore;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * @since 3.14
 */
public class RestoreBlobDataSupport
{
  private final RestoreBlobData blobData;

  public RestoreBlobDataSupport(final RestoreBlobData blobData) {
    this.blobData = checkNotNull(blobData);
  }

  public RestoreBlobData getBlobData() {
    return blobData;
  }
}
