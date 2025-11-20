

package org.globalart.astrum.servlet;

import java.io.Serializable;
import java.util.HashSet;
import java.util.Set;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

/**
 * @since 3.11
 */
@Component
public class XFrameOptions
    implements Serializable
{
  private static final long serialVersionUID = 5092514762222572451L;

  private static final String COPYRIGHT_PATH = "/COPYRIGHT.html";

  private static final String HEALTHCHECK_PATH = "/static/healthcheck-tos.html";

  private static final String OC_LICENSE_PATH = "/OC-LICENSE.html";

  private static final String PRO_LICENSE_PATH = "/PRO-LICENSE.html";

  private static final String CE_LICENSE_PATH = "/CE-LICENSE.html";

  private static final String SWAGGER_UI = "/swagger-ui/";

  public static final String DENY = "DENY";

  public static final String SAME_ORIGIN = "SAMEORIGIN";

  private final boolean defaultDeny;

  private final Set<String> frameablePaths;

  @Autowired
  public XFrameOptions(
      @Value("${astrum.http.denyframe.enabled:true}") final boolean defaultDeny)
  {
    this.defaultDeny = defaultDeny;
    frameablePaths = new HashSet<>();
    frameablePaths.add(COPYRIGHT_PATH);
    frameablePaths.add(HEALTHCHECK_PATH);
    frameablePaths.add(OC_LICENSE_PATH);
    frameablePaths.add(PRO_LICENSE_PATH);
    frameablePaths.add(CE_LICENSE_PATH);
    frameablePaths.add(SWAGGER_UI);
  }

  public String getValueForPath(final String path) {
    if (!defaultDeny || frameablePaths.contains(path)) {
      return SAME_ORIGIN;
    }
    return DENY;
  }
}
