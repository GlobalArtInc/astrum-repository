
package org.globalart.astrum.siesta;

import org.globalart.astrum.siesta.internal.resteasy.ComponentContainerImpl;

// ASTRUM-46395: javax.servlet → jakarta.servlet for RESTEasy 7.
import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;
import jakarta.ws.rs.Path;
import org.jboss.resteasy.core.ResourceMethodInvoker;
import org.jboss.resteasy.core.SynchronousDispatcher;
import org.jboss.resteasy.plugins.server.servlet.HttpServletInputMessage;
import org.jboss.resteasy.spi.HttpRequest;
import org.jboss.resteasy.spi.ResourceInvoker;
import org.jboss.resteasy.spi.ResteasyDeployment;

import static java.util.Objects.nonNull;
import static java.util.Objects.requireNonNull;
import static org.jboss.resteasy.plugins.server.servlet.ServletUtil.extractHttpHeaders;
import static org.jboss.resteasy.plugins.server.servlet.ServletUtil.extractUriInfo;

/**
 * Resource Method finder for all the resources loaded through the {@link SiestaServlet}
 * and {@link ComponentContainer}.
 *
 * @since 3.20
 */
public class SiestaResourceMethodFinder
{
  private ComponentContainerImpl componentContainer;

  private ResteasyDeployment deployment;

  public SiestaResourceMethodFinder(
      final ComponentContainerImpl componentContainer,
      final ResteasyDeployment deployment)
  {
    this.componentContainer = requireNonNull(componentContainer);
    this.deployment = requireNonNull(deployment);
  }

  public String getResourceMethodPath(
      final HttpServletRequest request,
      final HttpServletResponse response)
  {
    StringBuilder buffer = new StringBuilder();
    ResourceInvoker invoker = getResourceMethod(request, response);

    if (invoker instanceof ResourceMethodInvoker method) {
      Path classPath = method.getResourceClass().getAnnotation(Path.class);
      if (nonNull(classPath)) {
        buffer.append(maybePrependWithForwardSlash(classPath.value()));
      }

      Path methodPath = method.getMethod().getDeclaredAnnotation(Path.class);
      if (nonNull(methodPath)) {
        buffer.append(maybePrependWithForwardSlash(methodPath.value()));
      }
    }

    return cleanForwardSlashes(buffer.toString());
  }

  public ResourceInvoker getResourceMethod(
      final HttpServletRequest request,
      final HttpServletResponse response)
  {
    HttpRequest httpRequest = new HttpServletInputMessage(
        request,
        response,
        request.getServletContext(),
        null,
        extractHttpHeaders(request),
        extractUriInfo(request, SiestaServlet.REST_SERVICE_MP),
        request.getMethod(),
        (SynchronousDispatcher) this.componentContainer.getDispatcher());

    return deployment.getRegistry().getResourceInvoker(httpRequest);
  }

  private static String maybePrependWithForwardSlash(final String value) {
    return value.startsWith("/") ? value : "/" + value;
  }

  private static String cleanForwardSlashes(final String s) {
    return s.replaceAll("//", "/");
  }
}
