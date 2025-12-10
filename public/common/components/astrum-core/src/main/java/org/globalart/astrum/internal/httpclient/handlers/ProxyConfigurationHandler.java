
package org.globalart.astrum.internal.httpclient.handlers;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.crypto.secrets.SecretsFactory;
import org.globalart.astrum.httpclient.config.ProxyConfiguration;
import org.globalart.astrum.kv.KeyValueStore;

import org.apache.ibatis.type.TypeHandler;
import org.springframework.stereotype.Component;

/**
 * MyBatis {@link TypeHandler} that maps a {@link ProxyConfiguration} to/from JSON.
 *
 * @since 3.21
 */
@Component
public class ProxyConfigurationHandler
    extends HttpClientConfigurationHandler<ProxyConfiguration>
{
  @Autowired
  public ProxyConfigurationHandler(final SecretsFactory secretsFactory, final KeyValueStore keyValueStore) {
    super(secretsFactory, keyValueStore);
  }
}
