
package org.globalart.astrum.coreui.internal.wonderland;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.core.MediaType;

import org.globalart.astrum.bootstrap.entrypoint.edition.AstrumEditionSelector;
import org.globalart.astrum.common.app.ApplicationVersion;
import org.globalart.astrum.rest.Resource;

import org.apache.shiro.authz.annotation.RequiresUser;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Status resource.
 *
 * @since 3.0
 */
@Component
@Path(StatusResource.RESOURCE_URI)
public class StatusResource
    implements Resource
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public static final String RESOURCE_URI = "/wonderland/status";

  private static final String CLOUD_EDITION_ID = "astrum-cloud-edition";

  private final ApplicationVersion applicationVersion;

  private final AstrumEditionSelector astrumEditionSelector;

  @Autowired
  public StatusResource(
      final ApplicationVersion applicationVersion,
      final AstrumEditionSelector astrumEditionSelector)
  {
    this.applicationVersion = checkNotNull(applicationVersion);
    this.astrumEditionSelector = checkNotNull(astrumEditionSelector);
  }

  @GET
  @Produces({MediaType.APPLICATION_XML, MediaType.APPLICATION_JSON})
  @RequiresUser
  public StatusXO get() {
    StatusXO result = new StatusXO();
    result.setVersion(applicationVersion.getVersion());
    result.setEdition(applicationVersion.getEdition());
    result.setIsCloud(CLOUD_EDITION_ID.equals(astrumEditionSelector.getCurrent().getId()));
    return result;
  }
}
