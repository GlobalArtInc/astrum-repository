
/*global Ext, NX*/

/**
 * Add LDAP connection window.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.ldap.LdapServerConnectionAdd', {
  extend: 'NX.view.AddPanel',
  alias: 'widget.nx-coreui-ldapserver-connection-add',
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
      xtype: 'nx-coreui-ldapserver-connection-form',
      isAddPage: true,

      editableCondition: NX.Conditions.isPermitted('astrum:ldap:create'),
      editableMarker: NX.I18n.get('Ldap_LdapServerConnectionAdd_Create_Error'),

      buttons: [
        { text: NX.I18n.get('Ldap_LdapServerConnectionAdd_Text'), action: 'next', ui: 'nx-primary' },
        { text: NX.I18n.get('Add_Cancel_Button'), action: 'back' }
      ]
    };

    me.callParent();

    NX.Conditions.formIs(me.down('form'), function(form) {
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
