
/*global Ext, AS*/

/**
 * Privilege feature panel.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.privilege.PrivilegeFeature', {
  extend: 'AS.view.drilldown.Drilldown',
  alias: 'widget.as-coreui-privilege-feature',
  requires: [
    'AS.I18n'
  ],

  iconName: 'privilege-default',

  masters: [
    { xtype: 'as-coreui-privilege-list' }
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    Ext.apply(me, {
      tabs: {
        xtype: 'as-coreui-privilege-settings',
        title: AS.I18n.get('Privilege_PrivilegeFeature_Settings_Title'),
        weight: 10
      },

      asActions: [
        { xtype: 'button', text: AS.I18n.get('Privilege_PrivilegeFeature_Delete_Button'), iconCls: 'x-fa fa-trash', action: 'delete', disabled: true }
      ]
    });

    me.callParent();
  }
});
