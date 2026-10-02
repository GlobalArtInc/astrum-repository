
package org.globalart.astrum.api.rest.selfhosted.security.anonymous;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.validation.Valid;
import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.PUT;
import jakarta.ws.rs.Produces;

import org.globalart.astrum.api.rest.selfhosted.security.anonymous.model.AnonymousAccessSettingsXO;
import org.globalart.astrum.rest.Resource;
import org.globalart.astrum.rest.ValidationErrorsException;
import org.globalart.astrum.security.anonymous.AnonymousConfiguration;
import org.globalart.astrum.security.anonymous.AnonymousManager;

import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.apache.shiro.mgt.RealmSecurityManager;
import org.apache.shiro.realm.Realm;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;

/**
 * @since 3.24
 */
@Consumes(APPLICATION_JSON)
@Produces(APPLICATION_JSON)
public class AnonymousAccessApiResource
    implements Resource, AnonymousAccessApiResourceDoc
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final AnonymousManager anonymousManager;

  private final RealmSecurityManager realmSecurityManager;

  @Autowired
  public AnonymousAccessApiResource(
      final AnonymousManager anonymousManager,
      final RealmSecurityManager realmSecurityManager)
  {
    this.anonymousManager = checkNotNull(anonymousManager);
    this.realmSecurityManager = checkNotNull(realmSecurityManager);
  }

  @GET
  @RequiresPermissions("astrum:settings:read")
  @Override
  public AnonymousAccessSettingsXO read() {
    return new AnonymousAccessSettingsXO(anonymousManager.getConfiguration());
  }

  @PUT
  @RequiresAuthentication
  @RequiresPermissions("astrum:settings:update")
  @Override
  public AnonymousAccessSettingsXO update(@Valid final AnonymousAccessSettingsXO anonymousXO) {
    Realm realm = validate(anonymousXO);
    AnonymousConfiguration configuration = anonymousManager.newConfiguration();
    configuration.setEnabled(anonymousXO.isEnabled());
    configuration.setUserId(anonymousXO.getUserId());
    configuration.setRealmName(realm.getName());
    anonymousManager.setConfiguration(configuration);
    return new AnonymousAccessSettingsXO(anonymousManager.getConfiguration());
  }

  Realm validate(AnonymousAccessSettingsXO settings) {
    return realmSecurityManager.getRealms()
        .stream()
        .filter(realm -> realm.getName().equals(settings.getRealmName()))
        .findFirst()
        .orElseThrow(() -> new ValidationErrorsException("realmName", "Realm does not exist"));
  }
}
