
package org.globalart.astrum.bootstrap.jetty;

import org.eclipse.jetty.http.HttpScheme;
import org.eclipse.jetty.server.HttpConfiguration;

/**
 * Connector configuration should be registered as service by plugin requesting dedicated connectors, and unregistered
 * once the connector is not needed.
 */
public interface ConnectorConfiguration
{
  /**
   * The required connector scheme.
   */
  HttpScheme getScheme();

  /**
   * The required connector port.
   */
  int getPort();

  /**
   * Allows implementation to customize the default configuration for needed connector or replace it completely.
   */
  HttpConfiguration customize(HttpConfiguration configuration);
}
