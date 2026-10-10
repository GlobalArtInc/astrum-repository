package org.globalart.astrum.ldap.internal;

import java.util.ArrayList;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.Set;
import java.util.UUID;
import java.util.function.Consumer;
import java.util.function.Predicate;
import java.util.function.Supplier;

import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.ldap.internal.LdapConfigurationChangedEvent.Type;
import org.globalart.astrum.rapture.PasswordPlaceholder;
import org.globalart.astrum.rest.ValidationErrorsException;
import org.globalart.astrum.security.realm.RealmManager;

import com.google.common.net.InetAddresses;
import com.google.common.net.InternetDomainName;
import com.unboundid.ldap.sdk.DN;
import com.unboundid.ldap.sdk.LDAPException;
import org.apache.shiro.SecurityUtils;
import org.apache.shiro.UnavailableSecurityManagerException;
import org.apache.shiro.subject.Subject;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Strings.isNullOrEmpty;

/**
 * Creates, changes, orders and removes LDAP servers. Validation errors are reported with the REST field names.
 */
@Component
public class LdapConfigurationManager
{
  static final int MAX_CONNECTION_TIMEOUT = 3600;

  private final LdapConfigurationStore store;

  private final RealmManager realmManager;

  private final EventManager eventManager;

  @Autowired
  public LdapConfigurationManager(
      final LdapConfigurationStore store,
      final RealmManager realmManager,
      final EventManager eventManager)
  {
    this.store = checkNotNull(store);
    this.realmManager = checkNotNull(realmManager);
    this.eventManager = checkNotNull(eventManager);
  }

  public List<LdapServer> list() {
    return store.list();
  }

  public Optional<LdapServer> findById(final String id) {
    return isNullOrEmpty(id) ? Optional.empty() : find(server -> id.equals(server.getId()));
  }

  public Optional<LdapServer> findByName(final String name) {
    if (isNullOrEmpty(name)) {
      return Optional.empty();
    }
    Optional<LdapServer> exact = find(server -> name.equals(server.getName()));
    return exact.isPresent() ? exact : find(server -> name.equalsIgnoreCase(server.getName()));
  }

  /**
   * @param password the bind password; required unless the server binds anonymously
   */
  public synchronized LdapServer create(final LdapServer draft, final String password) {
    LdapServer server = normalize(draft);
    validate(server, null);
    if (server.getAuthScheme().requiresCredentials() && !isPassword(password)) {
      throw new ValidationErrorsException("authPassword", "Password is required");
    }
    server.setId(UUID.randomUUID().toString());
    server.setAuthPasswordId(server.getAuthScheme().requiresCredentials()
        ? store.encryptPassword(password, currentUserId())
        : null);

    List<LdapServer> servers = new ArrayList<>(store.list());
    servers.add(server);
    saveOrForget(servers, server.getAuthPasswordId());

    if (!realmManager.isRealmEnabled(LdapRealm.NAME)) {
      realmManager.enableRealm(LdapRealm.NAME);
    }
    eventManager.post(LdapConfigurationChangedEvent.of(Type.CREATED, server));
    return server.copy();
  }

  /**
   * @param password a new bind password; blank or the password placeholder keeps the stored one, as long as the
   *                 server is still reached the same way and with the same account
   */
  public synchronized LdapServer update(final String id, final LdapServer draft, final String password) {
    List<LdapServer> servers = new ArrayList<>(store.list());
    int index = indexOf(servers, id);
    if (index < 0) {
      throw new LdapServerNotFoundException(id);
    }
    LdapServer existing = servers.get(index);
    LdapServer server = normalize(draft);
    server.setId(existing.getId());
    validate(server, existing.getId());

    String obsoleteSecret = null;
    String newSecret = null;
    if (!server.getAuthScheme().requiresCredentials()) {
      server.setAuthPasswordId(null);
      obsoleteSecret = existing.getAuthPasswordId();
    }
    else if (isPassword(password)) {
      newSecret = store.encryptPassword(password, currentUserId());
      server.setAuthPasswordId(newSecret);
      obsoleteSecret = existing.getAuthPasswordId();
    }
    else if (existing.getAuthPasswordId() != null && sameBindTarget(existing, server)) {
      server.setAuthPasswordId(existing.getAuthPasswordId());
    }
    else {
      throw new ValidationErrorsException("authPassword", "Password is required");
    }

    servers.set(index, server);
    saveOrForget(servers, newSecret);
    store.removePassword(obsoleteSecret);
    eventManager.post(LdapConfigurationChangedEvent.of(Type.UPDATED, server));
    return server.copy();
  }

  public synchronized LdapServer delete(final String id) {
    List<LdapServer> servers = new ArrayList<>(store.list());
    int index = indexOf(servers, id);
    if (index < 0) {
      throw new LdapServerNotFoundException(id);
    }
    LdapServer removed = servers.remove(index);
    store.save(servers);
    store.removePassword(removed.getAuthPasswordId());
    eventManager.post(LdapConfigurationChangedEvent.of(Type.DELETED, removed));
    return removed;
  }

  /**
   * Puts the given servers first, in the given order; servers not listed keep their relative order after them.
   */
  public synchronized void changeOrder(final List<String> ids) {
    List<LdapServer> servers = store.list();
    Set<String> wanted = new LinkedHashSet<>(ids != null ? ids : List.of());
    ValidationErrorsException errors = new ValidationErrorsException();
    for (String id : wanted) {
      if (indexOf(servers, id) < 0) {
        errors.withError("order", "LDAP server '" + id + "' does not exist");
      }
    }
    if (errors.hasValidationErrors()) {
      throw errors;
    }
    List<LdapServer> ordered = new ArrayList<>();
    wanted.forEach(id -> ordered.add(servers.get(indexOf(servers, id))));
    servers.stream().filter(server -> !wanted.contains(server.getId())).forEach(ordered::add);
    store.save(ordered);
    eventManager.post(LdapConfigurationChangedEvent.reordered(ordered.stream().map(LdapServer::getName).toList()));
  }

  /**
   * The bind password to test a draft with: the one given, or the stored password of {@code existing} when the draft
   * still binds to the same server with the same account.
   *
   * @return {@code null} for an anonymous bind
   */
  public char[] bindPassword(final LdapServer draft, final String password, final Optional<LdapServer> existing) {
    if (draft.getAuthScheme() == null || !draft.getAuthScheme().requiresCredentials()) {
      return null;
    }
    if (isPassword(password)) {
      return password.toCharArray();
    }
    if (existing.isPresent() && existing.get().getAuthPasswordId() != null && sameBindTarget(existing.get(), draft)) {
      return store.password(existing.get());
    }
    throw new ValidationErrorsException("authPassword", "Password is required");
  }

  /**
   * Checks only what is needed to connect and bind, as the user and group settings may not be filled in yet.
   */
  public LdapServer validateConnection(final LdapServer draft) {
    LdapServer server = normalize(draft);
    ValidationErrorsException errors = new ValidationErrorsException();
    validateConnection(server, errors);
    if (errors.hasValidationErrors()) {
      throw errors;
    }
    return server;
  }

  public LdapServer validateAll(final LdapServer draft) {
    LdapServer server = normalize(draft);
    validate(server, server.getId());
    return server;
  }

  private void validate(final LdapServer server, final String ownId) {
    ValidationErrorsException errors = new ValidationErrorsException();
    if (required(errors, "name", "Name", server.getName())) {
      store.list().stream()
          .filter(other -> !other.getId().equals(ownId))
          .filter(other -> other.getName().equalsIgnoreCase(server.getName()))
          .findFirst()
          .ifPresent(other -> errors.withError("name",
              "An LDAP server named '" + other.getName() + "' already exists"));
    }
    validateConnection(server, errors);
    validateMapping(server, errors);
    if (errors.hasValidationErrors()) {
      throw errors;
    }
  }

  private static void validateConnection(final LdapServer server, final ValidationErrorsException errors) {
    if (server.getProtocol() == null) {
      errors.withError("protocol", "Protocol must be ldap or ldaps");
    }
    if (required(errors, "host", "Host", server.getHost()) && !isHost(server.getHost())) {
      errors.withError("host", "Host must be a host name or an IP address");
    }
    if (server.getPort() < 1 || server.getPort() > 65535) {
      errors.withError("port", "Port must be between 1 and 65535");
    }
    if (required(errors, "searchBase", "Search base", server.getSearchBase())) {
      dn(errors, "searchBase", "Search base", server.getSearchBase());
    }
    if (server.getAuthScheme() == null) {
      errors.withError("authScheme", "Authentication method must be none, simple, DIGEST-MD5 or CRAM-MD5");
    }
    else if (server.getAuthScheme().requiresCredentials()) {
      required(errors, "authUsername", "Username or DN", server.getAuthUsername());
    }
    if (server.getConnectionTimeoutSeconds() < 1 || server.getConnectionTimeoutSeconds() > MAX_CONNECTION_TIMEOUT) {
      errors.withError("connectionTimeoutSeconds",
          "Connection timeout must be between 1 and " + MAX_CONNECTION_TIMEOUT + " seconds");
    }
    if (server.getConnectionRetryDelaySeconds() < 0) {
      errors.withError("connectionRetryDelaySeconds", "Retry delay must not be negative");
    }
    if (server.getMaxIncidentsCount() < 0) {
      errors.withError("maxIncidentsCount", "Max incidents count must not be negative");
    }
  }

  private static void validateMapping(final LdapServer server, final ValidationErrorsException errors) {
    if (!isNullOrEmpty(server.getUserBaseDn())) {
      dn(errors, "userBaseDn", "User relative DN", server.getUserBaseDn());
    }
    required(errors, "userObjectClass", "User object class", server.getUserObjectClass());
    if (!isNullOrEmpty(server.getUserLdapFilter())) {
      try {
        LdapDirectory.parseFilter(server.getUserLdapFilter());
      }
      catch (LDAPException e) {
        errors.withError("userLdapFilter", "User filter is not a valid LDAP filter");
      }
    }
    required(errors, "userIdAttribute", "Username attribute", server.getUserIdAttribute());
    required(errors, "userRealNameAttribute", "Real name attribute", server.getUserRealNameAttribute());
    required(errors, "userEmailAddressAttribute", "Email attribute", server.getUserEmailAddressAttribute());
    if (!server.isLdapGroupsAsRoles()) {
      return;
    }
    if (server.getGroupType() == null) {
      errors.withError("groupType", "Group type must be static or dynamic");
    }
    else if (server.getGroupType() == LdapGroupType.STATIC) {
      if (!isNullOrEmpty(server.getGroupBaseDn())) {
        dn(errors, "groupBaseDn", "Group relative DN", server.getGroupBaseDn());
      }
      required(errors, "groupObjectClass", "Group object class", server.getGroupObjectClass());
      required(errors, "groupIdAttribute", "Group ID attribute", server.getGroupIdAttribute());
      required(errors, "groupMemberAttribute", "Group member attribute", server.getGroupMemberAttribute());
      required(errors, "groupMemberFormat", "Group member format", server.getGroupMemberFormat());
    }
    else {
      required(errors, "userMemberOfAttribute", "Group member of attribute", server.getUserMemberOfAttribute());
    }
  }

  private static boolean required(
      final ValidationErrorsException errors,
      final String field,
      final String label,
      final String value)
  {
    if (isNullOrEmpty(value)) {
      errors.withError(field, label + " is required");
      return false;
    }
    return true;
  }

  private static void dn(
      final ValidationErrorsException errors,
      final String field,
      final String label,
      final String value)
  {
    if (!DN.isValidDN(value)) {
      errors.withError(field, label + " is not a valid DN");
    }
  }

  static boolean isHost(final String host) {
    return InetAddresses.isInetAddress(host)
        || InetAddresses.isUriInetAddress(host)
        || InternetDomainName.isValid(host);
  }

  /**
   * Whether the stored bind password may be reused for {@code draft}: it would go to the same server, for the same
   * account.
   */
  static boolean sameBindTarget(final LdapServer existing, final LdapServer draft) {
    return existing.getProtocol() == draft.getProtocol()
        && existing.getHost() != null && existing.getHost().equalsIgnoreCase(draft.getHost())
        && existing.getPort() == draft.getPort()
        && existing.getAuthScheme() == draft.getAuthScheme()
        && Objects.equals(existing.getAuthUsername(), draft.getAuthUsername())
        && Objects.equals(existing.getAuthRealm(), draft.getAuthRealm());
  }

  static boolean isPassword(final String password) {
    return !isNullOrEmpty(password) && PasswordPlaceholder.isNot(password);
  }

  /**
   * Trims text, turns blanks into {@code null} and drops settings that do not apply.
   */
  static LdapServer normalize(final LdapServer draft) {
    LdapServer server = draft.copy();
    trim(server::getName, server::setName);
    trim(server::getHost, server::setHost);
    trim(server::getSearchBase, server::setSearchBase);
    trim(server::getAuthRealm, server::setAuthRealm);
    trim(server::getAuthUsername, server::setAuthUsername);
    trim(server::getUserBaseDn, server::setUserBaseDn);
    trim(server::getUserObjectClass, server::setUserObjectClass);
    trim(server::getUserLdapFilter, server::setUserLdapFilter);
    trim(server::getUserIdAttribute, server::setUserIdAttribute);
    trim(server::getUserRealNameAttribute, server::setUserRealNameAttribute);
    trim(server::getUserEmailAddressAttribute, server::setUserEmailAddressAttribute);
    trim(server::getUserPasswordAttribute, server::setUserPasswordAttribute);
    trim(server::getGroupBaseDn, server::setGroupBaseDn);
    trim(server::getGroupObjectClass, server::setGroupObjectClass);
    trim(server::getGroupIdAttribute, server::setGroupIdAttribute);
    trim(server::getGroupMemberAttribute, server::setGroupMemberAttribute);
    trim(server::getGroupMemberFormat, server::setGroupMemberFormat);
    trim(server::getUserMemberOfAttribute, server::setUserMemberOfAttribute);
    if (server.getProtocol() != LdapProtocol.LDAPS) {
      server.setUseTrustStore(false);
    }
    if (server.getAuthScheme() == LdapAuthScheme.NONE) {
      server.setAuthUsername(null);
    }
    if (server.getAuthScheme() == null || !server.getAuthScheme().usesRealm()) {
      server.setAuthRealm(null);
    }
    if (!server.isLdapGroupsAsRoles()) {
      server.setGroupType(null);
    }
    return server;
  }

  private static void trim(final Supplier<String> getter, final Consumer<String> setter) {
    String value = getter.get();
    if (value != null) {
      value = value.trim();
      setter.accept(value.isEmpty() ? null : value);
    }
  }

  private void saveOrForget(final List<LdapServer> servers, final String newSecret) {
    try {
      store.save(servers);
    }
    catch (RuntimeException e) {
      store.removePassword(newSecret);
      throw e;
    }
  }

  private Optional<LdapServer> find(final Predicate<LdapServer> predicate) {
    return store.list().stream().filter(predicate).findFirst();
  }

  private static int indexOf(final List<LdapServer> servers, final String id) {
    for (int i = 0; i < servers.size(); i++) {
      if (servers.get(i).getId().equals(id)) {
        return i;
      }
    }
    return -1;
  }

  private static String currentUserId() {
    try {
      Subject subject = SecurityUtils.getSubject();
      return subject.getPrincipal() != null ? subject.getPrincipal().toString() : null;
    }
    catch (UnavailableSecurityManagerException e) {
      return null;
    }
  }
}
