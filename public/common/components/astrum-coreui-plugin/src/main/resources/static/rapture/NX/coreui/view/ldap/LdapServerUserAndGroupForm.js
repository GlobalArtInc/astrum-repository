
/*global Ext, NX*/

/**
 * LDAP Server "User & Group" form.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.ldap.LdapServerUserAndGroupForm', {
  extend: 'NX.view.SettingsForm',
  alias: 'widget.nx-coreui-ldapserver-userandgroup-form',
  requires: [
    'NX.Conditions',
    'NX.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = { xtype: 'nx-coreui-ldapserver-userandgroup-fieldset' };

    me.editableMarker = NX.I18n.get('Ldap_LdapServerConnectionForm_Update_Error');

    me.editableCondition = me.editableCondition || NX.Conditions.isPermitted('astrum:ldap:update');

    me.callParent();

    me.getDockedItems('toolbar[dock="bottom"]')[0].add(
        { xtype: 'button', text: NX.I18n.get('Ldap_LdapServerUserAndGroupForm_VerifyGroupMapping_Button'), formBind: true, action: 'verifyusermapping' },
        { xtype: 'button', text: NX.I18n.get('Ldap_LdapServerUserAndGroupForm_VerifyLogin_Button'), formBind: true, action: 'verifylogin' }
    );
  },

  /**
   * @override
   * Additionally, marks invalid properties.
   */
  markInvalid: function(errors) {
    this.down('nx-coreui-ldapserver-userandgroup-fieldset').markInvalid(errors);
  }

});
