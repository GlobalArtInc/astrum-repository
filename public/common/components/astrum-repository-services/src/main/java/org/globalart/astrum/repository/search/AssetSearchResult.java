
package org.globalart.astrum.repository.search;

import java.util.Date;
import java.util.Map;

/**
 * Result of an Asset search
 *
 * @since 3.38
 */
public interface AssetSearchResult
{
  Map<String, Object> getAttributes();

  Date getCreated();

  String getBlobRef();

  Date getBinaryUpdated();

  Map<String, String> getChecksum();

  String getContentType();

  Long getFileSize();

  String getFormat();

  String getId();

  Date getLastDownloaded();

  Date getLastModified();

  Date getLastVerified();

  String getPath();

  String getRepository();

  String getUploader();

  String getUploaderIp();
}
