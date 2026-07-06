
/*global Ext, NX*/

/**
 * LDAP Server "Connection" form.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.ldap.LdapServerConnectionForm', {
  extend: 'NX.view.SettingsForm',
  alias: 'widget.nx-coreui-ldapserver-connection-form',
  requires: [
    'NX.Conditions',
    'NX.I18n'
  ],
  // isAddPage variable uses for define, is user add new LDAP configuration or edit an old one.
  isAddPage: false,

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = { xtype: 'nx-coreui-ldapserver-connection-fieldset' };

    me.editableMarker = NX.I18n.get('Ldap_LdapServerConnectionForm_Update_Error');

    me.editableCondition = me.editableCondition || NX.Conditions.isPermitted('astrum:ldap:update');

    me.callParent();

    me.getDockedItems('toolbar[dock="bottom"]')[0].add(
        { xtype: 'button', text: NX.I18n.get('Ldap_LdapServerConnectionForm_VerifyConnection_Button'), action: 'verifyconnection' }
    );

    NX.Conditions.formIs(me, function(form) {
      return !form.isDisabled() && form.isValid();
    }).on({
      satisfied: function() {
        this.enable();
      },
      unsatisfied: function() {
        this.disable();
      },
      scope: me.down('button[action=verifyconnection]')
    });
  }

});
