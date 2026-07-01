
/*global Ext, NX*/

/**
 * LDAP Server model.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.model.LdapServer', {
  extend: 'Ext.data.Model',
  fields: [
    {name: 'id', type: 'string'},
    {name: 'name', type: 'string', sortType: 'asUCText'},
    {name: 'order', type: 'int'},
    {name: 'url', type: 'string'},

    {name: 'protocol', type: 'auto' /*object*/},
    {name: 'host', type: 'string'},
    {name: 'port', type: 'int'},
    {name: 'useTrustStore', type: 'boolean'},
    {name: 'searchBase', type: 'string'},

    {name: 'authScheme', type: 'string'},
    {name: 'authRealm', type: 'string'},
    {name: 'authUsername', type: 'string'},
    {name: 'authPassword', type: 'string'},

    {name: 'connectionTimeout', type: 'int'},
    {name: 'connectionRetryDelay', type: 'int'},
    {name: 'maxIncidentsCount', type: 'int'},

    {name: 'userBaseDn', type: 'string'},
    {name: 'userSubtree', type: 'boolean'},
    {name: 'userObjectClass', type: 'string'},
    {name: 'userLdapFilter', type: 'string'},
    {name: 'userIdAttribute', type: 'string'},
    {name: 'userRealNameAttribute', type: 'string'},
    {name: 'userMemberOfAttribute', type: 'string'},
    {name: 'userEmailAddressAttribute', type: 'string'},
    {name: 'userPasswordAttribute', type: 'string'},

    {name: 'ldapGroupsAsRoles', type: 'boolean'},
    {name: 'groupType', type: 'string'},
    {name: 'groupBaseDn', type: 'string'},
    {name: 'groupSubtree', type: 'boolean'},
    {name: 'groupIdAttribute', type: 'string'},
    {name: 'groupMemberAttribute', type: 'string'},
    {name: 'groupMemberFormat', type: 'string'},
    {name: 'groupObjectClass', type: 'string'}
  ]
});
