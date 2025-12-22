
package org.globalart.astrum.security.user;

import java.util.Collection;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

import org.globalart.astrum.common.Description;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.security.SecuritySystem;
import org.globalart.astrum.security.config.CUserRoleMapping;
import org.globalart.astrum.security.config.SecurityConfigurationManager;

import com.google.common.collect.Sets;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.context.annotation.Lazy;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * This allows you to easily search for users that have added roles.
 * For example if your users generally come from an external Realm (LDAP) you could see which users have had roles
 * added to them.
 */
@Component
@Qualifier(ConfiguredUsersUserManager.SOURCE)
@Description("All Users with Roles")
public class ConfiguredUsersUserManager
    extends AbstractReadOnlyUserManager
{
  public static final String SOURCE = "allConfigured";

  private final SecuritySystem securitySystem;

  private final SecurityConfigurationManager configuration;

  @Autowired
  public ConfiguredUsersUserManager(
      final EventManager eventManager,
      @Lazy final SecuritySystem securitySystem,
      @Lazy final SecurityConfigurationManager configuration)
  {
    super(eventManager);
    this.securitySystem = checkNotNull(securitySystem);
    this.configuration = checkNotNull(configuration);
  }

  @Override
  public String getSource() {
    return SOURCE;
  }

  @Override
  public Set<User> listUsers() {
    Set<User> users = new HashSet<>();

    List<CUserRoleMapping> userRoleMappings = configuration.listUserRoleMappings();
    for (CUserRoleMapping userRoleMapping : userRoleMappings) {
      try {
        User user = securitySystem.getUser(userRoleMapping.getUserId(), userRoleMapping.getSource(),
            userRoleMapping.getRoles());
        if (user != null) {
          users.add(user);
        }
      }
      catch (UserNotFoundException e) {
        log.warn("User: '{}' of source: '{}' could not be found",
            userRoleMapping.getUserId(), userRoleMapping.getSource());
        log.debug("Most likely caused by a user role mapping that is invalid", e);
      }
      catch (NoSuchUserManagerException e) {
        log.warn("User: '{}' of source: '{}' could not be found",
            userRoleMapping.getUserId(), userRoleMapping.getSource(), e);
      }
    }

    return users;
  }

  @Override
  public Set<String> listUserIds() {
    Set<String> userIds = new HashSet<>();

    List<CUserRoleMapping> userRoleMappings = configuration.listUserRoleMappings();
    for (CUserRoleMapping userRoleMapping : userRoleMappings) {
      String userId = userRoleMapping.getUserId();
      if (!Strings2.isBlank(userId)) {
        userIds.add(userId);
      }
    }

    return userIds;
  }

  @Override
  public User getUser(final String userId) {
    // this resource will only list the users
    return null;
  }

  @Override
  public User getUser(final String userId, final Set<String> roleIds) {
    // this resource will only list the users
    return null;
  }

  @Override
  public Set<User> searchUsers(final UserSearchCriteria criteria) {
    // we only want to do this if the criteria is set to the source
    if (getSource().equals(criteria.getSource())) {
      return filterListInMemory(listUsers(), criteria);
    }
    else {
      return new HashSet<>();
    }
  }

  @Override
  protected boolean matchesCriteria(
      final String userId,
      final String userSource,
      final Collection<String> usersRoles,
      final UserSearchCriteria criteria)
  {
    // basically the same as the super, but we don't want to check the source
    if (!Strings2.isBlank(criteria.getUserId())
        && !userId.toLowerCase().startsWith(criteria.getUserId().toLowerCase())) {
      return false;
    }

    if (criteria.getOneOfRoleIds() != null && !criteria.getOneOfRoleIds().isEmpty()) {
      Set<String> userRoles = new HashSet<>();
      if (usersRoles != null) {
        userRoles.addAll(usersRoles);
      }

      // check the intersection of the roles
      if (Sets.intersection(criteria.getOneOfRoleIds(), userRoles).isEmpty()) {
        return false;
      }
    }

    return true;
  }

  @Override
  public String getAuthenticationRealmName() {
    return null;
  }

  @Override
  public boolean isConfigured() {
    throw new UnsupportedOperationException("Not supported yet.");
  }
}
