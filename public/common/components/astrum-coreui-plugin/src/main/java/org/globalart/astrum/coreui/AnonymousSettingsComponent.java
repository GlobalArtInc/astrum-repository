
package org.globalart.astrum.coreui;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.validation.Valid;
import jakarta.validation.constraints.NotNull;

import org.globalart.astrum.extdirect.DirectComponent;
import org.globalart.astrum.extdirect.DirectComponentSupport;
import org.globalart.astrum.security.anonymous.AnonymousConfiguration;
import org.globalart.astrum.security.anonymous.AnonymousManager;

import com.codahale.metrics.annotation.ExceptionMetered;
import com.codahale.metrics.annotation.Timed;
import com.softwarementors.extjs.djn.config.annotations.DirectAction;
import com.softwarementors.extjs.djn.config.annotations.DirectMethod;
import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Anonymous Security Settings {@link DirectComponent}.
 */
@Component
@DirectAction(action = "coreui_AnonymousSettings")
public class AnonymousSettingsComponent
    extends DirectComponentSupport
{
  private final AnonymousManager anonymousManager;

  @Autowired
  public AnonymousSettingsComponent(final AnonymousManager anonymousManager) {
    this.anonymousManager = checkNotNull(anonymousManager);
  }

  /**
   * Retrieves anonymous security settings.
   *
   * @return anonymous security settings
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresPermissions("astrum:settings:read")
  public AnonymousSettingsXO read() {
    AnonymousConfiguration config = anonymousManager.getConfiguration();
    AnonymousSettingsXO xo = new AnonymousSettingsXO();
    xo.setEnabled(config.isEnabled());
    xo.setUserId(config.getUserId());
    xo.setRealmName(config.getRealmName());
    return xo;
  }

  /**
   * Updates anonymous security settings.
   *
   * @return updated anonymous security settings
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresAuthentication
  @RequiresPermissions("astrum:settings:update")
  public AnonymousSettingsXO update(@NotNull @Valid final AnonymousSettingsXO anonymousXO) {
    AnonymousConfiguration configuration = anonymousManager.newConfiguration();
    configuration.setEnabled(anonymousXO.getEnabled());
    configuration.setRealmName(anonymousXO.getRealmName());
    configuration.setUserId(anonymousXO.getUserId());
    anonymousManager.setConfiguration(configuration);
    return read();
  }
}
