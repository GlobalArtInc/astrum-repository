package org.globalart.astrum.ldap.internal;

import java.security.NoSuchAlgorithmException;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.Set;
import java.util.TreeSet;
import java.util.stream.Stream;

import javax.net.SocketFactory;
import javax.net.ssl.SSLContext;

import jakarta.inject.Provider;

import org.globalart.astrum.ssl.TrustStore;

import com.unboundid.ldap.sdk.BindRequest;
import com.unboundid.ldap.sdk.CRAMMD5BindRequest;
import com.unboundid.ldap.sdk.DIGESTMD5BindRequest;
import com.unboundid.ldap.sdk.DN;
import com.unboundid.ldap.sdk.Filter;
import com.unboundid.ldap.sdk.LDAPConnection;
import com.unboundid.ldap.sdk.LDAPConnectionOptions;
import com.unboundid.ldap.sdk.LDAPException;
import com.unboundid.ldap.sdk.LDAPSearchException;
import com.unboundid.ldap.sdk.RDN;
import com.unboundid.ldap.sdk.ResultCode;
import com.unboundid.ldap.sdk.SearchRequest;
import com.unboundid.ldap.sdk.SearchResultEntry;
import com.unboundid.ldap.sdk.SearchScope;
import com.unboundid.ldap.sdk.SimpleBindRequest;
import com.unboundid.util.ssl.HostNameSSLSocketVerifier;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Strings.isNullOrEmpty;

/**
 * Talks to a single LDAP server. Every operation opens its own connection, binds with the configured system account
 * and closes the connection when done.
 */
@Component
public class LdapDirectory
{
  /**
   * Object classes treated as groups when membership comes from an attribute on the user entry.
   */
  static final List<String> GROUP_OBJECT_CLASSES =
      List.of("group", "groupOfNames", "groupOfUniqueNames", "groupOfURLs", "posixGroup");

  private static final String NO_ATTRIBUTES = "1.1";

  private static final Logger log = LoggerFactory.getLogger(LdapDirectory.class);

  public enum Outcome
  {
    AUTHENTICATED,
    INCORRECT_PASSWORD,
    USER_NOT_FOUND
  }

  public record Authentication(Outcome outcome, LdapUser user)
  {
  }

  private final Provider<TrustStore> trustStore;

  @Autowired
  public LdapDirectory(final Provider<TrustStore> trustStore) {
    this.trustStore = checkNotNull(trustStore);
  }

  /**
   * Connects, binds with the system account and checks that the search base exists.
   */
  public void verifyConnection(final LdapServer server, final char[] bindPassword) {
    try (LDAPConnection connection = connect(server, bindPassword)) {
      SearchRequest request = new SearchRequest(server.getSearchBase(), SearchScope.BASE,
          Filter.createPresenceFilter("objectClass"), NO_ATTRIBUTES);
      try {
        connection.search(request);
      }
      catch (LDAPSearchException e) {
        if (e.getResultCode() == ResultCode.NO_SUCH_OBJECT) {
          throw new LdapServerException("The search base '" + server.getSearchBase() + "' does not exist", e);
        }
        if (e.getResultCode() != ResultCode.INSUFFICIENT_ACCESS_RIGHTS) {
          throw searchFailed(server.getSearchBase(), e);
        }
      }
    }
  }

  public Optional<LdapUser> findUser(final LdapServer server, final char[] bindPassword, final String userId) {
    try (LDAPConnection connection = connect(server, bindPassword)) {
      return findUserEntry(connection, server, userId, false)
          .map(entry -> toUser(connection, server, entry));
    }
  }

  /**
   * @param prefix the start of the user id, or {@code null} for any user
   */
  public List<LdapUser> searchUsers(
      final LdapServer server,
      final char[] bindPassword,
      final String prefix,
      final int limit)
  {
    Filter idFilter = isNullOrEmpty(prefix)
        ? Filter.createPresenceFilter(server.getUserIdAttribute())
        : Filter.createSubstringFilter(server.getUserIdAttribute(), prefix, null, null);
    try (LDAPConnection connection = connect(server, bindPassword)) {
      List<LdapUser> users = new ArrayList<>();
      for (SearchResultEntry entry : search(connection, userBase(server), scope(server.isUserSubtree()),
          userFilter(server, idFilter), limit, userAttributes(server, false))) {
        if (entry.getAttributeValue(server.getUserIdAttribute()) != null) {
          users.add(toUser(connection, server, entry));
        }
      }
      return users;
    }
  }

  /**
   * Checks the password of a user, either by binding as that user or, when the server names a password attribute, by
   * comparing it with the attribute value. An empty password never authenticates.
   */
  public Authentication authenticate(
      final LdapServer server,
      final char[] bindPassword,
      final String userId,
      final char[] password)
  {
    try (LDAPConnection connection = connect(server, bindPassword)) {
      boolean compare = !isNullOrEmpty(server.getUserPasswordAttribute());
      Optional<SearchResultEntry> entry = findUserEntry(connection, server, userId, compare);
      if (entry.isEmpty()) {
        return new Authentication(Outcome.USER_NOT_FOUND, null);
      }
      if (password == null || password.length == 0) {
        return new Authentication(Outcome.INCORRECT_PASSWORD, null);
      }
      boolean valid = compare
          ? LdapPasswords.matches(password, entry.get().getAttributeValueBytes(server.getUserPasswordAttribute()))
          : bindsAs(server, entry.get().getDN(), password);
      if (!valid) {
        return new Authentication(Outcome.INCORRECT_PASSWORD, null);
      }
      return new Authentication(Outcome.AUTHENTICATED, toUser(connection, server, entry.get()));
    }
  }

  /**
   * @param query part of the group name, or {@code null} for any group
   * @return names of groups that can be mapped to roles; empty if the server does not map groups
   */
  public List<String> searchGroups(
      final LdapServer server,
      final char[] bindPassword,
      final String query,
      final int limit)
  {
    if (!mapsGroups(server)) {
      return List.of();
    }
    try (LDAPConnection connection = connect(server, bindPassword)) {
      return groups(connection, server, isNullOrEmpty(query) ? null
          : Filter.createSubstringFilter(groupNameAttribute(server), null, new String[]{query}, null), limit);
    }
  }

  public boolean groupExists(final LdapServer server, final char[] bindPassword, final String groupId) {
    if (!mapsGroups(server) || isNullOrEmpty(groupId)) {
      return false;
    }
    try (LDAPConnection connection = connect(server, bindPassword)) {
      return groups(connection, server, Filter.createEqualityFilter(groupNameAttribute(server), groupId), 10)
          .stream()
          .anyMatch(groupId::equalsIgnoreCase);
    }
  }

  static boolean mapsGroups(final LdapServer server) {
    return server.isLdapGroupsAsRoles() && server.getGroupType() != null;
  }

  static String userBase(final LdapServer server) {
    return relativeTo(server.getUserBaseDn(), server.getSearchBase());
  }

  static String groupBase(final LdapServer server) {
    return relativeTo(server.getGroupBaseDn(), server.getSearchBase());
  }

  /**
   * Parses a filter typed by an administrator; the surrounding parentheses are optional.
   */
  static Filter parseFilter(final String filter) throws LDAPException {
    String trimmed = filter.trim();
    return Filter.create(trimmed.startsWith("(") ? trimmed : "(" + trimmed + ")");
  }

  /**
   * @return the value of the first RDN of a DN such as {@code cn=developers,ou=groups,dc=example,dc=com}, or the value
   *     itself if it is not a DN
   */
  static String groupName(final String value) {
    try {
      RDN rdn = new DN(value).getRDN();
      if (rdn != null && rdn.getAttributeValues().length > 0) {
        return rdn.getAttributeValues()[0];
      }
    }
    catch (LDAPException e) {
      // not a DN
    }
    return value;
  }

  private static String relativeTo(final String relative, final String base) {
    return isNullOrEmpty(relative) ? base : relative + "," + base;
  }

  private static SearchScope scope(final boolean subtree) {
    return subtree ? SearchScope.SUB : SearchScope.ONE;
  }

  private LDAPConnection connect(final LdapServer server, final char[] bindPassword) {
    LDAPConnection connection = open(server);
    try {
      BindRequest request = systemBind(server, bindPassword);
      if (request != null) {
        connection.bind(request);
      }
      return connection;
    }
    catch (LDAPException e) {
      connection.close();
      if (e.getResultCode() == ResultCode.INVALID_CREDENTIALS) {
        throw new LdapServerException("The LDAP server rejected the username or password of the system account", e);
      }
      throw new LdapServerException("Could not authenticate to " + address(server) + ": " + e.getMessage(), e);
    }
  }

  private LDAPConnection open(final LdapServer server) {
    int timeout = timeoutMillis(server);
    LDAPConnectionOptions options = new LDAPConnectionOptions();
    options.setConnectTimeoutMillis(timeout);
    options.setResponseTimeoutMillis(timeout);
    options.setFollowReferrals(false);
    SocketFactory socketFactory = null;
    if (server.getProtocol() == LdapProtocol.LDAPS) {
      socketFactory = sslContext(server).getSocketFactory();
      options.setSSLSocketVerifier(new HostNameSSLSocketVerifier(true));
    }
    LDAPConnection connection = new LDAPConnection(socketFactory, options);
    try {
      connection.connect(host(server), server.getPort(), timeout);
      return connection;
    }
    catch (LDAPException e) {
      connection.close();
      throw new LdapServerException("Could not connect to " + address(server) + ": " + e.getMessage(), e);
    }
  }

  private static BindRequest systemBind(final LdapServer server, final char[] password) {
    LdapAuthScheme scheme = server.getAuthScheme() != null ? server.getAuthScheme() : LdapAuthScheme.NONE;
    if (!scheme.requiresCredentials()) {
      return null;
    }
    if (password == null || password.length == 0) {
      // an empty simple bind is an anonymous bind; never let it pass for the system account
      throw new LdapServerException("No password is set for the system account of " + address(server));
    }
    byte[] bytes = LdapPasswords.utf8(password);
    return switch (scheme) {
      case SIMPLE -> new SimpleBindRequest(server.getAuthUsername(), bytes);
      case DIGEST_MD5 -> new DIGESTMD5BindRequest(server.getAuthUsername(), null, bytes,
          isNullOrEmpty(server.getAuthRealm()) ? null : server.getAuthRealm());
      case CRAM_MD5 -> new CRAMMD5BindRequest(server.getAuthUsername(), bytes);
      case NONE -> null;
    };
  }

  /**
   * Binds as the user on a separate connection, so the searches that follow still run as the system account.
   */
  private boolean bindsAs(final LdapServer server, final String userDn, final char[] password) {
    try (LDAPConnection connection = open(server)) {
      byte[] bytes = LdapPasswords.utf8(password);
      try {
        connection.bind(new SimpleBindRequest(userDn, bytes));
        return true;
      }
      finally {
        Arrays.fill(bytes, (byte) 0);
      }
    }
    catch (LDAPException e) {
      if (!ResultCode.isConnectionUsable(e.getResultCode())) {
        throw new LdapServerException("Lost the connection to " + address(server) + ": " + e.getMessage(), e);
      }
      log.debug("Bind as {} failed: {}", userDn, e.getMessage());
      return false;
    }
  }

  private Optional<SearchResultEntry> findUserEntry(
      final LDAPConnection connection,
      final LdapServer server,
      final String userId,
      final boolean withPassword)
  {
    if (isNullOrEmpty(userId)) {
      return Optional.empty();
    }
    List<SearchResultEntry> entries = search(connection, userBase(server), scope(server.isUserSubtree()),
        userFilter(server, Filter.createEqualityFilter(server.getUserIdAttribute(), userId)), 2,
        userAttributes(server, withPassword));
    if (entries.size() > 1) {
      log.warn("More than one LDAP entry on {} matches user {}; using {}", server.getName(), userId,
          entries.get(0).getDN());
    }
    return entries.stream().findFirst();
  }

  private LdapUser toUser(final LDAPConnection connection, final LdapServer server, final SearchResultEntry entry) {
    String username = entry.getAttributeValue(server.getUserIdAttribute());
    return new LdapUser(
        username,
        entry.getDN(),
        entry.getAttributeValue(server.getUserRealNameAttribute()),
        entry.getAttributeValue(server.getUserEmailAddressAttribute()),
        membership(connection, server, entry, username));
  }

  private Set<String> membership(
      final LDAPConnection connection,
      final LdapServer server,
      final SearchResultEntry user,
      final String username)
  {
    if (!mapsGroups(server)) {
      return Set.of();
    }
    Set<String> groups = new TreeSet<>(String.CASE_INSENSITIVE_ORDER);
    if (server.getGroupType() == LdapGroupType.DYNAMIC) {
      String[] values = user.getAttributeValues(server.getUserMemberOfAttribute());
      if (values != null) {
        Arrays.stream(values).map(LdapDirectory::groupName).forEach(groups::add);
      }
      return groups;
    }
    String member = server.getGroupMemberFormat()
        .replace("${username}", username)
        .replace("${dn}", user.getDN());
    Filter filter = Filter.createANDFilter(
        Filter.createEqualityFilter("objectClass", server.getGroupObjectClass()),
        Filter.createEqualityFilter(server.getGroupMemberAttribute(), member));
    for (SearchResultEntry entry : search(connection, groupBase(server), scope(server.isGroupSubtree()), filter, 0,
        server.getGroupIdAttribute())) {
      String name = entry.getAttributeValue(server.getGroupIdAttribute());
      if (name != null) {
        groups.add(name);
      }
    }
    return groups;
  }

  private List<String> groups(
      final LDAPConnection connection,
      final LdapServer server,
      final Filter nameFilter,
      final int limit)
  {
    Set<String> names = new LinkedHashSet<>();
    if (server.getGroupType() == LdapGroupType.STATIC) {
      Filter filter = Filter.createEqualityFilter("objectClass", server.getGroupObjectClass());
      if (nameFilter != null) {
        filter = Filter.createANDFilter(filter, nameFilter);
      }
      for (SearchResultEntry entry : search(connection, groupBase(server), scope(server.isGroupSubtree()), filter,
          limit, server.getGroupIdAttribute())) {
        String[] values = entry.getAttributeValues(server.getGroupIdAttribute());
        if (values != null) {
          names.addAll(Arrays.asList(values));
        }
      }
    }
    else {
      Filter filter = Filter.createORFilter(GROUP_OBJECT_CLASSES.stream()
          .map(objectClass -> Filter.createEqualityFilter("objectClass", objectClass))
          .toList());
      if (nameFilter != null) {
        filter = Filter.createANDFilter(filter, nameFilter);
      }
      for (SearchResultEntry entry : search(connection, server.getSearchBase(), SearchScope.SUB, filter, limit,
          NO_ATTRIBUTES)) {
        names.add(groupName(entry.getDN()));
      }
    }
    return new ArrayList<>(names);
  }

  /**
   * Group names come from the group id attribute for static groups, and from the first RDN (normally {@code cn}) for
   * dynamic ones.
   */
  private static String groupNameAttribute(final LdapServer server) {
    return server.getGroupType() == LdapGroupType.STATIC ? server.getGroupIdAttribute() : "cn";
  }

  private static Filter userFilter(final LdapServer server, final Filter idFilter) {
    List<Filter> parts = new ArrayList<>();
    parts.add(Filter.createEqualityFilter("objectClass", server.getUserObjectClass()));
    parts.add(idFilter);
    if (!isNullOrEmpty(server.getUserLdapFilter())) {
      try {
        parts.add(parseFilter(server.getUserLdapFilter()));
      }
      catch (LDAPException e) {
        throw new LdapServerException("The user filter '" + server.getUserLdapFilter() + "' is not valid", e);
      }
    }
    return Filter.createANDFilter(parts);
  }

  private static String[] userAttributes(final LdapServer server, final boolean withPassword) {
    return Stream.of(
            server.getUserIdAttribute(),
            server.getUserRealNameAttribute(),
            server.getUserEmailAddressAttribute(),
            server.getGroupType() == LdapGroupType.DYNAMIC ? server.getUserMemberOfAttribute() : null,
            withPassword ? server.getUserPasswordAttribute() : null)
        .filter(Objects::nonNull)
        .distinct()
        .toArray(String[]::new);
  }

  private static List<SearchResultEntry> search(
      final LDAPConnection connection,
      final String base,
      final SearchScope scope,
      final Filter filter,
      final int limit,
      final String... attributes)
  {
    SearchRequest request = new SearchRequest(base, scope, filter, attributes);
    request.setSizeLimit(Math.max(limit, 0));
    try {
      return connection.search(request).getSearchEntries();
    }
    catch (LDAPSearchException e) {
      if (e.getResultCode() == ResultCode.SIZE_LIMIT_EXCEEDED) {
        return e.getSearchEntries();
      }
      if (e.getResultCode() == ResultCode.NO_SUCH_OBJECT) {
        log.debug("LDAP search base {} does not exist", base);
        return List.of();
      }
      throw searchFailed(base, e);
    }
  }

  private static LdapServerException searchFailed(final String base, final LDAPSearchException e) {
    return new LdapServerException("LDAP search under '" + base + "' failed: " + e.getMessage(), e);
  }

  private SSLContext sslContext(final LdapServer server) {
    if (server.isUseTrustStore()) {
      return trustStore.get().getSSLContext();
    }
    try {
      return SSLContext.getDefault();
    }
    catch (NoSuchAlgorithmException e) {
      throw new LdapServerException("TLS is not available", e);
    }
  }

  private static int timeoutMillis(final LdapServer server) {
    int seconds = server.getConnectionTimeoutSeconds() > 0
        ? server.getConnectionTimeoutSeconds()
        : LdapServer.DEFAULT_CONNECTION_TIMEOUT;
    return (int) Math.min(Integer.MAX_VALUE, seconds * 1000L);
  }

  /**
   * Hosts may be given as {@code [::1]}; the SDK wants the bare address.
   */
  static String host(final LdapServer server) {
    String host = server.getHost();
    if (host != null && host.startsWith("[") && host.endsWith("]")) {
      return host.substring(1, host.length() - 1);
    }
    return host;
  }

  private static String address(final LdapServer server) {
    String scheme = server.getProtocol() != null ? server.getProtocol().scheme() : "ldap";
    return scheme + "://" + server.getHost() + ":" + server.getPort();
  }
}
