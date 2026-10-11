
package org.globalart.astrum.coreui;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.validation.Valid;
import jakarta.validation.constraints.NotNull;
import jakarta.validation.constraints.Email;

import org.globalart.astrum.email.EmailConfiguration;
import org.globalart.astrum.email.EmailManager;
import org.globalart.astrum.extdirect.DirectComponent;
import org.globalart.astrum.extdirect.DirectComponentSupport;
import org.globalart.astrum.rapture.PasswordPlaceholder;
import org.globalart.astrum.validation.Validate;

import com.codahale.metrics.annotation.ExceptionMetered;
import com.codahale.metrics.annotation.Timed;
import com.softwarementors.extjs.djn.config.annotations.DirectAction;
import com.softwarementors.extjs.djn.config.annotations.DirectMethod;
import org.apache.commons.mail2.core.EmailException;
import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Email {@link DirectComponent}.
 *
 * @since 3.0
 */
/**
 * Email server settings.
 *
 * The SMTP server is set by administrators and usually sits in the local network or on the same host, so its address
 * is not subject to SSRF protection.
 */
@Component
@DirectAction(action = "coreui_Email")
public class EmailComponent
    extends DirectComponentSupport
{
  private final EmailManager emailManager;

  @Autowired
  public EmailComponent(final EmailManager emailManager) {
    this.emailManager = checkNotNull(emailManager);
  }

  /**
   * Returns current configuration.
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresPermissions("astrum:settings:read")
  public EmailConfigurationXO read() {
    return convert(emailManager.getConfiguration());
  }

  EmailConfigurationXO convert(final EmailConfiguration value) {
    return new EmailConfigurationXO(
        value.isEnabled(),
        value.getHost(),
        value.getPort(),
        value.getUsername(),
        value.getPassword() != null ? PasswordPlaceholder.get() : null,
        value.getFromAddress(),
        value.getSubjectPrefix(),
        value.isStartTlsEnabled(),
        value.isStartTlsRequired(),
        value.isSslOnConnectEnabled(),
        value.isSslCheckServerIdentityEnabled(),
        value.isAstrumTrustStoreEnabled());
  }

  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresAuthentication
  @RequiresPermissions("astrum:settings:update")
  @Validate
  public EmailConfigurationXO update(@NotNull @Valid final EmailConfigurationXO configuration) {
    emailManager.setConfiguration(convert(configuration), configuration.getPassword());
    return read();
  }

  EmailConfiguration convert(final EmailConfigurationXO value) {
    EmailConfiguration emailConfiguration = emailManager.newConfiguration();
    emailConfiguration.setEnabled(value.isEnabled());
    emailConfiguration.setHost(value.getHost());
    emailConfiguration.setPort(value.getPort());
    emailConfiguration.setUsername(value.getUsername());
    emailConfiguration.setFromAddress(value.getFromAddress());
    emailConfiguration.setSubjectPrefix(value.getSubjectPrefix());
    emailConfiguration.setStartTlsEnabled(value.isStartTlsEnabled());
    emailConfiguration.setStartTlsRequired(value.isStartTlsRequired());
    emailConfiguration.setSslOnConnectEnabled(value.isSslOnConnectEnabled());
    emailConfiguration.setSslCheckServerIdentityEnabled(value.isSslCheckServerIdentityEnabled());
    emailConfiguration.setAstrumTrustStoreEnabled(value.isAstrumTrustStoreEnabled());

    return emailConfiguration;
  }

  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresAuthentication
  @RequiresPermissions("astrum:settings:update")
  @Validate
  public void sendVerification(
      @NotNull @Valid final EmailConfigurationXO configuration,
      @NotNull @Email final String address) throws EmailException
  {
    emailManager.sendVerification(convert(configuration), configuration.getPassword(), address);
  }
}
