package org.globalart.astrum.ldap.internal;

import java.util.LinkedHashSet;
import java.util.Set;
import java.util.stream.Collectors;

import org.globalart.astrum.common.Description;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.security.role.RoleIdentifier;
import org.globalart.astrum.security.user.AbstractReadOnlyUserManager;
import org.globalart.astrum.security.user.User;
import org.globalart.astrum.security.user.UserNotFoundException;
import org.globalart.astrum.security.user.UserNotFoundTransientException;
import org.globalart.astrum.security.user.UserSearchCriteria;
import org.globalart.astrum.security.user.UserStatus;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Read-only view of LDAP users. Their LDAP groups appear as roles of source {@value #SOURCE}.
 */
@Component
@Qualifier(LdapUserManager.SOURCE)
@Description("LDAP")
public class LdapUserManager
    extends AbstractReadOnlyUserManager
{
  public static final String SOURCE = "LDAP";

  private final LdapManager ldapManager;

  @Autowired
  public LdapUserManager(final EventManager eventManager, final LdapManager ldapManager) {
    super(eventManager);
    this.ldapManager = checkNotNull(ldapManager);
  }

  @Override
  public String getSource() {
    return SOURCE;
  }

  @Override
  public String getAuthenticationRealmName() {
    return LdapRealm.NAME;
  }

  @Override
  public Set<User> listUsers() {
    return toUsers(ldapManager.searchUsers(null, LdapManager.DEFAULT_SEARCH_LIMIT));
  }

  @Override
  public Set<String> listUserIds() {
    return ldapManager.searchUsers(null, LdapManager.DEFAULT_SEARCH_LIMIT).stream()
        .map(LdapUser::username)
        .collect(Collectors.toCollection(LinkedHashSet::new));
  }

  @Override
  public Set<User> searchUsers(final UserSearchCriteria criteria) {
    String source = criteria.getSource();
    if (source != null && !SOURCE.equalsIgnoreCase(source)) {
      return Set.of();
    }
    int limit = criteria.getLimit() != null && criteria.getLimit() > 0
        ? criteria.getLimit()
        : LdapManager.DEFAULT_SEARCH_LIMIT;
    return ldapManager.searchUsers(criteria.getUserId(), limit).stream()
        .map(this::toUser)
        .filter(user -> matchesCriteria(user.getUserId(), source, roleIds(user), criteria))
        .limit(limit)
        .collect(Collectors.toCollection(LinkedHashSet::new));
  }

  @Override
  public User getUser(final String userId) throws UserNotFoundException {
    try {
      return ldapManager.getUser(userId)
          .map(this::toUser)
          .orElseThrow(() -> new UserNotFoundException(userId));
    }
    catch (LdapServerException e) {
      // the user may well exist; do not let callers treat them as deleted
      throw new UserNotFoundTransientException(userId, e.getMessage(), e);
    }
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
  public boolean isConfigured() {
    return ldapManager.isConfigured();
  }

  private Set<User> toUsers(final Iterable<LdapUser> users) {
    Set<User> result = new LinkedHashSet<>();
    users.forEach(user -> result.add(toUser(user)));
    return result;
  }

  private User toUser(final LdapUser ldapUser) {
    User user = new User();
    user.setUserId(ldapUser.username());
    user.setName(ldapUser.realName());
    user.setEmailAddress(ldapUser.email());
    user.setSource(SOURCE);
    user.setStatus(UserStatus.active);
    user.setReadOnly(true);
    ldapUser.groups().forEach(group -> user.addRole(new RoleIdentifier(SOURCE, group)));
    return user;
  }

  private static Set<String> roleIds(final User user) {
    return user.getRoles().stream().map(RoleIdentifier::getRoleId).collect(Collectors.toSet());
  }
}
