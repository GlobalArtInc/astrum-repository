
/*global Ext, AS*/

/**
 * Add LDAP users and groups window.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.ldap.LdapServerUserAndGroupAdd', {
  extend: 'AS.view.AddPanel',
  alias: 'widget.as-coreui-ldapserver-userandgroup-add',
  requires: [
    'AS.Conditions',
    'AS.I18n'
  ],

  dockedItems: null,

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.settingsForm = {
      xtype: 'as-coreui-ldapserver-userandgroup-form',

      editableCondition: AS.Conditions.isPermitted('astrum:ldap:create'),
      editableMarker: AS.I18n.get('Ldap_LdapServerConnectionAdd_Create_Error'),

      buttons: [
        { text: AS.I18n.get('Add_Submit_Button'), action: 'add', formBind: true, ui: 'as-primary' },
        { text: AS.I18n.get('Add_Cancel_Button'), action: 'back' }
      ]
    };

    me.callParent();
  }
});
