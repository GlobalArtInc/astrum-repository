package org.globalart.astrum.ldap.internal;

/**
 * LDAP server as exchanged with the classic UI over ExtDirect. Enumerations travel in the UI's spelling ({@code ldap},
 * {@code simple}, {@code DIGEST-MD5}, {@code static}); the bind password is never sent back.
 */
public class LdapServerXO
{
  private String id;

  private String name;

  private Integer order;

  private String url;

  private String protocol;

  private String host;

  private Integer port;

  private Boolean useTrustStore;

  private String searchBase;

  private String authScheme;

  private String authRealm;

  private String authUsername;

  private String authPassword;

  private Integer connectionTimeout;

  private Integer connectionRetryDelay;

  private Integer maxIncidentsCount;

  private String userBaseDn;

  private Boolean userSubtree;

  private String userObjectClass;

  private String userLdapFilter;

  private String userIdAttribute;

  private String userRealNameAttribute;

  private String userEmailAddressAttribute;

  private String userPasswordAttribute;

  private Boolean ldapGroupsAsRoles;

  private String groupType;

  private String groupBaseDn;

  private Boolean groupSubtree;

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

  public Integer getOrder() {
    return order;
  }

  public void setOrder(final Integer order) {
    this.order = order;
  }

  public String getUrl() {
    return url;
  }

  public void setUrl(final String url) {
    this.url = url;
  }

  public String getProtocol() {
    return protocol;
  }

  public void setProtocol(final String protocol) {
    this.protocol = protocol;
  }

  public String getHost() {
    return host;
  }

  public void setHost(final String host) {
    this.host = host;
  }

  public Integer getPort() {
    return port;
  }

  public void setPort(final Integer port) {
    this.port = port;
  }

  public Boolean getUseTrustStore() {
    return useTrustStore;
  }

  public void setUseTrustStore(final Boolean useTrustStore) {
    this.useTrustStore = useTrustStore;
  }

  public String getSearchBase() {
    return searchBase;
  }

  public void setSearchBase(final String searchBase) {
    this.searchBase = searchBase;
  }

  public String getAuthScheme() {
    return authScheme;
  }

  public void setAuthScheme(final String authScheme) {
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

  public String getAuthPassword() {
    return authPassword;
  }

  public void setAuthPassword(final String authPassword) {
    this.authPassword = authPassword;
  }

  public Integer getConnectionTimeout() {
    return connectionTimeout;
  }

  public void setConnectionTimeout(final Integer connectionTimeout) {
    this.connectionTimeout = connectionTimeout;
  }

  public Integer getConnectionRetryDelay() {
    return connectionRetryDelay;
  }

  public void setConnectionRetryDelay(final Integer connectionRetryDelay) {
    this.connectionRetryDelay = connectionRetryDelay;
  }

  public Integer getMaxIncidentsCount() {
    return maxIncidentsCount;
  }

  public void setMaxIncidentsCount(final Integer maxIncidentsCount) {
    this.maxIncidentsCount = maxIncidentsCount;
  }

  public String getUserBaseDn() {
    return userBaseDn;
  }

  public void setUserBaseDn(final String userBaseDn) {
    this.userBaseDn = userBaseDn;
  }

  public Boolean getUserSubtree() {
    return userSubtree;
  }

  public void setUserSubtree(final Boolean userSubtree) {
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

  public Boolean getLdapGroupsAsRoles() {
    return ldapGroupsAsRoles;
  }

  public void setLdapGroupsAsRoles(final Boolean ldapGroupsAsRoles) {
    this.ldapGroupsAsRoles = ldapGroupsAsRoles;
  }

  public String getGroupType() {
    return groupType;
  }

  public void setGroupType(final String groupType) {
    this.groupType = groupType;
  }

  public String getGroupBaseDn() {
    return groupBaseDn;
  }

  public void setGroupBaseDn(final String groupBaseDn) {
    this.groupBaseDn = groupBaseDn;
  }

  public Boolean getGroupSubtree() {
    return groupSubtree;
  }

  public void setGroupSubtree(final Boolean groupSubtree) {
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
}
