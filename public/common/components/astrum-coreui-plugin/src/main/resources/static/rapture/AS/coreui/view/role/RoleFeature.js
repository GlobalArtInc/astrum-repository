
/*global Ext, AS*/

/**
 * Role feature panel.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.role.RoleFeature', {
  extend: 'AS.view.drilldown.Drilldown',
  alias: 'widget.as-coreui-role-feature',
  requires: [
    'AS.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      iconName: 'role-default',

      masters: [
        { xtype: 'as-coreui-role-list' }
      ],

      tabs: [
        { xtype: 'as-coreui-role-settings', title: AS.I18n.get('Role_RoleFeature_Settings_Title'), weight: 10 }
      ],

      asActions: [
        { xtype: 'button', text: AS.I18n.get('Role_RoleFeature_Delete_Button'), iconCls: 'x-fa fa-trash', action: 'delete', disabled: true }
      ]
    });

    this.callParent();
  }
});
