
/*global Ext, AS*/

/**
 * Add LDAP connection window.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.ldap.LdapServerConnectionAdd', {
  extend: 'AS.view.AddPanel',
  alias: 'widget.as-coreui-ldapserver-connection-add',
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
      xtype: 'as-coreui-ldapserver-connection-form',
      isAddPage: true,

      editableCondition: AS.Conditions.isPermitted('astrum:ldap:create'),
      editableMarker: AS.I18n.get('Ldap_LdapServerConnectionAdd_Create_Error'),

      buttons: [
        { text: AS.I18n.get('Ldap_LdapServerConnectionAdd_Text'), action: 'next', ui: 'as-primary' },
        { text: AS.I18n.get('Add_Cancel_Button'), action: 'back' }
      ]
    };

    me.callParent();

    AS.Conditions.formIs(me.down('form'), function(form) {
      return !form.isDisabled() && form.isValid();
    }).on({
      satisfied: function() {
        this.enable();
      },
      unsatisfied: function() {
        this.disable();
      },
      scope: me.down('button[action=next]')
    });
  }
});
