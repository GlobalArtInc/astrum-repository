
package org.globalart.astrum.rest.client;

import java.net.URI;

import jakarta.ws.rs.client.Client;

/**
 * REST client factory.
 *
 * @since 3.0
 */
public interface RestClientFactory
{

  Client create(RestClientConfiguration configuration);

  default Client create() {
    return create(RestClientConfiguration.DEFAULTS);
  }

  /**
   * Returns a proxy backed by the given client and URI.
   *
   * @since 3.2.1
   */
  <T> T proxy(Class<T> api, Client client, URI baseUri);
}
