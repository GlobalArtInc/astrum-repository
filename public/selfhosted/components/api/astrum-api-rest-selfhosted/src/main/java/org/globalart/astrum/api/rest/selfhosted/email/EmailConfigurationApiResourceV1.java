
package org.globalart.astrum.api.rest.selfhosted.email;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.email.EmailManager;
import org.globalart.astrum.validation.ssrf.AntiSsrfService;

import static org.globalart.astrum.api.rest.selfhosted.email.EmailConfigurationApiResourceV1.RESOURCE_URI;
import static org.globalart.astrum.rest.APIConstants.V1_API_PREFIX;
import org.springframework.stereotype.Component;

/**
 * v1 endpoint for email configuration REST API
 *
 * @since 3.25
 */
@Component
@Path(RESOURCE_URI)
public class EmailConfigurationApiResourceV1
    extends EmailConfigurationApiResource
{
  static final String RESOURCE_URI = V1_API_PREFIX + "/email";

  @Autowired
  public EmailConfigurationApiResourceV1(final EmailManager emailManager, final AntiSsrfService antiSsrfService) {
    super(emailManager, antiSsrfService);
  }
}
