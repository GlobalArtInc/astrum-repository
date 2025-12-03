
package org.globalart.astrum.httpclient.internal;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.httpclient.HttpClientManager;
import org.globalart.astrum.httpclient.config.HttpClientConfiguration;

import org.springframework.beans.factory.FactoryBean;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Initial {@link HttpClientConfiguration} provider.
 *
 * @since 3.0
 */
@Component
@Qualifier("initial")
public class InitialHttpClientConfigurationProvider
    implements FactoryBean<HttpClientConfiguration>
{
  private final HttpClientManager clientManager;

  @Autowired
  public InitialHttpClientConfigurationProvider(final HttpClientManager clientManager) {
    this.clientManager = checkNotNull(clientManager);
  }

  @Override
  public HttpClientConfiguration getObject() throws Exception {
    return clientManager.newConfiguration();
  }

  @Override
  public Class<?> getObjectType() {
    return HttpClientConfiguration.class;
  }
}
