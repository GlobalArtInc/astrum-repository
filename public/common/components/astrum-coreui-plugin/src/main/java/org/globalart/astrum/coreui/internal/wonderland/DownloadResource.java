
package org.globalart.astrum.coreui.internal.wonderland;

import java.io.IOException;
import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.PathParam;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.WebApplicationException;
import jakarta.ws.rs.core.Response;

import org.globalart.astrum.common.supportzip.SupportZipConstants;
import org.globalart.astrum.common.wonderland.AuthTicketService;
import org.globalart.astrum.common.wonderland.DownloadService;
import org.globalart.astrum.common.wonderland.DownloadService.Download;
import org.globalart.astrum.rest.NotCacheable;
import org.globalart.astrum.rest.Resource;

import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.net.HttpHeaders.CONTENT_DISPOSITION;
import static com.google.common.net.HttpHeaders.CONTENT_LENGTH;
import static jakarta.ws.rs.core.Response.Status.BAD_REQUEST;
import static jakarta.ws.rs.core.Response.Status.INTERNAL_SERVER_ERROR;
import static jakarta.ws.rs.core.Response.Status.NOT_FOUND;
import org.springframework.stereotype.Component;

/**
 * Downloads resource.
 *
 * @since 2.8
 */
@Component
@Path(SupportZipConstants.WONDERLAND_DOWNLOAD_RESOURCE_URI)
public class DownloadResource
    implements Resource
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final DownloadService downloadService;

  private final AuthTicketService authTicketService;

  @Autowired
  public DownloadResource(final DownloadService downloadService, final AuthTicketService authTicketService) {
    this.downloadService = checkNotNull(downloadService);
    this.authTicketService = checkNotNull(authTicketService);
  }

  /**
   * Download a support ZIP file.
   */
  @GET
  @Path("{fileName}")
  @Produces("application/zip")
  @RequiresPermissions("astrum:wonderland:download")
  @NotCacheable
  public Response downloadZip(@PathParam("fileName") final String fileName) {
    checkNotNull(fileName);
    log.info("Download: {}", fileName);

    String authTicket = authTicketService.createTicket();

    // handle one-time auth
    if (authTicket == null) {
      throw new WebApplicationException("Missing authentication ticket", BAD_REQUEST);
    }

    try {
      Download download = downloadService.get(fileName, authTicket);

      if (download == null) {
        return Response.status(NOT_FOUND).build();
      }

      log.debug("Sending support ZIP file: {}", fileName);
      return Response.ok(download.getBytes())
          .header(CONTENT_DISPOSITION, "attachment; filename=\"" + fileName + "\"")
          .header(CONTENT_LENGTH, download.getLength())
          .build();
    }
    catch (IOException e) {
      log.error("Failed to serve file for download {}", fileName, e);
      throw new WebApplicationException("Failed to service file for download", INTERNAL_SERVER_ERROR);
    }
  }
}
