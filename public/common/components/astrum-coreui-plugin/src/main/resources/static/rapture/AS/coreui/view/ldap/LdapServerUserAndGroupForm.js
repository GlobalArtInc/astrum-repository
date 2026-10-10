
/*global Ext, AS*/

/**
 * LDAP Server "User & Group" form.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.ldap.LdapServerUserAndGroupForm', {
  extend: 'AS.view.SettingsForm',
  alias: 'widget.as-coreui-ldapserver-userandgroup-form',
  requires: [
    'AS.Conditions',
    'AS.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = { xtype: 'as-coreui-ldapserver-userandgroup-fieldset' };

    me.editableMarker = AS.I18n.get('Ldap_LdapServerConnectionForm_Update_Error');

    me.editableCondition = me.editableCondition || AS.Conditions.isPermitted('astrum:ldap:update');

    me.callParent();

    me.getDockedItems('toolbar[dock="bottom"]')[0].add(
        { xtype: 'button', text: AS.I18n.get('Ldap_LdapServerUserAndGroupForm_VerifyGroupMapping_Button'), formBind: true, action: 'verifyusermapping' },
        { xtype: 'button', text: AS.I18n.get('Ldap_LdapServerUserAndGroupForm_VerifyLogin_Button'), formBind: true, action: 'verifylogin' }
    );
  },

  /**
   * @override
   * Additionally, marks invalid properties.
   */
  markInvalid: function(errors) {
    this.down('as-coreui-ldapserver-userandgroup-fieldset').markInvalid(errors);
  }

});
