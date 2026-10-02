
package org.globalart.astrum.api.rest.selfhosted.security.anonymous;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.rest.APIConstants;
import org.globalart.astrum.security.anonymous.AnonymousManager;
import org.globalart.astrum.security.internal.rest.SecurityApiConstants;

import io.swagger.v3.oas.annotations.Hidden;
import org.apache.shiro.mgt.RealmSecurityManager;

import static org.globalart.astrum.api.rest.selfhosted.security.anonymous.AnonymousAccessApiResourceBeta.RESOURCE_URI;
import org.springframework.stereotype.Component;

/**
 * @since 3.26
 * @deprecated beta prefix is being phased out, prefer starting new APIs with {@link APIConstants#V1_API_PREFIX} instead
 */
@Hidden
@Component
@Path(RESOURCE_URI)
@Deprecated
public class AnonymousAccessApiResourceBeta
    extends AnonymousAccessApiResource
{
  static final String RESOURCE_URI = SecurityApiConstants.BETA_RESOURCE_URI + "anonymous";

  @Autowired
  public AnonymousAccessApiResourceBeta(
      final AnonymousManager anonymousManager,
      final RealmSecurityManager realmSecurityManager)
  {
    super(anonymousManager, realmSecurityManager);
  }
}
