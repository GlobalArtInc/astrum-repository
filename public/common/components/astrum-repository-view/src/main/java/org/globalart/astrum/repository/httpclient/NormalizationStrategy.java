
package org.globalart.astrum.repository.httpclient;

import org.apache.http.client.config.RequestConfig;

/**
 * Determines if the httpclient should normalize the URI
 * See {@link RequestConfig.Builder setNormalizeUri}
 *
 * @since 3.26
 */
public interface NormalizationStrategy
{
  boolean shouldNormalizeUri();
}
