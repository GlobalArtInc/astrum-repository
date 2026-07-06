
/*global Ext, NX*/

/**
 * LDAP Server feature panel.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.ldap.LdapServerFeature', {
  extend: 'NX.view.drilldown.Drilldown',
  alias: 'widget.nx-coreui-ldapserver-feature',
  requires: [
    'NX.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      iconName: 'ldapserver-default',

      masters: [
        { xtype: 'nx-coreui-ldapserver-list' }
      ],

      tabs: [
        { xtype: 'nx-coreui-ldapserver-connection', title: NX.I18n.get('Ldap_LdapServerFeature_Connection_Title'), weight: 10 },
        { xtype: 'nx-coreui-ldapserver-userandgroup', title: NX.I18n.get('Ldap_LdapServerFeature_UserAndGroup_Title'), weight: 20 }
      ],

      nxActions: [
        { xtype: 'button', text: NX.I18n.get('Ldap_LdapServerFeature_Delete_Button'), iconCls: 'x-fa fa-trash', action: 'delete', disabled: true }
      ]
    });

    this.callParent();
  }

});
