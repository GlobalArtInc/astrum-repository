
package org.globalart.astrum.bootstrap.jetty;

import java.util.ArrayList;
import java.util.List;

/**
 * Jetty server default configuration.
 */
public class JettyServerConfiguration
{
  private final List<ConnectorConfiguration> defaultConnectors;

  private final List<ConnectorConfiguration> customConnectors;

  public JettyServerConfiguration(final List<ConnectorConfiguration> defaultConnectors) {
    this(defaultConnectors, new ArrayList<>());
  }

  public JettyServerConfiguration(
      final List<ConnectorConfiguration> defaultConnectors,
      final List<ConnectorConfiguration> customConnectors)
  {
    this.defaultConnectors = defaultConnectors;
    this.customConnectors = customConnectors;
  }

  public List<ConnectorConfiguration> defaultConnectors() {
    return defaultConnectors;
  }

  public List<ConnectorConfiguration> customConnectors() {
    return customConnectors;
  }
}
