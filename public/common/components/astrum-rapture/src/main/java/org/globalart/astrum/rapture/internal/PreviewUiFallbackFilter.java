
package org.globalart.astrum.rapture.internal;

import java.io.IOException;

import jakarta.servlet.Filter;
import jakarta.servlet.FilterChain;
import jakarta.servlet.ServletException;
import jakarta.servlet.ServletRequest;
import jakarta.servlet.ServletResponse;
import jakarta.servlet.annotation.WebFilter;
import jakarta.servlet.http.HttpServletRequest;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.core.annotation.Order;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.common.app.WebFilterPriority.WEB_RESOURCES;

/**
 * SPA fallback filter for Preview UI path-based routing.
 *
 * Intercepts requests to /preview/* paths and forwards them to /index.html,
 * allowing the React router to handle client-side routing.
 *
 * This enables clean URLs like /preview/browse/welcome instead of
 * hash-based URLs like #preview/browse/welcome.
 *
 * @since 3.92.0
 */
@WebFilter(filterName = PreviewUiFallbackFilter.NAME, urlPatterns = "/preview/*")
@Order(WEB_RESOURCES - 100)
@Component
public class PreviewUiFallbackFilter
    implements Filter
{
  private static final Logger log = LoggerFactory.getLogger(PreviewUiFallbackFilter.class);

  public static final String NAME = "as-preview-ui-fallback";

  private static final String ROOT_PATH = "/";

  @Override
  public void doFilter(
      final ServletRequest request,
      final ServletResponse response,
      final FilterChain chain) throws IOException, ServletException
  {
    if (request instanceof HttpServletRequest) {
      HttpServletRequest httpRequest = (HttpServletRequest) request;
      String requestUri = httpRequest.getRequestURI();

      log.debug("Preview UI fallback: forwarding {} to {}", requestUri, ROOT_PATH);

      // Forward to root (/) for SPA routing - the React app handles all /preview/* routes
      request.getRequestDispatcher(ROOT_PATH).forward(request, response);
      return;
    }

    // Non-HTTP requests pass through
    chain.doFilter(request, response);
  }
}
