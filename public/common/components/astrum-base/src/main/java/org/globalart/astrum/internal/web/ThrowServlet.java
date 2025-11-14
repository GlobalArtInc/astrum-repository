
package org.globalart.astrum.internal.web;

import java.io.IOException;

import jakarta.servlet.ServletException;
import jakarta.servlet.annotation.WebServlet;
import jakarta.servlet.http.HttpServlet;
import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;

import org.globalart.astrum.common.app.WebFilterPriority;
import org.globalart.astrum.common.text.Strings2;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.core.annotation.Order;
import org.springframework.stereotype.Component;

/**
 * Helper servlet to throw an exception.
 *
 * @since 3.0
 */
// needs to be first-most when calculating order (some fudge room for edge-cases)
@Order(WebFilterPriority.WEB)
@WebServlet("/throw.html")
@Component
public class ThrowServlet
    extends HttpServlet
{
  private static final Logger log = LoggerFactory.getLogger(ThrowServlet.class);

  private enum Type
  {
    RUNTIME,
    ERROR,
    IO,
    SERVLET;

    static Type parse(final String value) {
      if (value == null) {
        return RUNTIME;
      }
      return valueOf(Strings2.upper(value));
    }
  }

  @Override
  protected void service(
      final HttpServletRequest request,
      final HttpServletResponse response) throws ServletException, IOException
  {
    Type type = Type.parse(request.getParameter("type"));
    String message = request.getParameter("message");
    log.info("Throwing {} w/message: {}", type, message);

    switch (type) {
      case RUNTIME:
        throw new RuntimeException(message);

      case IO:
        throw new IOException(message);

      case SERVLET:
        throw new ServletException(message);

      case ERROR:
      default:
        throw new Error(message);
    }
  }
}
