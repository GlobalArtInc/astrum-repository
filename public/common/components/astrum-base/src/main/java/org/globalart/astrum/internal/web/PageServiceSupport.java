
package org.globalart.astrum.internal.web;

import java.io.IOException;
import java.io.PrintWriter;
import java.net.URL;

import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;

import org.globalart.astrum.common.template.TemplateHelper;
import org.globalart.astrum.common.template.TemplateParameters;
import org.globalart.astrum.servlet.ServletHelper;
import org.globalart.astrum.servlet.XFrameOptions;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.net.HttpHeaders.X_FRAME_OPTIONS;

/**
 * Support service to write a rendered HTML page
 */
public abstract class PageServiceSupport
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  protected final TemplateHelper templateHelper;

  private final XFrameOptions xFrameOptions;

  private final URL template;

  protected PageServiceSupport(
      final TemplateHelper templateHelper,
      final XFrameOptions xFrameOptions,
      final String templateResource)
  {
    this.templateHelper = checkNotNull(templateHelper);
    this.xFrameOptions = checkNotNull(xFrameOptions);

    template = getClass().getResource(templateResource);
    checkNotNull(template);
  }

  /**
   * Writes a response with a header specifying caches should be disabled.
   *
   * @param params
   * @param request
   * @param response
   * @throws IOException
   */
  protected void writeResponseWithoutCaching(
      final TemplateParameters params,
      final HttpServletRequest request,
      final HttpServletResponse response) throws IOException
  {
    ServletHelper.addNoCacheResponseHeaders(response);
    writeResponseWithCaching(params, request, response);
  }

  /**
   * Writes a response and allows caching of the page.
   *
   * @param params the parameters to render
   * @param request the http request
   * @param response the http response
   * @throws IOException
   */
  protected void writeResponseWithCaching(
      final TemplateParameters params,
      final HttpServletRequest request,
      final HttpServletResponse response) throws IOException
  {
    writeHeaders(request, response);

    String html = templateHelper.render(template, params);

    try (PrintWriter out = response.getWriter()) {
      out.println(html);
    }
  }

  private void writeHeaders(final HttpServletRequest request, final HttpServletResponse response) {
    log.trace("Writing headers");
    response.setHeader(X_FRAME_OPTIONS, xFrameOptions.getValueForPath(request.getPathInfo()));
    response.setContentType("text/html");
  }
}
