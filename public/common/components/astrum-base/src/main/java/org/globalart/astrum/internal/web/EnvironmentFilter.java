
package org.globalart.astrum.internal.web;

import java.io.IOException;

import jakarta.servlet.Filter;
import jakarta.servlet.FilterChain;
import jakarta.servlet.ServletException;
import jakarta.servlet.ServletRequest;
import jakarta.servlet.ServletResponse;
import jakarta.servlet.annotation.WebFilter;
import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;

import org.globalart.astrum.common.app.ApplicationVersion;
import org.globalart.astrum.common.app.BaseUrlManager;
import org.globalart.astrum.common.app.WebFilterPriority;
import org.globalart.astrum.security.UserIdMdcHelper;

import jakarta.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.core.annotation.Order;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.net.HttpHeaders.CONTENT_SECURITY_POLICY;
import static com.google.common.net.HttpHeaders.SERVER;
import static com.google.common.net.HttpHeaders.STRICT_TRANSPORT_SECURITY;
import static com.google.common.net.HttpHeaders.X_CONTENT_TYPE_OPTIONS;

/**
 * Sets up the basic environment for web-requests.
 *
 * @since 3.0
 */
@Order(WebFilterPriority.WEB)
@WebFilter("/*")
@Component
public class EnvironmentFilter
    implements Filter
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  static final String STS_VALUE = "max-age=31536000; includeSubDomains;";

  @Nullable
  private final String serverHeader;

  private final BaseUrlManager baseUrlManager;

  @Autowired
  public EnvironmentFilter(
      final ApplicationVersion applicationVersion,
      final BaseUrlManager baseUrlManager,
      @Value("${astrum.header.server:true}") final boolean includeServerHeader)
  {
    // cache "Server" header value
    checkNotNull(applicationVersion);

    this.serverHeader = includeServerHeader
        ? "Astrum/%s (%s)".formatted(applicationVersion.getVersion(), applicationVersion.getEdition())
        : null;

    this.baseUrlManager = checkNotNull(baseUrlManager);
  }

  @Override
  public void destroy() {
    // ignore
  }

  @Override
  public void doFilter(
      final ServletRequest request,
      final ServletResponse response,
      final FilterChain chain) throws IOException, ServletException
  {
    // start with default unknown user-id in MDC
    UserIdMdcHelper.unknown();

    // detect base-url
    baseUrlManager.detectAndHoldUrl();

    // fill in default response headers
    defaultHeaders((HttpServletRequest) request, (HttpServletResponse) response);

    try {
      chain.doFilter(request, response);
    }
    finally {
      // unset user-id MDC
      UserIdMdcHelper.unset();
    }
  }

  /**
   * Add default headers to servlet response.
   */
  private void defaultHeaders(final HttpServletRequest request, final HttpServletResponse response) {
    if (serverHeader != null) {
      response.setHeader(SERVER, serverHeader);
    }
    // ASTRUM-5023 disable IE for sniffing into response content
    response.setHeader(X_CONTENT_TYPE_OPTIONS, "nosniff");

    response.setHeader(CONTENT_SECURITY_POLICY,
        "default-src " + request.getScheme() + ": data: blob: 'unsafe-inline'; script-src " + request.getScheme()
            + ": 'unsafe-inline' 'unsafe-eval'");

    if ("https".equals(request.getScheme())) {
      response.setHeader(STRICT_TRANSPORT_SECURITY, STS_VALUE);
    }
  }
}
