
package org.globalart.astrum.api.rest.selfhosted.security.jwt;

import java.util.UUID;
import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.PUT;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.core.Response;

import org.globalart.astrum.rest.Resource;
import org.globalart.astrum.security.jwt.SecretStore;

import org.springframework.beans.factory.annotation.Autowired;
import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;
import static jakarta.ws.rs.core.Response.Status.OK;
import static jakarta.ws.rs.core.Response.status;
import static org.globalart.astrum.api.rest.selfhosted.security.jwt.JwtSecretApiResourceV1.PATH;
import static org.globalart.astrum.common.app.FeatureFlags.JWT_ENABLED;
import static org.globalart.astrum.rest.APIConstants.V1_API_PREFIX;

/**
 * REST API to reset the stored JWT secret.
 *
 * @since 3.38
 */
@ConditionalOnProperty(name = JWT_ENABLED, havingValue = "true")
@Consumes(APPLICATION_JSON)
@Produces(APPLICATION_JSON)
@Path(PATH)
@Component
public class JwtSecretApiResourceV1
    implements Resource, JwtSecretApiResourceDoc
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public static final String PATH = V1_API_PREFIX + "/security/jwt";

  private final SecretStore secretStore;

  @Autowired
  public JwtSecretApiResourceV1(final SecretStore secretStore) {
    this.secretStore = checkNotNull(secretStore);
  }

  @PUT
  @RequiresAuthentication
  @RequiresPermissions("astrum:settings:update")
  @Override
  public Response resetSecret() {
    String secret = UUID.randomUUID().toString();
    secretStore.setSecret(secret);
    return status(OK).build();
  }
}
