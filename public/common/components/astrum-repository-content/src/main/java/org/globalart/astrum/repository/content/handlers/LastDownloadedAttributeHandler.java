
package org.globalart.astrum.repository.content.handlers;

import java.time.OffsetDateTime;

import org.globalart.astrum.blobstore.api.Blob;
import org.globalart.astrum.repository.content.fluent.FluentAsset;

/**
 * Store and read asset's lastDownloaded attribute in blob's property.
 */
public interface LastDownloadedAttributeHandler
{
  /**
   * Write asset lastDownloaded attribute to the blob's property file.
   */
  void writeLastDownloadedAttribute(FluentAsset asset);

  /**
   * Read lastDownload asset's attribute from blob's property file.
   */
  OffsetDateTime readLastDownloadedAttribute(String blobstore, Blob blob);
}
