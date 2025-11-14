
package org.globalart.astrum.internal.web;

import java.io.IOException;

import jakarta.servlet.Filter;
import jakarta.servlet.FilterChain;
import jakarta.servlet.FilterConfig;
import jakarta.servlet.ServletException;
import jakarta.servlet.ServletRequest;
import jakarta.servlet.ServletResponse;
import jakarta.servlet.annotation.WebFilter;
import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;

import org.globalart.astrum.common.app.WebFilterPriority;
import org.globalart.astrum.security.authc.AstrumAuthenticationException;
import org.globalart.astrum.servlet.XFrameOptions;

import com.google.common.base.Throwables;
import org.springframework.beans.factory.annotation.Autowired;
import org.apache.shiro.authc.AuthenticationException;
import org.eclipse.jetty.io.EofException;
import org.springframework.core.annotation.Order;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.net.HttpHeaders.X_FRAME_OPTIONS;
import static jakarta.servlet.http.HttpServletResponse.SC_INTERNAL_SERVER_ERROR;
import static jakarta.servlet.http.HttpServletResponse.SC_UNAUTHORIZED;

/**
 * Servlet filter to add error page rendering.
 *
 * @since 2.8
 *
 * @see ErrorPageServlet
 */
@Order(WebFilterPriority.WEB)
@WebFilter("/*")
@Component
public class ErrorPageFilter
    implements Filter
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final XFrameOptions xFrameOptions;

  @Autowired
  public ErrorPageFilter(final XFrameOptions xFrameOptions) {
    this.xFrameOptions = checkNotNull(xFrameOptions);
  }

  @Override
  public void init(final FilterConfig config) throws ServletException {
    // ignore
  }

  @Override
  public void destroy() {
    // ignore
  }

  @Override
  public void doFilter(
      final ServletRequest req,
      final ServletResponse resp,
      final FilterChain chain) throws IOException, ServletException
  {
    final HttpServletRequest request = (HttpServletRequest) req;
    final HttpServletResponse response = (HttpServletResponse) resp;

    // Delegate any exceptions to the ErrorPageServlet via standard sendError servlet api
    // Custom handling here to avoid logging from Jetty implementation
    try {
      chain.doFilter(request, response);
    }
    catch (Exception e) {
      // ASTRUM-49601 / ASTRUM-53768: Rethrow the unwrapped EofException so Jetty properly aborts the response.
      // ASTRUM-53768 extends the original direct-instanceof check to detect an EofException anywhere in the
      // cause chain (e.g. wrapped in UncheckedIOException on the PyPI group serve path). If not rethrown,
      // Jetty continues to the COMPLETE state and validates Content-Length vs bytes written, causing spurious
      // "Insufficient content written" errors when clients disconnect during large file transfers.
      EofException eofException = findEofException(e);
      if (eofException != null) {
        log.debug("Client terminated connection", e);
        throw eofException;
      }

      ErrorPageServlet.attachCause(request, e);
      if (resp.isCommitted()) {
        log.debug("Response is committed, cannot change status", e);
        return;
      }
      response.setHeader(X_FRAME_OPTIONS, xFrameOptions.getValueForPath(request.getPathInfo()));

      int errorCode = SC_INTERNAL_SERVER_ERROR;
      if (e instanceof AuthenticationException || e.getCause() instanceof AuthenticationException
          || e instanceof AstrumAuthenticationException || e.getCause() instanceof AstrumAuthenticationException) {
        errorCode = SC_UNAUTHORIZED;
      }
      response.sendError(errorCode);
    }
  }

  private static EofException findEofException(final Throwable throwable) {
    for (Throwable cause : Throwables.getCausalChain(throwable)) {
      if (cause instanceof EofException) {
        return (EofException) cause;
      }
    }
    return null;
  }
}
