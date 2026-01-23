
package org.globalart.astrum.blobstore;

import java.util.Map;

/**
 * @since 3.30
 */
public interface ConnectionChecker
{
  boolean verifyConnection(final String blobStoreName, final Map<String, Map<String, Object>> blobStoreAttributes);
}
