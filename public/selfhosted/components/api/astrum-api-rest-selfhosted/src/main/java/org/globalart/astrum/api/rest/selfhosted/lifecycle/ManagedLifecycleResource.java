
package org.globalart.astrum.api.rest.selfhosted.lifecycle;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.PUT;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;

import org.globalart.astrum.common.app.ManagedLifecycle.Phase;
import org.globalart.astrum.common.app.ManagedLifecycleManager;
import org.globalart.astrum.rest.Resource;
import org.globalart.astrum.rest.WebApplicationMessageException;

import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static jakarta.ws.rs.core.MediaType.TEXT_PLAIN;
import static jakarta.ws.rs.core.Response.Status.INTERNAL_SERVER_ERROR;
import static org.globalart.astrum.rest.APIConstants.V1_API_PREFIX;
import org.springframework.stereotype.Component;

/**
 * REST implementation to manage the Astrum application lifecycle. (also exposed as JMX)
 *
 * @since 3.16
 */
@Path(ManagedLifecycleResource.RESOURCE_URI)
@Component
public class ManagedLifecycleResource
    implements Resource, ManagedLifecycleResourceDoc
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public static final String RESOURCE_URI = V1_API_PREFIX + "/lifecycle";

  private final ManagedLifecycleManager lifecycleManager;

  @Autowired
  public ManagedLifecycleResource(final ManagedLifecycleManager lifecycleManager) {
    this.lifecycleManager = checkNotNull(lifecycleManager);
  }

  @GET
  @Path("phase")
  @Produces(TEXT_PLAIN)
  @Override
  public String getPhase() {
    return lifecycleManager.getCurrentPhase().name();
  }

  @PUT
  @Path("phase")
  @Consumes(TEXT_PLAIN)
  @RequiresAuthentication
  @RequiresPermissions("astrum:*")
  @Override
  public void setPhase(final String phase) {
    try {
      lifecycleManager.to(Phase.valueOf(phase));
    }
    catch (Exception e) {
      log.warn("Problem moving to phase {}", phase, e);
      throw new WebApplicationMessageException(INTERNAL_SERVER_ERROR, "Problem moving to phase " + phase + ": " + e);
    }
  }

  @PUT
  @Path("bounce")
  @Consumes(TEXT_PLAIN)
  @RequiresAuthentication
  @RequiresPermissions("astrum:*")
  @Override
  public void bounce(final String phase) {
    try {
      lifecycleManager.bounce(Phase.valueOf(phase));
    }
    catch (Exception e) {
      log.warn("Problem bouncing phase {}", phase, e);
      throw new WebApplicationMessageException(INTERNAL_SERVER_ERROR, "Problem bouncing phase " + phase + ": " + e);
    }
  }
}
