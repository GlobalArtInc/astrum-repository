
package org.globalart.astrum.httpclient.internal;

import org.globalart.astrum.httpclient.HttpClientManager;

import org.springframework.beans.factory.annotation.Autowired;
import org.apache.http.client.HttpClient;
import org.apache.http.impl.client.CloseableHttpClient;
import org.springframework.beans.factory.FactoryBean;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * {@link HttpClient} (and {@link CloseableHttpClient}) provider via {@link HttpClientManager}.
 *
 * @since 3.0
 */
@Component
public class HttpClientProvider
    implements FactoryBean<CloseableHttpClient>
{
  private final HttpClientManager httpClientManager;

  @Autowired
  public HttpClientProvider(final HttpClientManager httpClientManager) {
    this.httpClientManager = checkNotNull(httpClientManager);
  }

  @Override
  public CloseableHttpClient getObject() {
    return httpClientManager.create();
  }

  @Override
  public Class<?> getObjectType() {
    return CloseableHttpClient.class;
  }

  @Override
  public boolean isSingleton() {
    return false;
  }
}
