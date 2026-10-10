
/*global Ext, AS*/

/**
 * LDAP Server "Connection" form.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.ldap.LdapServerConnectionForm', {
  extend: 'AS.view.SettingsForm',
  alias: 'widget.as-coreui-ldapserver-connection-form',
  requires: [
    'AS.Conditions',
    'AS.I18n'
  ],
  // isAddPage variable uses for define, is user add new LDAP configuration or edit an old one.
  isAddPage: false,

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = { xtype: 'as-coreui-ldapserver-connection-fieldset' };

    me.editableMarker = AS.I18n.get('Ldap_LdapServerConnectionForm_Update_Error');

    me.editableCondition = me.editableCondition || AS.Conditions.isPermitted('astrum:ldap:update');

    me.callParent();

    me.getDockedItems('toolbar[dock="bottom"]')[0].add(
        { xtype: 'button', text: AS.I18n.get('Ldap_LdapServerConnectionForm_VerifyConnection_Button'), action: 'verifyconnection' }
    );

    AS.Conditions.formIs(me, function(form) {
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
