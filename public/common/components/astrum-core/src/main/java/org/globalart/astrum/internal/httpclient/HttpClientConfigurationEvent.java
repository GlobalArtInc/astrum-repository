
package org.globalart.astrum.internal.httpclient;

import org.globalart.astrum.httpclient.config.HttpClientConfiguration;

/**
 * Event for {@link HttpClientConfiguration} entity.
 * 
 * @since 3.1
 */
public interface HttpClientConfigurationEvent
{
  boolean isLocal();

  String getRemoteNodeId();
}
