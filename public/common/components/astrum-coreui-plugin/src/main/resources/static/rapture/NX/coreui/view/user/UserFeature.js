
/*global Ext, NX*/

/**
 * User feature panel.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.user.UserFeature', {
  extend: 'NX.view.drilldown.Drilldown',
  alias: 'widget.nx-coreui-user-feature',
  requires: [
    'NX.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      iconName: 'user-default',

      masters: [
        { xtype: 'nx-coreui-user-list' }
      ],

      tabs: [
        { xtype: 'nx-coreui-user-settings', title: NX.I18n.get('User_UserFeature_Settings_Title'), weight: 10 }
      ],

      nxActions: [
        { xtype: 'button', text: NX.I18n.get('User_UserFeature_Delete_Button'), iconCls: 'x-fa fa-trash', action: 'delete', disabled: true },
        { xtype: 'button', text: NX.I18n.get('User_UserFeature_ChangePasswordItem'), iconCls: 'x-fa fa-key', action: 'setpassword', disabled: true}
      ]
    });

    this.callParent();
  }
});
