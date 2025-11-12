
package org.globalart.astrum.internal.metrics;

import java.io.IOException;

import jakarta.servlet.ServletException;
import jakarta.servlet.annotation.WebServlet;
import jakarta.servlet.http.HttpServlet;
import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;
import jakarta.ws.rs.core.HttpHeaders;

import org.globalart.astrum.common.app.WebFilterPriority;

import org.springframework.core.annotation.Order;
import org.springframework.stereotype.Component;

import static jakarta.servlet.http.HttpServletResponse.SC_MOVED_PERMANENTLY;

/**
 * Forwards requests to the moved metrics resources
 */
@Order(WebFilterPriority.WEB)
@Component
@WebServlet(MetricsForwardingServlet.PATH)
public class MetricsForwardingServlet
    extends HttpServlet
{
  private static final String PREFIX = "/service/metrics/";

  private static final int PREFIX_LENGTH = PREFIX.length();

  public static final String PATH = PREFIX + "*";

  @Override
  protected void doGet(
      final HttpServletRequest req,
      final HttpServletResponse resp) throws ServletException, IOException
  {
    resp.setHeader(HttpHeaders.LOCATION, computePath(req));
    resp.setStatus(SC_MOVED_PERMANENTLY);
  }

  private static String computePath(final HttpServletRequest req) {
    return switch (req.getRequestURI().substring(PREFIX_LENGTH)) {
      case "healthcheck" -> req.getContextPath() + "/service/rest/v1/status/check";
      default -> req.getRequestURI().replace("/service/", "/service/rest/");
    };
  }
}
