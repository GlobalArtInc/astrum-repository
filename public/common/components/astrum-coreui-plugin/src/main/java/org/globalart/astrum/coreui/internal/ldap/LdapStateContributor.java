
package org.globalart.astrum.coreui.internal.ldap;

import java.util.Map;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.rapture.StateContributor;
import org.globalart.astrum.security.realm.RealmManager;

import com.google.common.collect.ImmutableMap;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

@Component
public class LdapStateContributor
    implements StateContributor
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String LDAP_REALM_NAME = "LdapRealm";

  public boolean featureFlag;

  public int mappedRoleQueryCharacterLimit;

  private final RealmManager realmManager;

  @Autowired
  public LdapStateContributor(
      @Value("${astrum.react.ldap:false}") final Boolean featureFlag,
      @Value("${astrum.ldap.mapped.role.query.character.limit:3}") final int mappedRoleQueryCharacterLimit,
      final RealmManager realmManager)
  {
    this.featureFlag = featureFlag;
    this.mappedRoleQueryCharacterLimit = mappedRoleQueryCharacterLimit;
    this.realmManager = realmManager;
  }

  @Override
  public Map<String, Object> getState() {
    return ImmutableMap.of(
        "astrum.react.ldap", featureFlag,
        "astrum.ldap.mapped.role.query.character.limit", mappedRoleQueryCharacterLimit,
        "ldapRealmEnabled", realmManager.isRealmEnabled(LDAP_REALM_NAME));
  }
}
