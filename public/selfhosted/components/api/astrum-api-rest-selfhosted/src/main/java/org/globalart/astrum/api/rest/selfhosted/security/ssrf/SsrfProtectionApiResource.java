
package org.globalart.astrum.api.rest.selfhosted.security.ssrf;

import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.PUT;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;

import org.globalart.astrum.api.rest.selfhosted.security.ssrf.model.SsrfProtectionConfigurationXO;
import org.globalart.astrum.rest.Resource;
import org.globalart.astrum.validation.ssrf.AntiSsrfService;

import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;
import static org.globalart.astrum.rest.APIConstants.V1_API_PREFIX;

/**
 * REST resource for managing SSRF protection settings.
 */
@Component
@Path(SsrfProtectionApiResource.RESOURCE_URI)
@Consumes(APPLICATION_JSON)
@Produces(APPLICATION_JSON)
public class SsrfProtectionApiResource
    implements Resource, SsrfProtectionApiResourceDoc
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  static final String RESOURCE_URI = V1_API_PREFIX + "/security/ssrf-protection";

  private final AntiSsrfService antiSsrfService;

  @Autowired
  public SsrfProtectionApiResource(final AntiSsrfService antiSsrfService) {
    this.antiSsrfService = checkNotNull(antiSsrfService);
  }

  @GET
  @RequiresAuthentication
  @RequiresPermissions("astrum:*")
  @Override
  public SsrfProtectionConfigurationXO getConfiguration() {
    return new SsrfProtectionConfigurationXO(antiSsrfService.getConfiguration());
  }

  @PUT
  @RequiresAuthentication
  @RequiresPermissions("astrum:*")
  @Override
  public SsrfProtectionConfigurationXO updateConfiguration(final SsrfProtectionConfigurationXO configuration) {
    antiSsrfService.updateConfiguration(configuration.toConfiguration());
    return new SsrfProtectionConfigurationXO(antiSsrfService.getConfiguration());
  }
}
