
/*global Ext, NX*/

/**
 * Privilege feature panel.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.privilege.PrivilegeFeature', {
  extend: 'NX.view.drilldown.Drilldown',
  alias: 'widget.nx-coreui-privilege-feature',
  requires: [
    'NX.I18n'
  ],

  iconName: 'privilege-default',

  masters: [
    { xtype: 'nx-coreui-privilege-list' }
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    Ext.apply(me, {
      tabs: {
        xtype: 'nx-coreui-privilege-settings',
        title: NX.I18n.get('Privilege_PrivilegeFeature_Settings_Title'),
        weight: 10
      },

      nxActions: [
        { xtype: 'button', text: NX.I18n.get('Privilege_PrivilegeFeature_Delete_Button'), iconCls: 'x-fa fa-trash', action: 'delete', disabled: true }
      ]
    });

    me.callParent();
  }
});
