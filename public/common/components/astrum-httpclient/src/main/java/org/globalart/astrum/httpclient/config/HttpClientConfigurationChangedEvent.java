

package org.globalart.astrum.httpclient.config;

/**
 * Emitted when {@link HttpClientConfiguration} has changed.
 *
 * @since 3.1
 */
public class HttpClientConfigurationChangedEvent
{
  private final HttpClientConfiguration configuration;

  public HttpClientConfigurationChangedEvent(final HttpClientConfiguration configuration) {
    this.configuration = configuration;
  }

  public HttpClientConfiguration getConfiguration() {
    return configuration;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "configuration=" + configuration +
        '}';
  }
}
