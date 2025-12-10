
package org.globalart.astrum.internal.httpclient.handlers;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.crypto.secrets.SecretsFactory;
import org.globalart.astrum.httpclient.config.ConnectionConfiguration;
import org.globalart.astrum.kv.KeyValueStore;

import org.apache.ibatis.type.TypeHandler;
import org.springframework.stereotype.Component;

/**
 * MyBatis {@link TypeHandler} that maps a {@link ConnectionConfiguration} to/from JSON.
 *
 * @since 3.21
 */
@Component
public class ConnectionConfigurationHandler
    extends HttpClientConfigurationHandler<ConnectionConfiguration>
{
  @Autowired
  public ConnectionConfigurationHandler(final SecretsFactory secretsFactory, final KeyValueStore keyValueStore) {
    super(secretsFactory, keyValueStore);
  }
}
