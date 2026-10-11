
package org.globalart.astrum.api.rest.selfhosted.email;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.validation.Valid;
import jakarta.validation.ValidationException;
import jakarta.validation.constraints.NotNull;
import jakarta.ws.rs.DELETE;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.POST;
import jakarta.ws.rs.PUT;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.core.Response;

import org.globalart.astrum.api.rest.selfhosted.email.model.ApiEmailConfiguration;
import org.globalart.astrum.api.rest.selfhosted.email.model.ApiEmailValidation;
import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.email.EmailConfiguration;
import org.globalart.astrum.email.EmailManager;
import org.globalart.astrum.rest.Resource;
import org.globalart.astrum.rest.ValidationErrorsException;
import org.globalart.astrum.validation.Validate;
import org.globalart.astrum.validation.ssrf.AntiSsrfService;

import org.apache.commons.lang3.StringUtils;
import org.apache.commons.lang3.exception.ExceptionUtils;
import org.apache.commons.mail2.core.EmailException;
import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;
import static jakarta.ws.rs.core.Response.Status.BAD_REQUEST;

@Produces(APPLICATION_JSON)
public class EmailConfigurationApiResource
    implements Resource, EmailConfigurationApiResourceDoc
{
  private static final Logger log = LoggerFactory.getLogger(EmailConfigurationApiResource.class);

  private final EmailManager emailManager;

  private final AntiSsrfService antiSsrfService;

  @Autowired
  public EmailConfigurationApiResource(EmailManager emailManager, AntiSsrfService antiSsrfService) {
    this.emailManager = emailManager;
    this.antiSsrfService = antiSsrfService;
  }

  @GET
  @RequiresPermissions("astrum:settings:read")
  public ApiEmailConfiguration getEmailConfiguration() {
    return convert(emailManager.getConfiguration());
  }

  @PUT
  @RequiresAuthentication
  @Validate
  @RequiresPermissions("astrum:settings:update")
  public void setEmailConfiguration(@NotNull @Valid final ApiEmailConfiguration apiEmailConfiguration) {
    try {
      antiSsrfService.validateHostWithoutCache(apiEmailConfiguration.getHost());
    }
    catch (ValidationException e) {
      // a host the SSRF protection refuses is invalid input, not a server error
      throw new ValidationErrorsException("host", e.getMessage() != null ? e.getMessage() : "Invalid host");
    }
    emailManager.setConfiguration(convert(apiEmailConfiguration), apiEmailConfiguration.getPassword());
  }

  @POST
  @Path("/verify")
  @RequiresAuthentication
  @RequiresPermissions("astrum:settings:update")
  public Response testEmailConfiguration(@NotNull String verificationAddress) {
    EmailConfiguration emailConfiguration = emailManager.getConfiguration();

    if (emailConfiguration == null) {
      return Response.status(BAD_REQUEST)
          .entity(new ApiEmailValidation(false, "Email Settings are not yet configured"))
          .type(APPLICATION_JSON)
          .build();
    }

    try {
      emailManager.sendVerification(emailConfiguration, verificationAddress);
      return Response.ok(new ApiEmailValidation(true), APPLICATION_JSON)
          .build();
    }
    catch (EmailException e) {
      log.warn("Unable to send verification", e);
      Throwable rootCause = ExceptionUtils.getRootCause(e);
      if (rootCause == null) {
        rootCause = e;
      }
      String rawMessage = rootCause.getMessage();
      String exceptionMessage = rawMessage != null
          ? rawMessage.replace(rootCause.getClass().getName() + ": ", "")
          : rootCause.getClass().getSimpleName();

      return Response.status(BAD_REQUEST)
          .entity(new ApiEmailValidation(false, exceptionMessage))
          .type(APPLICATION_JSON)
          .build();
    }
  }

  @DELETE
  @RequiresAuthentication
  @RequiresPermissions("astrum:settings:update")
  public void deleteEmailConfiguration() {
    emailManager.setConfiguration(emailManager.newConfiguration(), Strings2.EMPTY);
  }

  private EmailConfiguration convert(ApiEmailConfiguration apiEmailConfiguration) {

    EmailConfiguration emailConfiguration = emailManager.newConfiguration();
    emailConfiguration.setEnabled(apiEmailConfiguration.isEnabled());
    emailConfiguration.setHost(apiEmailConfiguration.getHost());
    emailConfiguration.setPort(apiEmailConfiguration.getPort() == null ? 0 : apiEmailConfiguration.getPort());
    emailConfiguration.setAstrumTrustStoreEnabled(apiEmailConfiguration.isAstrumTrustStoreEnabled());

    if (StringUtils.isNotEmpty(apiEmailConfiguration.getUsername())) {
      emailConfiguration.setUsername(apiEmailConfiguration.getUsername());
    }
    else {
      emailConfiguration.setUsername("");
    }

    emailConfiguration.setFromAddress(apiEmailConfiguration.getFromAddress());
    emailConfiguration.setSubjectPrefix(apiEmailConfiguration.getSubjectPrefix());
    emailConfiguration.setStartTlsEnabled(apiEmailConfiguration.isStartTlsEnabled());
    emailConfiguration.setStartTlsRequired(apiEmailConfiguration.isStartTlsRequired());
    emailConfiguration.setSslOnConnectEnabled(apiEmailConfiguration.isSslOnConnectEnabled());
    emailConfiguration.setSslCheckServerIdentityEnabled(apiEmailConfiguration.isSslServerIdentityCheckEnabled());
    return emailConfiguration;
  }

  private ApiEmailConfiguration convert(EmailConfiguration emailConfiguration) {
    if (emailConfiguration == null) {
      return new ApiEmailConfiguration();
    }

    ApiEmailConfiguration apiEmailConfiguration = new ApiEmailConfiguration();
    apiEmailConfiguration.setEnabled(emailConfiguration.isEnabled());
    apiEmailConfiguration.setHost(emailConfiguration.getHost());
    apiEmailConfiguration.setPort(emailConfiguration.getPort());
    apiEmailConfiguration.setAstrumTrustStoreEnabled(emailConfiguration.isAstrumTrustStoreEnabled());
    apiEmailConfiguration.setUsername(emailConfiguration.getUsername());
    apiEmailConfiguration.setPassword(null);
    apiEmailConfiguration.setFromAddress(emailConfiguration.getFromAddress());
    apiEmailConfiguration.setSubjectPrefix(emailConfiguration.getSubjectPrefix());
    apiEmailConfiguration.setStartTlsEnabled(emailConfiguration.isStartTlsEnabled());
    apiEmailConfiguration.setStartTlsRequired(emailConfiguration.isStartTlsRequired());
    apiEmailConfiguration.setSslOnConnectEnabled(emailConfiguration.isSslOnConnectEnabled());
    apiEmailConfiguration.setSslServerIdentityCheckEnabled(emailConfiguration.isSslCheckServerIdentityEnabled());
    return apiEmailConfiguration;
  }
}
