
package org.globalart.astrum.internal.security;

import org.globalart.astrum.security.config.MemorySecurityConfiguration;
import org.globalart.astrum.security.config.SecurityContributor;
import org.globalart.astrum.security.config.SecurityContributorSupport;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

import static org.apache.commons.lang3.StringUtils.capitalize;
import static org.globalart.astrum.security.Roles.ADMIN_ROLE_ID;
import static org.globalart.astrum.security.Roles.ANONYMOUS_ROLE_ID;

/**
 * Default Astrum security configuration.
 *
 * @since 3.0
 */
@Component
public class AstrumSecurityContributor
    extends SecurityContributorSupport
    implements SecurityContributor
{
  private static final String SETTINGS_DOMAIN = "settings";

  private static final String BUNDLES_DOMAIN = "bundles";

  private static final String SEARCH_DOMAIN = "search";

  private static final String APIKEY_DOMAIN = "apikey";

  public static final String AS_ALL_PRIV_ID = "as-all";

  public static final String AS_ALL_PRIV_DESCRIPTION = "All permissions";

  public static final String AS_ALL_PATTERN = "astrum:*";

  public static final String AS_SETTINGS_ALL_PRIV_ID = "as-settings-all";

  public static final String AS_SETTINGS_READ_PRIV_ID = "as-settings-read";

  public static final String AS_SETTINGS_UPDATE_PRIV_ID = "as-settings-update";

  public static final String AS_BUNDLES_ALL_PRIV_ID = "as-bundles-all";

  public static final String AS_BUNDLES_READ_PRIV_ID = "as-bundles-read";

  public static final String AS_SEARCH_READ_PRIV_ID = "as-search-read";

  public static final String AS_APIKEY_ALL_PRIV_ID = "as-apikey-all";

  public static final String AS_APIKEY_ALL_DESCRIPTION = ALL_DESCRIPTION_BASE + "APIKey";

  public static final String AS_PRIVILEGE_DOMAIN = "privileges";

  public static final String AS_PRIVILEGE_PRIV_ID_PREFIX = "as-privileges";

  public static final String AS_ROLE_DOMAIN = "roles";

  public static final String AS_ROLE_PRIV_ID_PREFIX = "as-roles";

  public static final String AS_USER_DOMAIN = "users";

  public static final String AS_USER_PRIV_ID_PREFIX = "as-users";

  public static final String AS_USERCHANGEPW_DOMAIN = "userschangepw";

  public static final String AS_USERCHANGEPW_PRIV_ID = "as-userschangepw";

  public static final String AS_USERCHANGEPW_PRIV_DESCRIPTION = "Change password permission";

  public static final String AS_UPLOAD_DOMAIN = "component";

  public static final String AS_UPLOAD_PRIV_ID = "as-component-upload";

  public static final String AS_UPLOAD_PRIV_DESCRIPTION = "Upload component permission";

  public static final String AS_ADMIN_ROLE_DESCRIPTION = "Administrator Role";

  public static final String AS_ANON_ROLE_DESCRIPTION = "Anonymous Role";

  public static final String AS_HEALTHCHECK_READ_PRIV_ID = "as-healthcheck-read";

  public static final String AS_REPO_VIEW_ALL_BROWSE_PRIV_ID = "as-repository-view-*-*-browse";

  public static final String AS_REPO_VIEW_ALL_READ_PRIV_ID = "as-repository-view-*-*-read";

  private final boolean anonymousRoleEnabled;

  @Autowired
  public AstrumSecurityContributor(
      @Value("${astrum.security.default.anonymous:true}") final boolean anonymousRoleEnabled)
  {
    this.anonymousRoleEnabled = anonymousRoleEnabled;
  }

  @Override
  public MemorySecurityConfiguration getContribution() {
    MemorySecurityConfiguration configuration = new MemorySecurityConfiguration();

    configuration.addPrivilege(createWildcardPrivilege(AS_ALL_PRIV_ID, AS_ALL_PRIV_DESCRIPTION, AS_ALL_PATTERN));

    configuration.addPrivilege(
        createApplicationPrivilege(AS_SETTINGS_ALL_PRIV_ID, ALL_DESCRIPTION_BASE + capitalize(SETTINGS_DOMAIN),
            SETTINGS_DOMAIN, ACTION_ALL));
    configuration.addPrivilege(
        createApplicationPrivilege(AS_SETTINGS_READ_PRIV_ID, READ_DESCRIPTION_BASE + capitalize(SETTINGS_DOMAIN),
            SETTINGS_DOMAIN, ACTION_READ));
    configuration.addPrivilege(
        createApplicationPrivilege(AS_SETTINGS_UPDATE_PRIV_ID, UPDATE_DESCRIPTION_BASE + capitalize(SETTINGS_DOMAIN),
            SETTINGS_DOMAIN, ACTION_UPDATE));

    configuration.addPrivilege(
        createApplicationPrivilege(AS_BUNDLES_ALL_PRIV_ID, ALL_DESCRIPTION_BASE + capitalize(BUNDLES_DOMAIN),
            BUNDLES_DOMAIN, ACTION_ALL));
    configuration.addPrivilege(
        createApplicationPrivilege(AS_BUNDLES_READ_PRIV_ID, READ_DESCRIPTION_BASE + capitalize(BUNDLES_DOMAIN),
            BUNDLES_DOMAIN, ACTION_READ));

    configuration.addPrivilege(
        createApplicationPrivilege(AS_SEARCH_READ_PRIV_ID, READ_DESCRIPTION_BASE + capitalize(SEARCH_DOMAIN),
            SEARCH_DOMAIN, ACTION_READ));

    configuration.addPrivilege(
        createApplicationPrivilege(AS_APIKEY_ALL_PRIV_ID, AS_APIKEY_ALL_DESCRIPTION, APIKEY_DOMAIN, ACTION_ALL));

    createCrudAndAllApplicationPrivileges(AS_PRIVILEGE_PRIV_ID_PREFIX, AS_PRIVILEGE_DOMAIN)
        .forEach(configuration::addPrivilege);
    createCrudAndAllApplicationPrivileges(AS_ROLE_PRIV_ID_PREFIX, AS_ROLE_DOMAIN).forEach(configuration::addPrivilege);
    createCrudAndAllApplicationPrivileges(AS_USER_PRIV_ID_PREFIX, AS_USER_DOMAIN).forEach(configuration::addPrivilege);

    // FIXME: Sort out what the use-case is for this distinct permission, consider astrum:users:change-password?
    configuration.addPrivilege(
        createApplicationPrivilege(AS_USERCHANGEPW_PRIV_ID, AS_USERCHANGEPW_PRIV_DESCRIPTION, AS_USERCHANGEPW_DOMAIN,
            ACTION_CREATE));

    configuration.addPrivilege(
        createApplicationPrivilege(AS_UPLOAD_PRIV_ID, AS_UPLOAD_PRIV_DESCRIPTION, AS_UPLOAD_DOMAIN,
            ACTION_CREATE_ONLY));

    configuration.addRole(createRole(ADMIN_ROLE_ID, ADMIN_ROLE_ID, AS_ADMIN_ROLE_DESCRIPTION, AS_ALL_PRIV_ID));
    if (anonymousRoleEnabled) {
      configuration.addRole(
          createRole(ANONYMOUS_ROLE_ID, ANONYMOUS_ROLE_ID, AS_ANON_ROLE_DESCRIPTION, AS_SEARCH_READ_PRIV_ID,
              AS_HEALTHCHECK_READ_PRIV_ID, AS_REPO_VIEW_ALL_BROWSE_PRIV_ID, AS_REPO_VIEW_ALL_READ_PRIV_ID));
    }
    return configuration;
  }
}
