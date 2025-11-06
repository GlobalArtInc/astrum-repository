
package org.globalart.astrum.bootstrap.jetty;

import java.util.List;

import org.eclipse.jetty.http.HttpScheme;

/**
 * Connector registrar component.
 */
public interface ConnectorRegistrar
{
  /**
   * Returns the list of available HTTP schemes. This list depends on current Jetty XML configuration (is HTTPS
   * enabled). If schema is not available, method {@link #addConnector(ConnectorConfiguration)} will reject to add it.
   */
  List<HttpScheme> availableSchemes();

  /**
   * Returns the list of already occupied ports. If configuration port is occupied, method {@link
   * #addConnector(ConnectorConfiguration)} will reject to add configuration.
   */
  List<Integer> unavailablePorts();

  /**
   * Registers a new Jetty {@link ConnectorConfiguration}, that creates a new connector on Jetty based on passed
   * in configuration.
   *
   * @throws UnsupportedHttpSchemeException if requested HTTP schema is not available.
   * @throws IllegalArgumentException requested port is not available, if current connectorConfiguration instance
   *           was already added
   */
  void addConnector(ConnectorConfiguration connectorConfiguration);

  /**
   * Removes a Jetty {@link ConnectorConfiguration} by closing and removing the Jetty connector. Caller MUST use the
   * same instance of {@link ConnectorConfiguration} that was used with {@link #addConnector(ConnectorConfiguration)}
   * method to create a connector in the first place.
   */
  void removeConnector(ConnectorConfiguration connectorConfiguration);
}
