package org.globalart.astrum.ldap.internal;

/**
 * Preset user and group settings for common directory layouts. {@code groupType} is lower case, as both UIs expect.
 */
public class LdapSchemaTemplateXO
{
  private String name;

  private String userBaseDn;

  private boolean userSubtree;

  private String userObjectClass;

  private String userLdapFilter;

  private String userIdAttribute;

  private String userRealNameAttribute;

  private String userEmailAddressAttribute;

  private String userPasswordAttribute;

  private boolean ldapGroupsAsRoles;

  private String groupType;

  private String groupBaseDn;

  private boolean groupSubtree;

  private String groupObjectClass;

  private String groupIdAttribute;

  private String groupMemberAttribute;

  private String groupMemberFormat;

  private String userMemberOfAttribute;

  static LdapSchemaTemplateXO dynamic(
      final String name,
      final String userBaseDn,
      final String userObjectClass,
      final String userIdAttribute)
  {
    LdapSchemaTemplateXO template = user(name, userBaseDn, userObjectClass, userIdAttribute);
    template.groupType = LdapGroupType.DYNAMIC.uiValue();
    template.userMemberOfAttribute = "memberOf";
    return template;
  }

  static LdapSchemaTemplateXO posixStatic() {
    LdapSchemaTemplateXO template = user("Posix with Static Groups", "ou=people", "posixAccount", "uid");
    template.groupType = LdapGroupType.STATIC.uiValue();
    template.groupBaseDn = "ou=groups";
    template.groupObjectClass = "posixGroup";
    template.groupIdAttribute = "cn";
    template.groupMemberAttribute = "memberUid";
    template.groupMemberFormat = "${username}";
    return template;
  }

  private static LdapSchemaTemplateXO user(
      final String name,
      final String userBaseDn,
      final String userObjectClass,
      final String userIdAttribute)
  {
    LdapSchemaTemplateXO template = new LdapSchemaTemplateXO();
    template.name = name;
    template.userBaseDn = userBaseDn;
    template.userObjectClass = userObjectClass;
    template.userIdAttribute = userIdAttribute;
    template.userRealNameAttribute = "cn";
    template.userEmailAddressAttribute = "mail";
    template.ldapGroupsAsRoles = true;
    return template;
  }

  public String getName() {
    return name;
  }

  public String getUserBaseDn() {
    return userBaseDn;
  }

  public boolean isUserSubtree() {
    return userSubtree;
  }

  public String getUserObjectClass() {
    return userObjectClass;
  }

  public String getUserLdapFilter() {
    return userLdapFilter;
  }

  public String getUserIdAttribute() {
    return userIdAttribute;
  }

  public String getUserRealNameAttribute() {
    return userRealNameAttribute;
  }

  public String getUserEmailAddressAttribute() {
    return userEmailAddressAttribute;
  }

  public String getUserPasswordAttribute() {
    return userPasswordAttribute;
  }

  public boolean isLdapGroupsAsRoles() {
    return ldapGroupsAsRoles;
  }

  public String getGroupType() {
    return groupType;
  }

  public String getGroupBaseDn() {
    return groupBaseDn;
  }

  public boolean isGroupSubtree() {
    return groupSubtree;
  }

  public String getGroupObjectClass() {
    return groupObjectClass;
  }

  public String getGroupIdAttribute() {
    return groupIdAttribute;
  }

  public String getGroupMemberAttribute() {
    return groupMemberAttribute;
  }

  public String getGroupMemberFormat() {
    return groupMemberFormat;
  }

  public String getUserMemberOfAttribute() {
    return userMemberOfAttribute;
  }
}
