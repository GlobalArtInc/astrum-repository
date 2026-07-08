
/*global Ext, NX*/

/**
 * Role feature panel.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.role.RoleFeature', {
  extend: 'NX.view.drilldown.Drilldown',
  alias: 'widget.nx-coreui-role-feature',
  requires: [
    'NX.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      iconName: 'role-default',

      masters: [
        { xtype: 'nx-coreui-role-list' }
      ],

      tabs: [
        { xtype: 'nx-coreui-role-settings', title: NX.I18n.get('Role_RoleFeature_Settings_Title'), weight: 10 }
      ],

      nxActions: [
        { xtype: 'button', text: NX.I18n.get('Role_RoleFeature_Delete_Button'), iconCls: 'x-fa fa-trash', action: 'delete', disabled: true }
      ]
    });

    this.callParent();
  }
});
