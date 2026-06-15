
package org.globalart.astrum.coreui.internal.log;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.POST;
import jakarta.ws.rs.Path;

import org.globalart.astrum.common.log.LogMarker;
import org.globalart.astrum.rest.APIConstants;
import org.globalart.astrum.rest.Resource;

import com.google.common.base.Strings;
import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static jakarta.ws.rs.core.MediaType.TEXT_PLAIN;
import org.springframework.stereotype.Component;

/**
 * Log REST resource.
 *
 * @since 2.7
 */
@Component
@Path(LogResource.RESOURCE_URI)
public class LogResource
    implements Resource
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public static final String RESOURCE_URI = APIConstants.INTERNAL_API_PREFIX + "/logging/log";

  public static final String DEFAULT_MARK = "MARK";

  private final LogMarker logMarker;

  @Autowired
  public LogResource(final LogMarker logMarker) {
    this.logMarker = checkNotNull(logMarker);
  }

  /**
   */
  @POST
  @Path("/mark")
  @Consumes({TEXT_PLAIN})
  @RequiresPermissions("astrum:logging:create")
  public void mark(final String message) {
    if (Strings.isNullOrEmpty(message)) {
      logMarker.markLog(DEFAULT_MARK);
    }
    else {
      logMarker.markLog(message);
    }
  }
}
