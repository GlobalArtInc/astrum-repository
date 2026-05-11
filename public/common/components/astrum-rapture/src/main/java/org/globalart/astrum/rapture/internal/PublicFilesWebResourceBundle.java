
package org.globalart.astrum.rapture.internal;

import java.net.URL;
import java.util.Arrays;
import java.util.List;
import java.util.Objects;
import java.util.stream.Collectors;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.mime.MimeSupport;
import org.globalart.astrum.webresources.UrlWebResource;
import org.globalart.astrum.webresources.WebResource;
import org.globalart.astrum.webresources.WebResourceBundle;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

@Component
public class PublicFilesWebResourceBundle
    implements WebResourceBundle
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final List<String> FILES =
      Arrays.asList("apple-touch-icon.png", "browserconfig.xml", "favicon.ico", "favicon-16x16.png",
          "favicon-32x32.png", "mstile-144x144.png", "mstile-150x150.png", "mstile-310x310.png", "mstile-70x70.png",
          "CORE-LICENSE.html", "PRO-LICENSE.html", "CE-LICENSE.html", "robots.txt", "safari-pinned-tab.svg",
          "license.css");

  private final MimeSupport mimeSupport;

  @Autowired
  public PublicFilesWebResourceBundle(final MimeSupport mimeSupport) {
    this.mimeSupport = checkNotNull(mimeSupport);
  }

  @Override
  public List<WebResource> getResources() {
    return FILES.stream()
        .map(this::toWebResource)
        .filter(Objects::nonNull)
        .collect(Collectors.toList());
  }

  private WebResource toWebResource(final String filename) {
    URL url = PublicFilesWebResourceBundle.class.getResource("/static/public/" + filename);
    if (url == null) {
      return null;
    }
    return new UrlWebResource(url, '/' + filename, getContentType(filename));
  }

  private String getContentType(final String filename) {
    return mimeSupport.guessMimeTypeFromPath(filename);
  }
}
