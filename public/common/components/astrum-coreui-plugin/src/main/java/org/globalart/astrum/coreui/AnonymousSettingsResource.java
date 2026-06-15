
package org.globalart.astrum.coreui;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.validation.Valid;
import jakarta.validation.constraints.NotNull;
import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.PUT;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;

import org.globalart.astrum.rest.Resource;
import org.globalart.astrum.security.anonymous.AnonymousConfiguration;
import org.globalart.astrum.security.anonymous.AnonymousManager;

import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;
import org.springframework.stereotype.Component;

/**
 * @since 3.19
 */
@Component
@Consumes(APPLICATION_JSON)
@Produces(APPLICATION_JSON)
@Path(AnonymousSettingsResource.RESOURCE_PATH)
public class AnonymousSettingsResource
    implements Resource
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  static final String RESOURCE_PATH = "internal/ui/anonymous-settings";

  private final AnonymousManager anonymousManager;

  @Autowired
  public AnonymousSettingsResource(final AnonymousManager anonymousManager) {
    this.anonymousManager = checkNotNull(anonymousManager);
  }

  @GET
  @RequiresPermissions("astrum:settings:read")
  public AnonymousSettingsXO read() {
    AnonymousConfiguration config = anonymousManager.getConfiguration();
    AnonymousSettingsXO xo = new AnonymousSettingsXO();

    xo.setEnabled(config.isEnabled());
    xo.setUserId(config.getUserId());
    xo.setRealmName(config.getRealmName());

    return xo;
  }

  @PUT
  @RequiresAuthentication
  @RequiresPermissions("astrum:settings:update")
  public void update(@NotNull @Valid final AnonymousSettingsXO anonymousXO) {
    AnonymousConfiguration configuration = anonymousManager.newConfiguration();
    configuration.setEnabled(anonymousXO.getEnabled());
    configuration.setRealmName(anonymousXO.getRealmName());
    configuration.setUserId(anonymousXO.getUserId());
    anonymousManager.setConfiguration(configuration);
  }
}
