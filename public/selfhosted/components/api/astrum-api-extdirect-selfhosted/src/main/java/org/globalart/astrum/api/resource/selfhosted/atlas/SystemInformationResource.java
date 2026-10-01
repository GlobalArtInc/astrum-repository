
package org.globalart.astrum.api.resource.selfhosted.atlas;

import java.util.Map;
import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.core.MediaType;
import jakarta.ws.rs.core.Response;

import org.globalart.astrum.common.atlas.SystemInformationGenerator;
import org.globalart.astrum.rest.Resource;

import org.springframework.stereotype.Component;
import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.api.resource.selfhosted.atlas.SystemInformationResource.RESOURCE_URI;

/**
 * Renders system information. This is required for download of the report from the UI.
 */
@Component
@Path(RESOURCE_URI)
public class SystemInformationResource
    implements Resource
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public static final String RESOURCE_URI = "/atlas/system-information";

  private final SystemInformationGenerator systemInformationGenerator;

  @Autowired
  public SystemInformationResource(final SystemInformationGenerator systemInformationGenerator) {
    this.systemInformationGenerator = checkNotNull(systemInformationGenerator);
  }

  @GET
  @Produces(MediaType.APPLICATION_JSON)
  @RequiresPermissions("astrum:atlas:read")
  public Response report() {
    Map<String, Object> report = systemInformationGenerator.report();

    // support downloading the json directly
    return Response.ok(report)
        .header("Content-Disposition", "attachment; filename=\"sysinfo.json\"")
        .build();
  }
}
