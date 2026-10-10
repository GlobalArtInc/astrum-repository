package org.globalart.astrum.ldap.internal;

import com.fasterxml.jackson.annotation.JsonIgnore;
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;

/**
 * One configured LDAP server, as persisted. The bind password is held in the secrets store and only its id is kept
 * here. Servers are tried in the order they are stored.
 */
@JsonIgnoreProperties(ignoreUnknown = true)
public class LdapServer
{
  public static final int DEFAULT_CONNECTION_TIMEOUT = 30;

  public static final int DEFAULT_CONNECTION_RETRY_DELAY = 300;

  public static final int DEFAULT_MAX_INCIDENTS = 3;

  /**
   * Stable identifier; survives renames.
   */
  private String id;

  private String name;

  private LdapProtocol protocol;

  private String host;

  private int port;

  private boolean useTrustStore;

  private String searchBase;

  private LdapAuthScheme authScheme;

  private String authRealm;

  private String authUsername;

  /**
   * Id of the encrypted bind password in the secrets store.
   */
  private String authPasswordId;

  private int connectionTimeoutSeconds = DEFAULT_CONNECTION_TIMEOUT;

  private int connectionRetryDelaySeconds = DEFAULT_CONNECTION_RETRY_DELAY;

  private int maxIncidentsCount = DEFAULT_MAX_INCIDENTS;

  private String userBaseDn;

  private boolean userSubtree;

  private String userObjectClass;

  private String userLdapFilter;

  private String userIdAttribute;

  private String userRealNameAttribute;

  private String userEmailAddressAttribute;

  private String userPasswordAttribute;

  private boolean ldapGroupsAsRoles;

  private LdapGroupType groupType;

  private String groupBaseDn;

  private boolean groupSubtree;

  private String groupObjectClass;

  private String groupIdAttribute;

  private String groupMemberAttribute;

  private String groupMemberFormat;

  private String userMemberOfAttribute;

  public String getId() {
    return id;
  }

  public void setId(final String id) {
    this.id = id;
  }

  public String getName() {
    return name;
  }

  public void setName(final String name) {
    this.name = name;
  }

  public LdapProtocol getProtocol() {
    return protocol;
  }

  public void setProtocol(final LdapProtocol protocol) {
    this.protocol = protocol;
  }

  public String getHost() {
    return host;
  }

  public void setHost(final String host) {
    this.host = host;
  }

  public int getPort() {
    return port;
  }

  public void setPort(final int port) {
    this.port = port;
  }

  public boolean isUseTrustStore() {
    return useTrustStore;
  }

  public void setUseTrustStore(final boolean useTrustStore) {
    this.useTrustStore = useTrustStore;
  }

  public String getSearchBase() {
    return searchBase;
  }

  public void setSearchBase(final String searchBase) {
    this.searchBase = searchBase;
  }

  public LdapAuthScheme getAuthScheme() {
    return authScheme;
  }

  public void setAuthScheme(final LdapAuthScheme authScheme) {
    this.authScheme = authScheme;
  }

  public String getAuthRealm() {
    return authRealm;
  }

  public void setAuthRealm(final String authRealm) {
    this.authRealm = authRealm;
  }

  public String getAuthUsername() {
    return authUsername;
  }

  public void setAuthUsername(final String authUsername) {
    this.authUsername = authUsername;
  }

  public String getAuthPasswordId() {
    return authPasswordId;
  }

  public void setAuthPasswordId(final String authPasswordId) {
    this.authPasswordId = authPasswordId;
  }

  public int getConnectionTimeoutSeconds() {
    return connectionTimeoutSeconds;
  }

  public void setConnectionTimeoutSeconds(final int connectionTimeoutSeconds) {
    this.connectionTimeoutSeconds = connectionTimeoutSeconds;
  }

  public int getConnectionRetryDelaySeconds() {
    return connectionRetryDelaySeconds;
  }

  public void setConnectionRetryDelaySeconds(final int connectionRetryDelaySeconds) {
    this.connectionRetryDelaySeconds = connectionRetryDelaySeconds;
  }

  public int getMaxIncidentsCount() {
    return maxIncidentsCount;
  }

  public void setMaxIncidentsCount(final int maxIncidentsCount) {
    this.maxIncidentsCount = maxIncidentsCount;
  }

  public String getUserBaseDn() {
    return userBaseDn;
  }

  public void setUserBaseDn(final String userBaseDn) {
    this.userBaseDn = userBaseDn;
  }

  public boolean isUserSubtree() {
    return userSubtree;
  }

  public void setUserSubtree(final boolean userSubtree) {
    this.userSubtree = userSubtree;
  }

  public String getUserObjectClass() {
    return userObjectClass;
  }

  public void setUserObjectClass(final String userObjectClass) {
    this.userObjectClass = userObjectClass;
  }

  public String getUserLdapFilter() {
    return userLdapFilter;
  }

  public void setUserLdapFilter(final String userLdapFilter) {
    this.userLdapFilter = userLdapFilter;
  }

  public String getUserIdAttribute() {
    return userIdAttribute;
  }

  public void setUserIdAttribute(final String userIdAttribute) {
    this.userIdAttribute = userIdAttribute;
  }

  public String getUserRealNameAttribute() {
    return userRealNameAttribute;
  }

  public void setUserRealNameAttribute(final String userRealNameAttribute) {
    this.userRealNameAttribute = userRealNameAttribute;
  }

  public String getUserEmailAddressAttribute() {
    return userEmailAddressAttribute;
  }

  public void setUserEmailAddressAttribute(final String userEmailAddressAttribute) {
    this.userEmailAddressAttribute = userEmailAddressAttribute;
  }

  public String getUserPasswordAttribute() {
    return userPasswordAttribute;
  }

  public void setUserPasswordAttribute(final String userPasswordAttribute) {
    this.userPasswordAttribute = userPasswordAttribute;
  }

  public boolean isLdapGroupsAsRoles() {
    return ldapGroupsAsRoles;
  }

  public void setLdapGroupsAsRoles(final boolean ldapGroupsAsRoles) {
    this.ldapGroupsAsRoles = ldapGroupsAsRoles;
  }

  public LdapGroupType getGroupType() {
    return groupType;
  }

  public void setGroupType(final LdapGroupType groupType) {
    this.groupType = groupType;
  }

  public String getGroupBaseDn() {
    return groupBaseDn;
  }

  public void setGroupBaseDn(final String groupBaseDn) {
    this.groupBaseDn = groupBaseDn;
  }

  public boolean isGroupSubtree() {
    return groupSubtree;
  }

  public void setGroupSubtree(final boolean groupSubtree) {
    this.groupSubtree = groupSubtree;
  }

  public String getGroupObjectClass() {
    return groupObjectClass;
  }

  public void setGroupObjectClass(final String groupObjectClass) {
    this.groupObjectClass = groupObjectClass;
  }

  public String getGroupIdAttribute() {
    return groupIdAttribute;
  }

  public void setGroupIdAttribute(final String groupIdAttribute) {
    this.groupIdAttribute = groupIdAttribute;
  }

  public String getGroupMemberAttribute() {
    return groupMemberAttribute;
  }

  public void setGroupMemberAttribute(final String groupMemberAttribute) {
    this.groupMemberAttribute = groupMemberAttribute;
  }

  public String getGroupMemberFormat() {
    return groupMemberFormat;
  }

  public void setGroupMemberFormat(final String groupMemberFormat) {
    this.groupMemberFormat = groupMemberFormat;
  }

  public String getUserMemberOfAttribute() {
    return userMemberOfAttribute;
  }

  public void setUserMemberOfAttribute(final String userMemberOfAttribute) {
    this.userMemberOfAttribute = userMemberOfAttribute;
  }

  /**
   * The address shown in lists, e.g. {@code ldap://ldap.example.com:389/dc=example,dc=com}.
   */
  @JsonIgnore
  public String getUrl() {
    String scheme = protocol != null ? protocol.scheme() : "ldap";
    return scheme + "://" + host + ":" + port + "/" + (searchBase != null ? searchBase : "");
  }

  public LdapServer copy() {
    LdapServer copy = new LdapServer();
    copy.id = id;
    copy.name = name;
    copy.protocol = protocol;
    copy.host = host;
    copy.port = port;
    copy.useTrustStore = useTrustStore;
    copy.searchBase = searchBase;
    copy.authScheme = authScheme;
    copy.authRealm = authRealm;
    copy.authUsername = authUsername;
    copy.authPasswordId = authPasswordId;
    copy.connectionTimeoutSeconds = connectionTimeoutSeconds;
    copy.connectionRetryDelaySeconds = connectionRetryDelaySeconds;
    copy.maxIncidentsCount = maxIncidentsCount;
    copy.userBaseDn = userBaseDn;
    copy.userSubtree = userSubtree;
    copy.userObjectClass = userObjectClass;
    copy.userLdapFilter = userLdapFilter;
    copy.userIdAttribute = userIdAttribute;
    copy.userRealNameAttribute = userRealNameAttribute;
    copy.userEmailAddressAttribute = userEmailAddressAttribute;
    copy.userPasswordAttribute = userPasswordAttribute;
    copy.ldapGroupsAsRoles = ldapGroupsAsRoles;
    copy.groupType = groupType;
    copy.groupBaseDn = groupBaseDn;
    copy.groupSubtree = groupSubtree;
    copy.groupObjectClass = groupObjectClass;
    copy.groupIdAttribute = groupIdAttribute;
    copy.groupMemberAttribute = groupMemberAttribute;
    copy.groupMemberFormat = groupMemberFormat;
    copy.userMemberOfAttribute = userMemberOfAttribute;
    return copy;
  }

  @Override
  public String toString() {
    return "LdapServer{id=" + id + ", name=" + name + ", url=" + getUrl() + "}";
  }
}
