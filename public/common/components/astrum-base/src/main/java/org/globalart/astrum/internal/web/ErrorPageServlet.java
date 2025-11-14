
package org.globalart.astrum.internal.web;

import java.io.IOException;
import java.util.stream.Stream;

import org.globalart.astrum.common.app.WebFilterPriority;
import org.globalart.astrum.internal.web.ErrorPageService.ErrorInfo;

import jakarta.servlet.ServletException;
import jakarta.servlet.annotation.WebServlet;
import jakarta.servlet.http.HttpServlet;
import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;
import org.eclipse.jetty.io.QuietException;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.core.annotation.Order;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Throwables.getRootCause;

/**
 * An {@code error.html} servlet to handle generic servlet error-page dispatched requests.
 *
 * @since 2.8
 *
 * @see ErrorPageFilter
 */
// needs to be first-most when calculating order (some fudge room for edge-cases)
@Order(WebFilterPriority.WEB)
@WebServlet("/error.html")
@Component
public class ErrorPageServlet
    extends HttpServlet
{
  private static final Logger log = LoggerFactory.getLogger(ErrorPageServlet.class);

  /**
   * @since 3.0
   */
  private static final String ERROR_SERVLET_NAME = "jakarta.servlet.error.servlet_name";

  /**
   * @since 3.0
   */
  private static final String ERROR_REQUEST_URI = "jakarta.servlet.error.request_uri";

  /**
   * @since 3.0
   */
  private static final String ERROR_STATUS_CODE = "jakarta.servlet.error.status_code";

  /**
   * @since 3.0
   */
  private static final String ERROR_MESSAGE = "jakarta.servlet.error.message";

  /**
   * @since 3.0
   */
  private static final String ERROR_EXCEPTION_TYPE = "jakarta.servlet.error.exception_type";

  /**
   * @since 3.0
   */
  private static final String ERROR_EXCEPTION = "jakarta.servlet.error.exception";

  private final ErrorPageService errorPageService;

  @Autowired
  public ErrorPageServlet(final ErrorPageService errorPageService) {
    this.errorPageService = checkNotNull(errorPageService);
  }

  @SuppressWarnings("unused")
  @Override
  protected void service(
      final HttpServletRequest request,
      final HttpServletResponse response) throws ServletException, IOException
  {
    String servletName = (String) request.getAttribute(ERROR_SERVLET_NAME);
    String requestUri = (String) request.getAttribute(ERROR_REQUEST_URI);
    Integer errorCode = (Integer) request.getAttribute(ERROR_STATUS_CODE);
    String errorMessage = (String) request.getAttribute(ERROR_MESSAGE);
    Class<?> causeType = (Class<?>) request.getAttribute(ERROR_EXCEPTION_TYPE);
    Throwable cause = (Throwable) request.getAttribute(ERROR_EXCEPTION);

    log.trace("Handling errorCode {} for {} uri {} message '{}'", errorCode, servletName, requestUri, errorMessage,
        cause);

    errorPageService.writeErrorResponse(new ErrorInfo(errorCode, errorMessage, cause), request, response);
  }

  /**
   * Attach exception details to request.
   *
   * @since 3.0
   */
  static void attachCause(final HttpServletRequest request, final Throwable cause) {
    if (isJavaLangError(cause)) {
      // Log java.lang.Error exceptions at error level
      log.error("Unexpected exception", getRootCause(cause));
    }
    else if (isQuietException(cause)) {
      log.trace("Caught Jetty QuietException ignoring", cause);
    }
    else {
      log.debug("Attaching cause", cause);
    }
    request.setAttribute(ERROR_EXCEPTION_TYPE, cause.getClass());
    request.setAttribute(ERROR_EXCEPTION, cause);
  }

  /*
   * Jetty uses QuietExceptions for failures outside the application such as the client disconnecting or invalid
   * encoding.
   */
  private static boolean isQuietException(final Throwable e) {
    if (QuietException.isQuiet(e)) {
      return true;
    }
    return Stream.of(e.getSuppressed())
        .anyMatch(t -> t instanceof QuietException);
  }

  private static boolean isJavaLangError(final Throwable e) {
    return getRootCause(e) instanceof Error;
  }
}
