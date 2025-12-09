
package org.globalart.astrum.internal.httpclient;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.datastore.ConfigStoreSupport;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.httpclient.config.HttpClientConfiguration;
import org.globalart.astrum.transaction.Transactional;

import org.springframework.context.annotation.Primary;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * MyBatis {@link HttpClientConfigurationStore} implementation.
 *
 * @since 3.21
 */
@Primary
@Component
@Qualifier("mybatis")
public class HttpClientConfigurationStoreImpl
    extends ConfigStoreSupport<HttpClientConfigurationDAO>
    implements HttpClientConfigurationStore
{
  @Autowired
  public HttpClientConfigurationStoreImpl(final DataSessionSupplier sessionSupplier) {
    super(sessionSupplier);
  }

  @Override
  public HttpClientConfiguration newConfiguration() {
    return new HttpClientConfigurationData();
  }

  @Transactional
  @Override
  public HttpClientConfiguration load() {
    return dao().get().orElse(null);
  }

  @Transactional
  @Override
  public void save(final HttpClientConfiguration configuration) {
    postCommitEvent(HttpClientConfigurationChanged::new);

    dao().set((HttpClientConfigurationData) configuration);
  }
}
