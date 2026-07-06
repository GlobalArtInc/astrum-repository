
/*global Ext, NX*/

/**
 * Add LDAP users and groups window.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.ldap.LdapServerUserAndGroupAdd', {
  extend: 'NX.view.AddPanel',
  alias: 'widget.nx-coreui-ldapserver-userandgroup-add',
  requires: [
    'NX.Conditions',
    'NX.I18n'
  ],

  dockedItems: null,

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.settingsForm = {
      xtype: 'nx-coreui-ldapserver-userandgroup-form',

      editableCondition: NX.Conditions.isPermitted('astrum:ldap:create'),
      editableMarker: NX.I18n.get('Ldap_LdapServerConnectionAdd_Create_Error'),

      buttons: [
        { text: NX.I18n.get('Add_Submit_Button'), action: 'add', formBind: true, ui: 'nx-primary' },
        { text: NX.I18n.get('Add_Cancel_Button'), action: 'back' }
      ]
    };

    me.callParent();
  }
});
