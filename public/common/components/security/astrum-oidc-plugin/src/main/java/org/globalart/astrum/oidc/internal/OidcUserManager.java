package org.globalart.astrum.oidc.internal;

import java.util.LinkedHashSet;
import java.util.Set;
import java.util.stream.Collectors;

import org.globalart.astrum.common.Description;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.security.role.RoleIdentifier;
import org.globalart.astrum.security.user.AbstractReadOnlyUserManager;
import org.globalart.astrum.security.user.User;
import org.globalart.astrum.security.user.UserNotFoundException;
import org.globalart.astrum.security.user.UserSearchCriteria;
import org.globalart.astrum.security.user.UserStatus;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Component
@Qualifier(OidcUserManager.SOURCE)
@Description("OpenID Connect")
public class OidcUserManager
    extends AbstractReadOnlyUserManager
{
  public static final String SOURCE = "OAuth2";

  private final OidcUserStore users;

  private final OidcConfigurationStore configurationStore;

  @Autowired
  public OidcUserManager(
      final EventManager eventManager,
      final OidcUserStore users,
      final OidcConfigurationStore configurationStore)
  {
    super(eventManager);
    this.users = checkNotNull(users);
    this.configurationStore = checkNotNull(configurationStore);
  }

  @Override
  public String getSource() {
    return SOURCE;
  }

  @Override
  public String getAuthenticationRealmName() {
    return OidcRealm.NAME;
  }

  @Override
  public Set<User> listUsers() {
    return users.list().stream().map(this::toUser).collect(Collectors.toCollection(LinkedHashSet::new));
  }

  @Override
  public Set<String> listUserIds() {
    return users.list().stream().map(OidcUser::userId).collect(Collectors.toCollection(LinkedHashSet::new));
  }

  @Override
  public Set<User> searchUsers(final UserSearchCriteria criteria) {
    Set<User> result = filterListInMemory(listUsers(), criteria);
    if (criteria.getLimit() != null && criteria.getLimit() > 0 && result.size() > criteria.getLimit()) {
      return result.stream().limit(criteria.getLimit()).collect(Collectors.toCollection(LinkedHashSet::new));
    }
    return result;
  }

  @Override
  public User getUser(final String userId) throws UserNotFoundException {
    return toUser(users.get(userId).orElseThrow(() -> new UserNotFoundException(userId)));
  }

  @Override
  public User getUser(final String userId, final Set<String> roleIds) throws UserNotFoundException {
    User user = getUser(userId);
    if (roleIds != null) {
      roleIds.forEach(roleId -> user.addRole(new RoleIdentifier(DEFAULT_SOURCE, roleId)));
    }
    return user;
  }

  @Override
  public User doDeleteUser(final String userId) throws UserNotFoundException {
    User user = getUser(userId);
    users.remove(userId);
    return user;
  }

  @Override
  public boolean isConfigured() {
    return configurationStore.isConfigured();
  }

  private User toUser(final OidcUser oidcUser) {
    User user = new User();
    user.setUserId(oidcUser.userId());
    user.setFirstName(oidcUser.firstName());
    user.setLastName(oidcUser.lastName());
    user.setEmailAddress(oidcUser.email());
    user.setSource(SOURCE);
    user.setStatus(UserStatus.active);
    user.setReadOnly(true);
    oidcUser.groups().forEach(group -> user.addRole(new RoleIdentifier(SOURCE, group)));
    return user;
  }
}
