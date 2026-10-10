package org.globalart.astrum.ldap.internal;

import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.annotation.JsonProperty.Access;

/**
 * LDAP server in the REST API. {@code authPassword} is write-only: it is accepted on create, update and verify, and
 * never returned.
 */
@JsonInclude(JsonInclude.Include.NON_NULL)
public class LdapServerApiXO
{
  private String id;

  private Integer order;

  private String name;

  private LdapProtocol protocol;

  private Boolean useTrustStore;

  private String host;

  private Integer port;

  private String searchBase;

  private LdapAuthScheme authScheme;

  private String authRealm;

  private String authUsername;

  @JsonProperty(access = Access.WRITE_ONLY)
  private String authPassword;

  private Integer connectionTimeoutSeconds;

  private Integer connectionRetryDelaySeconds;

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

  private LdapGroupType groupType;

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

  public Integer getOrder() {
    return order;
  }

  public void setOrder(final Integer order) {
    this.order = order;
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

  public Boolean getUseTrustStore() {
    return useTrustStore;
  }

  public void setUseTrustStore(final Boolean useTrustStore) {
    this.useTrustStore = useTrustStore;
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

  public String getAuthPassword() {
    return authPassword;
  }

  public void setAuthPassword(final String authPassword) {
    this.authPassword = authPassword;
  }

  public Integer getConnectionTimeoutSeconds() {
    return connectionTimeoutSeconds;
  }

  public void setConnectionTimeoutSeconds(final Integer connectionTimeoutSeconds) {
    this.connectionTimeoutSeconds = connectionTimeoutSeconds;
  }

  public Integer getConnectionRetryDelaySeconds() {
    return connectionRetryDelaySeconds;
  }

  public void setConnectionRetryDelaySeconds(final Integer connectionRetryDelaySeconds) {
    this.connectionRetryDelaySeconds = connectionRetryDelaySeconds;
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
