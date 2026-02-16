
package org.globalart.astrum.repository.httpbridge.internal;

import java.io.IOException;
import java.io.InputStream;
import java.util.regex.Pattern;

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
import org.globalart.astrum.repository.http.HttpMethods;

import com.google.common.io.ByteStreams;
import com.google.common.net.HttpHeaders;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.core.annotation.Order;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * {@link Filter} which exhausts request bodies for specific user-agents when errors occur.
 *
 * This is needed to keep HTTPClient based clients such as Apache-Maven happy because they
 * always expect the request body to be fully consumed. Even when the server is responding
 * with an error, such as unauthenticated user (401).
 *
 * @see https://issues.apache.org/jira/browse/HTTPCLIENT-1188
 *
 * @since 3.2
 */
@Order(WebFilterPriority.LEGACY_HTTP_BRIDGE)
@WebFilter(urlPatterns = {
    ViewServlet.MOUNT_POINT,
    "/content/groups/*",
    "/content/repositories/*",
    "/content/sites/*",
    "/service/local/*"
})
@Component
public class ExhaustRequestFilter
    implements Filter
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final Pattern exhaustForAgentsPattern;

  @Autowired
  public ExhaustRequestFilter(
      @Value("${astrum.view.exhaustForAgents:Apache-Maven.*|Apache Ivy.*}") final String exhaustForAgents)
  {
    /*
     * NOTE: An exhaustForAgents pattern delimited by "\\s,\\s" is supported for backwards-compatibility reasons but
     * using a pattern that is instead pipe-delimited is recommended.
     */
    this.exhaustForAgentsPattern = Pattern.compile(exhaustForAgents.replace("\\s,\\s", "|"));
    if (log.isDebugEnabled()) {
      log.debug("astrum.view.exhaustForAgents={}", exhaustForAgentsPattern.pattern());
    }
  }

  @Override
  public void init(final FilterConfig filterConfig) {
    // empty
  }

  @Override
  public void doFilter(
      final ServletRequest request,
      final ServletResponse response,
      final FilterChain chain) throws IOException, ServletException
  {
    try {
      chain.doFilter(request, response);
    }
    finally {
      if (exhaustRequest(request, response)) {
        try (InputStream in = request.getInputStream()) {
          ByteStreams.exhaust(in);
        }
        catch (Exception e) {
          log.debug("Unable to exhaust request", e);
        }
      }
    }
  }

  @Override
  public void destroy() {
    // empty
  }

  /**
   * Returns {@code true} if we need to exhaust the request before responding to the client.
   */
  private boolean exhaustRequest(final ServletRequest request, final ServletResponse response) {
    if (request instanceof HttpServletRequest && response instanceof HttpServletResponse) {

      HttpServletRequest httpRequest = (HttpServletRequest) request;
      HttpServletResponse httpResponse = (HttpServletResponse) response;

      if (log.isTraceEnabled()) {
        final String agent = httpRequest.getHeader(HttpHeaders.USER_AGENT);
        log.trace("status: {}, method: {}, agent: {}, match: {}", httpResponse.getStatus(), httpRequest.getMethod(),
            agent, agent != null && exhaustForAgentsPattern.matcher(agent).matches());
      }

      // only needed when an error occurs...
      if (httpResponse.getStatus() >= 400) {
        String method = httpRequest.getMethod();
        // ...for upload requests
        if (HttpMethods.PUT.equals(method) || HttpMethods.POST.equals(method)) {
          /// ...from an affected user-agent
          String agent = httpRequest.getHeader(HttpHeaders.USER_AGENT);
          return agent != null && exhaustForAgentsPattern.matcher(agent).matches();
        }
      }
    }

    if (log.isTraceEnabled()) {
      log.trace("req: {}, resp: {}", request.getClass(), response.getClass());
    }

    return false;
  }
}
