
package org.globalart.astrum.siesta;

import java.io.IOException;

import jakarta.servlet.ServletConfig;
import jakarta.servlet.ServletException;
import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;

import org.globalart.astrum.rest.Component;
import org.globalart.astrum.rest.Resource;

// ASTRUM-46395: Dispatcher moved from org.jboss.resteasy.core to org.jboss.resteasy.spi
// in RESTEasy 7.
import org.jboss.resteasy.spi.Dispatcher;

/**
 * Siesta {@link Component} (and {@link Resource} container abstraction.
 *
 * @since 3.0
 */
public interface ComponentContainer
{
  void init(final ServletConfig config) throws ServletException;

  void service(final HttpServletRequest request, HttpServletResponse response) throws ServletException, IOException;

  void destroy();

  Dispatcher getDispatcher();
}
