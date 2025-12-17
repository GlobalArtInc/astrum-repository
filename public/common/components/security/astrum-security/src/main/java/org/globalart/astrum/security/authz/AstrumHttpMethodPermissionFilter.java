
package org.globalart.astrum.security.authz;

import jakarta.servlet.annotation.WebFilter;

import org.apache.shiro.web.filter.authz.HttpMethodPermissionFilter;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Component;

/**
 * Astrum {@link HttpMethodPermissionFilter}.
 *
 * @since 3.0
 */
@WebFilter(filterName = AstrumHttpMethodPermissionFilter.NAME)
@Component
public class AstrumHttpMethodPermissionFilter
    extends HttpMethodPermissionFilter
{
  public static final String NAME = "nx-http-permissions";

  protected final Logger log = LoggerFactory.getLogger(getClass());
}
