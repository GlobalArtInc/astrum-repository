
package org.globalart.astrum.internal.webresources;

import java.io.IOException;
import java.io.InputStream;

import jakarta.servlet.ServletException;
import jakarta.servlet.annotation.WebServlet;
import jakarta.servlet.http.HttpServlet;
import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;

import org.globalart.astrum.common.time.Time;
import org.globalart.astrum.common.app.BaseUrlHolder;
import org.globalart.astrum.common.app.WebFilterPriority;
import org.globalart.astrum.servlet.ServletHelper;
import org.globalart.astrum.servlet.XFrameOptions;
import org.globalart.astrum.webresources.WebResource;
import org.globalart.astrum.webresources.WebResource.Prepareable;
import org.globalart.astrum.webresources.WebResourceService;

import org.springframework.beans.factory.annotation.Autowired;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.core.annotation.Order;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Preconditions.checkState;
import static com.google.common.net.HttpHeaders.CACHE_CONTROL;
import static com.google.common.net.HttpHeaders.CONTENT_LENGTH;
import static com.google.common.net.HttpHeaders.CONTENT_TYPE;
import static com.google.common.net.HttpHeaders.IF_MODIFIED_SINCE;
import static com.google.common.net.HttpHeaders.LAST_MODIFIED;
import static com.google.common.net.HttpHeaders.X_FRAME_OPTIONS;
import static com.google.common.net.HttpHeaders.X_XSS_PROTECTION;
import static jakarta.servlet.http.HttpServletResponse.SC_NOT_FOUND;
import static jakarta.servlet.http.HttpServletResponse.SC_NOT_MODIFIED;

/**
 * Provides access to resources via configured {@link WebResourceService}.
 *
 * @since 2.8
 */
@Order(WebFilterPriority.WEB_RESOURCES)
@WebServlet("/*")
@Component
public class WebResourceServlet
    extends HttpServlet
{
  private static final Logger log = LoggerFactory.getLogger(WebResourceServlet.class);

  private final WebResourceService webResources;

  private final long maxAgeSeconds;

  private final XFrameOptions xframeOptions;

  private static final String INDEX_PATH = "/index.html";

  @Autowired
  public WebResourceServlet(
      final WebResourceService webResources,
      final XFrameOptions xframeOptions,
      @Value("${astrum.webresources.maxAge:30days}") final Time maxAge)
  {
    this.webResources = checkNotNull(webResources);
    this.maxAgeSeconds = checkNotNull(maxAge.toSeconds());
    this.xframeOptions = checkNotNull(xframeOptions);
    log.info("Max-age: {} ({} seconds)", maxAge, maxAgeSeconds);
  }

  @Override
  protected void doGet(
      final HttpServletRequest request,
      final HttpServletResponse response) throws ServletException, IOException
  {
    String path = request.getPathInfo();

    // default-page handling
    if ("".equals(path) || "/".equals(path)) {
      path = INDEX_PATH;
    }
    else if (path.endsWith("/")) {
      path += "index.html";
    }
    else if (INDEX_PATH.equals(path)) {
      response.sendRedirect(BaseUrlHolder.getRelativePath()); // prevent browser from sending XHRs to incorrect URL -
      // ASTRUM-14593
      return;
    }

    WebResource resource = webResources.getResource(path);
    if (resource == null) {
      // if there is an index.html for the requested path, redirect to it
      if (webResources.getResource(path + INDEX_PATH) != null) {
        String location = String.format("%s%s/", BaseUrlHolder.getRelativePath(), path);
        log.debug("Redirecting: {} -> {}", path, location);
        response.sendRedirect(location);
      }
      else {
        response.sendError(SC_NOT_FOUND);
      }
      return;
    }

    serveResource(resource, request, response);
  }

  private void serveResource(
      WebResource resource,
      final HttpServletRequest request,
      final HttpServletResponse response) throws IOException
  {
    log.trace("Serving resource: {}", resource);

    // ASTRUM-6569 Add X-Frame-Options header
    response.setHeader(X_FRAME_OPTIONS, xframeOptions.getValueForPath(request.getPathInfo()));
    response.setHeader(X_XSS_PROTECTION, "0");

    // support resources which need to be prepared before serving
    if (resource instanceof Prepareable) {
      resource = ((Prepareable) resource).prepare();
      checkState(resource != null, "Prepared resource is null");
    }
    checkNotNull(resource);

    String contentType = resource.getContentType();
    if (contentType == null) {
      contentType = WebResource.UNKNOWN_CONTENT_TYPE;
    }
    response.setHeader(CONTENT_TYPE, contentType);
    response.setDateHeader(LAST_MODIFIED, resource.getLastModified());

    // set content-length, complain if invalid
    long size = resource.getSize();
    if (size < 0) {
      log.warn("Resource {} has invalid size: {}", resource.getPath(), size);
    }
    response.setHeader(CONTENT_LENGTH, String.valueOf(size));

    // set max-age if cacheable
    if (resource.isCacheable()) {
      response.setHeader(CACHE_CONTROL, "max-age=" + maxAgeSeconds);
    }
    else {
      ServletHelper.addNoCacheResponseHeaders(response);
    }

    // honor if-modified-since GETs
    long ifModifiedSince = request.getDateHeader(IF_MODIFIED_SINCE);
    // handle conditional GETs
    if (ifModifiedSince > -1 && resource.getLastModified() <= ifModifiedSince) {
      // this is a conditional GET using time-stamp, and resource is not modified
      response.setStatus(SC_NOT_MODIFIED);
    }
    else {
      // send the content only if needed (this method will be called for HEAD requests too)
      if ("GET".equalsIgnoreCase(request.getMethod())) {
        try (InputStream in = resource.getInputStream()) {
          ServletHelper.sendContent(in, response);
        }
      }
    }
  }
}
