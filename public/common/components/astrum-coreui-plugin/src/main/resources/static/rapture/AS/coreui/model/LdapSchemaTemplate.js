
/*global Ext, AS*/

/**
 * LDAP Schema Template model.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.model.LdapSchemaTemplate', {
  extend: 'Ext.data.Model',
  fields: [
    {name: 'name', type: 'string', sortType: 'asUCText'},

    {name: 'userBaseDn', type: 'string'},
    {name: 'userSubtree', type: 'boolean'},
    {name: 'userObjectClass', type: 'string'},
    {name: 'userLdapFilter', type: 'string'},
    {name: 'userIdAttribute', type: 'string'},
    {name: 'userRealNameAttribute', type: 'string'},
    {name: 'userEmailAddressAttribute', type: 'string'},
    {name: 'userPasswordAttribute', type: 'string'},

    {name: 'ldapGroupsAsRoles', type: 'boolean'},
    {name: 'userMemberOfAttribute', type: 'string'},
    {name: 'groupType', type: 'string'},
    {name: 'groupBaseDn', type: 'string'},
    {name: 'groupSubtree', type: 'boolean'},
    {name: 'groupIdAttribute', type: 'string'},
    {name: 'groupMemberAttribute', type: 'string'},
    {name: 'groupMemberFormat', type: 'string'},
    {name: 'groupObjectClass', type: 'string'}
  ]
});
