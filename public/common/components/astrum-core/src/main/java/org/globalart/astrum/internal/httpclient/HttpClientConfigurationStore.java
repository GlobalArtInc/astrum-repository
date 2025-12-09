
package org.globalart.astrum.internal.httpclient;

import javax.annotation.Nullable;

import org.globalart.astrum.httpclient.config.HttpClientConfiguration;

/**
 * {@link HttpClientConfiguration} store.
 *
 * @since 3.0
 */
public interface HttpClientConfigurationStore
{
  @Nullable
  HttpClientConfiguration load();

  void save(HttpClientConfiguration configuration);

  /**
   * @since 3.20
   */
  HttpClientConfiguration newConfiguration();
}
