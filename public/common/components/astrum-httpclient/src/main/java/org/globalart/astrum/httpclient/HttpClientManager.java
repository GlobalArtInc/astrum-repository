
package org.globalart.astrum.httpclient;

import javax.annotation.Nullable;

import org.globalart.astrum.httpclient.HttpClientPlan.Customizer;
import org.globalart.astrum.httpclient.config.HttpClientConfiguration;

import org.apache.http.impl.client.CloseableHttpClient;
import org.apache.http.impl.client.HttpClientBuilder;

/**
 * HTTP-client manager.
 *
 * @since 3.0
 */
public interface HttpClientManager
{
  /**
   * Returns global HTTP-client configuration.
   */
  HttpClientConfiguration getConfiguration();

  /**
   * Installs new global HTTP-client configuration.
   */
  void setConfiguration(HttpClientConfiguration configuration);

  /**
   * Create customized HTTP-client.
   */
  CloseableHttpClient create(HttpClientPlan.Customizer customizer);

  /**
   * Create HTTP-client with defaults.
   *
   * Consider using {@code Provider<HttpClient>} instead.
   */
  CloseableHttpClient create();

  /**
   * Create customized HTTP-client builder.
   *
   * @deprecated method is only present to be able to implement http client to access CLM data and should be removed
   *             once the api will be changed to allow direct usage of an http client instance
   */
  @Deprecated
  HttpClientBuilder prepare(@Nullable final Customizer customizer);

  /**
   * Provides a new instance of {@link HttpClientConfiguration} applicable for use with this backing store
   */
  HttpClientConfiguration newConfiguration();
}
