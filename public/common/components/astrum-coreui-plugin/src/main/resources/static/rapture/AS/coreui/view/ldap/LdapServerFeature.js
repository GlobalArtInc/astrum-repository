
/*global Ext, AS*/

/**
 * LDAP Server feature panel.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.ldap.LdapServerFeature', {
  extend: 'AS.view.drilldown.Drilldown',
  alias: 'widget.as-coreui-ldapserver-feature',
  requires: [
    'AS.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      iconName: 'ldapserver-default',

      masters: [
        { xtype: 'as-coreui-ldapserver-list' }
      ],

      tabs: [
        { xtype: 'as-coreui-ldapserver-connection', title: AS.I18n.get('Ldap_LdapServerFeature_Connection_Title'), weight: 10 },
        { xtype: 'as-coreui-ldapserver-userandgroup', title: AS.I18n.get('Ldap_LdapServerFeature_UserAndGroup_Title'), weight: 20 }
      ],

      asActions: [
        { xtype: 'button', text: AS.I18n.get('Ldap_LdapServerFeature_Delete_Button'), iconCls: 'x-fa fa-trash', action: 'delete', disabled: true }
      ]
    });

    this.callParent();
  }

});
