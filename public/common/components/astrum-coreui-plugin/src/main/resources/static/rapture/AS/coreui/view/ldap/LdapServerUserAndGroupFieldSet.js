
/*global Ext, AS*/

/**
 * LDAP Server "User & Group" field set.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.ldap.LdapServerUserAndGroupFieldSet', {
  extend: 'Ext.panel.Panel',
  alias: 'widget.as-coreui-ldapserver-userandgroup-fieldset',
  requires: [
    'AS.I18n'
  ],

  defaults: {
    xtype: 'textfield',
    allowBlank: false
  },

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.items = [
      {
        xtype: 'combo',
        itemId: 'template',
        name: 'template',
        fieldLabel: AS.I18n.get('Ldap_LdapServerUserAndGroupFieldSet_Template_FieldLabel'),
        emptyText: AS.I18n.get('Ldap_LdapServerUserAndGroupFieldSet_Template_EmptyText'),
        editable: false,
        store: 'LdapSchemaTemplate',
        displayField: 'name',
        queryMode: 'local',
        listeners: {
          select: function (combo, selected) {
            var data = selected.getData();
            delete data.name;
            combo.up('form').getForm().setValues(data);
          }
        },
        allowBlank: true
      },

      //user
      {
        name: 'userBaseDn',
        fieldLabel: AS.I18n.get('Ldap_LdapServerUserAndGroupFieldSet_BaseDN_FieldLabel'),
        helpText: AS.I18n.get('Ldap_LdapServerUserAndGroupFieldSet_BaseDN_HelpText'),
        allowBlank: true
      },
      {
        xtype: 'checkbox',
        name: 'userSubtree',
        fieldLabel: AS.I18n.get('Ldap_LdapServerUserAndGroupFieldSet_UserSubtree_FieldLabel'),
        helpText: AS.I18n.get('Ldap_LdapServerUserAndGroupFieldSet_UserSubtree_HelpText')
      },
      {
        name: 'userObjectClass',
        fieldLabel: AS.I18n.get('Ldap_LdapServerUserAndGroupFieldSet_ObjectClass_FieldLabel'),
        helpText: AS.I18n.get('Ldap_LdapServerUserAndGroupFieldSet_ObjectClass_HelpText')
      },
      {
        name: 'userLdapFilter',
        fieldLabel: AS.I18n.get('Ldap_LdapServerUserAndGroupFieldSet_UserFilter_FieldLabel'),
        helpText: AS.I18n.get('Ldap_LdapServerUserAndGroupFieldSet_UserFilter_HelpText'),
        allowBlank: true
      },
      {
        name: 'userIdAttribute',
        fieldLabel: AS.I18n.get('Ldap_LdapServerUserAndGroupFieldSet_UserID_FieldLabel')
      },
      {
        name: 'userRealNameAttribute',
        fieldLabel: AS.I18n.get('Ldap_LdapServerUserAndGroupFieldSet_RealName_FieldLabel')
      },
      {
        name: 'userEmailAddressAttribute',
        fieldLabel: AS.I18n.get('Ldap_LdapServerUserAndGroupFieldSet_Email_FieldLabel')
      },
      {
        name: 'userPasswordAttribute',
        fieldLabel: AS.I18n.get('Ldap_LdapServerUserAndGroupFieldSet_Password_FieldLabel'),
        helpText: AS.I18n.get('Ldap_LdapServerUserAndGroupFieldSet_Password_HelpText'),
        allowBlank: true
      },
      // group
      {
        xtype: 'checkbox',
        name: 'ldapGroupsAsRoles',
        fieldLabel: AS.I18n.get('Ldap_LdapServerUserAndGroupFieldSet_GroupMap_FieldLabel'),
        value: true,
        listeners: {
          change: function (checkbox, newValue) {
            var panel = checkbox.up('panel');

            panel.showOrHide('ldapGroupsAsRoles', newValue);
            panel.showOrHide('groupType', newValue ? panel.down('#groupType').getValue() : undefined);
          }
        }
      },
      {
        xtype: 'combo',
        name: 'groupType',
        itemId: 'groupType',
        fieldLabel: AS.I18n.get('Ldap_LdapServerUserAndGroupFieldSet_GroupType_FieldLabel'),
        emptyText: AS.I18n.get('Ldap_LdapServerUserAndGroupFieldSet_GroupType_EmptyText'),
        editable: false,
        store: [
          ['dynamic', AS.I18n.get('Ldap_LdapServerUserAndGroupFieldSet_GroupType_DynamicItem')],
          ['static', AS.I18n.get('Ldap_LdapServerUserAndGroupFieldSet_GroupType_StaticItem')]
        ],
        queryMode: 'local',
        listeners: {
          change: function (combo, newValue) {
            combo.up('panel').showOrHide('groupType', newValue);
          }
        },
        ldapGroupsAsRoles: [true]
      },
      {
        name: 'groupBaseDn',
        fieldLabel: AS.I18n.get('Ldap_LdapServerUserAndGroupFieldSet_GroupBaseDN_FieldLabel'),
        helpText: AS.I18n.get('Ldap_LdapServerUserAndGroupFieldSet_GroupBaseDN_HelpText'),
        allowBlank: true,
        groupType: ['static']
      },
      {
        xtype: 'checkbox',
        name: 'groupSubtree',
        fieldLabel: AS.I18n.get('Ldap_LdapServerUserAndGroupFieldSet_GroupSubtree_FieldLabel'),
        helpText: AS.I18n.get('Ldap_LdapServerUserAndGroupFieldSet_GroupSubtree_HelpText'),
        groupType: ['static']
      },
      {
        name: 'groupObjectClass',
        fieldLabel: AS.I18n.get('Ldap_LdapServerUserAndGroupFieldSet_GroupObject_FieldLabel'),
        helpText: AS.I18n.get('Ldap_LdapServerUserAndGroupFieldSet_GroupObject_HelpText'),
        groupType: ['static']
      },
      {
        name: 'groupIdAttribute',
        fieldLabel: AS.I18n.get('Ldap_LdapServerUserAndGroupFieldSet_GroupID_FieldLabel'),
        groupType: ['static']
      },
      {
        name: 'groupMemberAttribute',
        fieldLabel: AS.I18n.get('Ldap_LdapServerUserAndGroupFieldSet_GroupMember_FieldLabel'),
        helpText: AS.I18n.get('Ldap_LdapServerUserAndGroupFieldSet_GroupMember_HelpText'),
        groupType: ['static']
      },
      {
        name: 'groupMemberFormat',
        fieldLabel: AS.I18n.get('Ldap_LdapServerUserAndGroupFieldSet_GroupMemberFormat_FieldLabel'),
        helpText: AS.I18n.get('Ldap_LdapServerUserAndGroupFieldSet_GroupMemberFormat_HelpText'),
        groupType: ['static']
      },
      {
        name: 'userMemberOfAttribute',
        fieldLabel: AS.I18n.get('Ldap_LdapServerUserAndGroupFieldSet_GroupMemberOf_FieldLabel'),
        helpText: AS.I18n.get('Ldap_LdapServerUserAndGroupFieldSet_GroupMemberOf_HelpText'),
        groupType: ['dynamic']
      }
    ];

    me.callParent();

    me.showOrHide('ldapGroupsAsRoles', true);
    me.showOrHide('groupType', undefined);
  },

  /**
   * @private
   * Show & enable or hide and disable components that have attributes that matches the specified value.
   * @param attribute name of attribute
   * @param value to be matched in order to show
   */
  showOrHide: function (attribute, value) {
    var me = this,
        form = me.up('form'),
        components = me.query('component[' + attribute + ']');

    Ext.iterate(components, function (component) {
      if (value && component[attribute].indexOf(value) > -1) {
        component.enable();
        component.show();
      }
      else {
        component.disable();
        component.hide();
      }
    });
    if (form && form.rendered) {
      form.isValid();
    }
  }

});
