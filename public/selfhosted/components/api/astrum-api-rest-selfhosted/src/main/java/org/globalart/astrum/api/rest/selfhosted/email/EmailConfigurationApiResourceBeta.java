
package org.globalart.astrum.api.rest.selfhosted.email;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.email.EmailManager;
import org.globalart.astrum.validation.ssrf.AntiSsrfService;

import io.swagger.v3.oas.annotations.Hidden;

import static org.globalart.astrum.api.rest.selfhosted.email.EmailConfigurationApiResourceBeta.RESOURCE_URI;
import static org.globalart.astrum.rest.APIConstants.BETA_API_PREFIX;
import org.springframework.stereotype.Component;

/**
 * beta endpoint for email configuration REST API
 *
 * @since 3.25
 * @deprecated moving to {@link EmailConfigurationApiResourceV1}
 */
@Hidden
@Component
@Path(RESOURCE_URI)
@Deprecated
public class EmailConfigurationApiResourceBeta
    extends EmailConfigurationApiResource
{
  static final String RESOURCE_URI = BETA_API_PREFIX + "/email";

  @Autowired
  public EmailConfigurationApiResourceBeta(final EmailManager emailManager, final AntiSsrfService antiSsrfService) {
    super(emailManager, antiSsrfService);
  }
}
