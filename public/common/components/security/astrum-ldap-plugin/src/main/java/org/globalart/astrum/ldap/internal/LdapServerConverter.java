package org.globalart.astrum.ldap.internal;

import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Base64;

import org.globalart.astrum.rest.ValidationErrorsException;

/**
 * Converts between the stored {@link LdapServer} and what the classic UI (ExtDirect) and the REST API exchange.
 * Missing numbers and flags fall back to the defaults of a new server.
 */
final class LdapServerConverter
{
  private LdapServerConverter() {
  }

  static LdapServer fromExtDirect(final LdapServerXO xo) {
    LdapServer server = new LdapServer();
    server.setId(xo.getId());
    server.setName(xo.getName());
    server.setProtocol(LdapProtocol.parse(xo.getProtocol()));
    server.setHost(xo.getHost());
    server.setPort(port(xo.getPort(), server.getProtocol()));
    server.setUseTrustStore(Boolean.TRUE.equals(xo.getUseTrustStore()));
    server.setSearchBase(xo.getSearchBase());
    server.setAuthScheme(LdapAuthScheme.parse(xo.getAuthScheme()));
    server.setAuthRealm(xo.getAuthRealm());
    server.setAuthUsername(xo.getAuthUsername());
    server.setConnectionTimeoutSeconds(or(xo.getConnectionTimeout(), LdapServer.DEFAULT_CONNECTION_TIMEOUT));
    server.setConnectionRetryDelaySeconds(or(xo.getConnectionRetryDelay(), LdapServer.DEFAULT_CONNECTION_RETRY_DELAY));
    server.setMaxIncidentsCount(or(xo.getMaxIncidentsCount(), LdapServer.DEFAULT_MAX_INCIDENTS));
    server.setUserBaseDn(xo.getUserBaseDn());
    server.setUserSubtree(Boolean.TRUE.equals(xo.getUserSubtree()));
    server.setUserObjectClass(xo.getUserObjectClass());
    server.setUserLdapFilter(xo.getUserLdapFilter());
    server.setUserIdAttribute(xo.getUserIdAttribute());
    server.setUserRealNameAttribute(xo.getUserRealNameAttribute());
    server.setUserEmailAddressAttribute(xo.getUserEmailAddressAttribute());
    server.setUserPasswordAttribute(xo.getUserPasswordAttribute());
    server.setGroupType(LdapGroupType.parse(xo.getGroupType()));
    server.setLdapGroupsAsRoles(groupsAsRoles(xo.getLdapGroupsAsRoles(), server.getGroupType()));
    server.setGroupBaseDn(xo.getGroupBaseDn());
    server.setGroupSubtree(Boolean.TRUE.equals(xo.getGroupSubtree()));
    server.setGroupObjectClass(xo.getGroupObjectClass());
    server.setGroupIdAttribute(xo.getGroupIdAttribute());
    server.setGroupMemberAttribute(xo.getGroupMemberAttribute());
    server.setGroupMemberFormat(xo.getGroupMemberFormat());
    server.setUserMemberOfAttribute(xo.getUserMemberOfAttribute());
    return server;
  }

  static LdapServerXO toExtDirect(final LdapServer server, final int order) {
    LdapServerXO xo = new LdapServerXO();
    xo.setId(server.getId());
    xo.setName(server.getName());
    xo.setOrder(order);
    xo.setUrl(server.getUrl());
    xo.setProtocol(server.getProtocol() != null ? server.getProtocol().scheme() : null);
    xo.setHost(server.getHost());
    xo.setPort(server.getPort());
    xo.setUseTrustStore(server.isUseTrustStore());
    xo.setSearchBase(server.getSearchBase());
    xo.setAuthScheme(server.getAuthScheme() != null ? server.getAuthScheme().uiValue() : null);
    xo.setAuthRealm(server.getAuthRealm());
    xo.setAuthUsername(server.getAuthUsername());
    xo.setConnectionTimeout(server.getConnectionTimeoutSeconds());
    xo.setConnectionRetryDelay(server.getConnectionRetryDelaySeconds());
    xo.setMaxIncidentsCount(server.getMaxIncidentsCount());
    xo.setUserBaseDn(server.getUserBaseDn());
    xo.setUserSubtree(server.isUserSubtree());
    xo.setUserObjectClass(server.getUserObjectClass());
    xo.setUserLdapFilter(server.getUserLdapFilter());
    xo.setUserIdAttribute(server.getUserIdAttribute());
    xo.setUserRealNameAttribute(server.getUserRealNameAttribute());
    xo.setUserEmailAddressAttribute(server.getUserEmailAddressAttribute());
    xo.setUserPasswordAttribute(server.getUserPasswordAttribute());
    xo.setLdapGroupsAsRoles(server.isLdapGroupsAsRoles());
    xo.setGroupType(server.getGroupType() != null ? server.getGroupType().uiValue() : null);
    xo.setGroupBaseDn(server.getGroupBaseDn());
    xo.setGroupSubtree(server.isGroupSubtree());
    xo.setGroupObjectClass(server.getGroupObjectClass());
    xo.setGroupIdAttribute(server.getGroupIdAttribute());
    xo.setGroupMemberAttribute(server.getGroupMemberAttribute());
    xo.setGroupMemberFormat(server.getGroupMemberFormat());
    xo.setUserMemberOfAttribute(server.getUserMemberOfAttribute());
    return xo;
  }

  static LdapServer fromApi(final LdapServerApiXO xo) {
    LdapServer server = new LdapServer();
    server.setId(xo.getId());
    server.setName(xo.getName());
    server.setProtocol(xo.getProtocol());
    server.setHost(xo.getHost());
    server.setPort(port(xo.getPort(), server.getProtocol()));
    server.setUseTrustStore(Boolean.TRUE.equals(xo.getUseTrustStore()));
    server.setSearchBase(xo.getSearchBase());
    server.setAuthScheme(xo.getAuthScheme());
    server.setAuthRealm(xo.getAuthRealm());
    server.setAuthUsername(xo.getAuthUsername());
    server.setConnectionTimeoutSeconds(or(xo.getConnectionTimeoutSeconds(), LdapServer.DEFAULT_CONNECTION_TIMEOUT));
    server.setConnectionRetryDelaySeconds(
        or(xo.getConnectionRetryDelaySeconds(), LdapServer.DEFAULT_CONNECTION_RETRY_DELAY));
    server.setMaxIncidentsCount(or(xo.getMaxIncidentsCount(), LdapServer.DEFAULT_MAX_INCIDENTS));
    server.setUserBaseDn(xo.getUserBaseDn());
    server.setUserSubtree(Boolean.TRUE.equals(xo.getUserSubtree()));
    server.setUserObjectClass(xo.getUserObjectClass());
    server.setUserLdapFilter(xo.getUserLdapFilter());
    server.setUserIdAttribute(xo.getUserIdAttribute());
    server.setUserRealNameAttribute(xo.getUserRealNameAttribute());
    server.setUserEmailAddressAttribute(xo.getUserEmailAddressAttribute());
    server.setUserPasswordAttribute(xo.getUserPasswordAttribute());
    server.setGroupType(xo.getGroupType());
    server.setLdapGroupsAsRoles(groupsAsRoles(xo.getLdapGroupsAsRoles(), server.getGroupType()));
    server.setGroupBaseDn(xo.getGroupBaseDn());
    server.setGroupSubtree(Boolean.TRUE.equals(xo.getGroupSubtree()));
    server.setGroupObjectClass(xo.getGroupObjectClass());
    server.setGroupIdAttribute(xo.getGroupIdAttribute());
    server.setGroupMemberAttribute(xo.getGroupMemberAttribute());
    server.setGroupMemberFormat(xo.getGroupMemberFormat());
    server.setUserMemberOfAttribute(xo.getUserMemberOfAttribute());
    return server;
  }

  static LdapServerApiXO toApi(final LdapServer server, final int order) {
    LdapServerApiXO xo = new LdapServerApiXO();
    xo.setId(server.getId());
    xo.setOrder(order);
    xo.setName(server.getName());
    xo.setProtocol(server.getProtocol());
    xo.setUseTrustStore(server.isUseTrustStore());
    xo.setHost(server.getHost());
    xo.setPort(server.getPort());
    xo.setSearchBase(server.getSearchBase());
    xo.setAuthScheme(server.getAuthScheme());
    xo.setAuthRealm(server.getAuthRealm());
    xo.setAuthUsername(server.getAuthUsername());
    xo.setConnectionTimeoutSeconds(server.getConnectionTimeoutSeconds());
    xo.setConnectionRetryDelaySeconds(server.getConnectionRetryDelaySeconds());
    xo.setMaxIncidentsCount(server.getMaxIncidentsCount());
    xo.setUserBaseDn(server.getUserBaseDn());
    xo.setUserSubtree(server.isUserSubtree());
    xo.setUserObjectClass(server.getUserObjectClass());
    xo.setUserLdapFilter(server.getUserLdapFilter());
    xo.setUserIdAttribute(server.getUserIdAttribute());
    xo.setUserRealNameAttribute(server.getUserRealNameAttribute());
    xo.setUserEmailAddressAttribute(server.getUserEmailAddressAttribute());
    xo.setUserPasswordAttribute(server.getUserPasswordAttribute());
    xo.setLdapGroupsAsRoles(server.isLdapGroupsAsRoles());
    xo.setGroupType(server.getGroupType());
    xo.setGroupBaseDn(server.getGroupBaseDn());
    xo.setGroupSubtree(server.isGroupSubtree());
    xo.setGroupObjectClass(server.getGroupObjectClass());
    xo.setGroupIdAttribute(server.getGroupIdAttribute());
    xo.setGroupMemberAttribute(server.getGroupMemberAttribute());
    xo.setGroupMemberFormat(server.getGroupMemberFormat());
    xo.setUserMemberOfAttribute(server.getUserMemberOfAttribute());
    return xo;
  }

  static LdapUserXO toUserXO(final LdapUser user) {
    LdapUserXO xo = new LdapUserXO();
    xo.setUsername(user.username());
    xo.setRealName(user.realName());
    xo.setEmail(user.email());
    xo.setMembership(new ArrayList<>(user.groups()));
    return xo;
  }

  /**
   * Decodes credentials sent as UTF-8 text in base64.
   */
  static String decodeBase64(final String field, final String value) {
    if (value == null) {
      return null;
    }
    try {
      return new String(Base64.getDecoder().decode(value.trim()), StandardCharsets.UTF_8);
    }
    catch (IllegalArgumentException e) {
      throw new ValidationErrorsException(field, "Not valid base64");
    }
  }

  private static int port(final Integer port, final LdapProtocol protocol) {
    if (port != null) {
      return port;
    }
    return protocol != null ? protocol.defaultPort() : 0;
  }

  private static int or(final Integer value, final int fallback) {
    return value != null ? value : fallback;
  }

  /**
   * Groups map to roles unless switched off; a client that leaves the flag out but names a group type wants them.
   */
  private static boolean groupsAsRoles(final Boolean flag, final LdapGroupType groupType) {
    return flag != null ? flag : groupType != null;
  }
}
