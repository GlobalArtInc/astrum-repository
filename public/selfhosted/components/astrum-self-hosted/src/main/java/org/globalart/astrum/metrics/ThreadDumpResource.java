
package org.globalart.astrum.metrics;

import java.lang.management.ManagementFactory;

import jakarta.ws.rs.DefaultValue;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.QueryParam;
import jakarta.ws.rs.core.CacheControl;
import jakarta.ws.rs.core.MediaType;
import jakarta.ws.rs.core.Response;
import jakarta.ws.rs.core.Response.ResponseBuilder;
import jakarta.ws.rs.core.StreamingOutput;

import org.globalart.astrum.rest.Resource;

import com.codahale.metrics.jvm.ThreadDump;
import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static jakarta.ws.rs.core.HttpHeaders.CONTENT_DISPOSITION;
import org.springframework.stereotype.Component;

/**
 * Provides current stacktraces for running threads
 */
@Path("/metrics/threads")
@Component
public class ThreadDumpResource
    implements Resource
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final ThreadDump threadDump = new ThreadDump(ManagementFactory.getThreadMXBean());

  private final CacheControl cacheControl = new CacheControl();

  public ThreadDumpResource() {
    cacheControl.setMustRevalidate(true);
    cacheControl.setNoCache(true);
    cacheControl.setNoStore(true);
  }

  @GET
  @RequiresPermissions("astrum:metrics:read")
  public Response dump(
      @DefaultValue("false") @QueryParam("download") final boolean download,
      @DefaultValue("false") @QueryParam("monitors") final boolean monitors,
      @DefaultValue("false") @QueryParam("synchronizers") final boolean synchronizers)
  {
    ResponseBuilder response = Response.ok(dump(monitors, synchronizers), MediaType.TEXT_PLAIN);

    if (download) {
      // we only care if download is true
      response.header(CONTENT_DISPOSITION, "attachment; filename='threads.txt'");
    }
    response.cacheControl(cacheControl);

    return response.build();
  }

  private StreamingOutput dump(final boolean lockedMonitors, final boolean lockedSynchronizers) {
    return out -> threadDump.dump(lockedMonitors, lockedSynchronizers, out);
  }
}
