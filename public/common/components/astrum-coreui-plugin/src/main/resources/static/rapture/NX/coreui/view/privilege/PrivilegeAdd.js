
/*global Ext, NX*/

/**
 * Add privilege window.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.privilege.PrivilegeAdd', {
  extend: 'NX.view.AddPanel',
  alias: 'widget.nx-coreui-privilege-add',
  requires: [
    'NX.I18n'
  ],

  defaultFocus: 'name',

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    Ext.apply(me, {
      settingsForm: {
        xtype: 'nx-coreui-privilege-settings-form',
        editableCondition: NX.Conditions.isPermitted('astrum:privileges:create'),
        editableMarker: NX.I18n.get('Privilege_PrivilegeSettingsForm_Update_Error'),

        buttons: [
          {text: NX.I18n.get('Privilege_PrivilegeList_New_Button'), action: 'add', formBind: true, ui: 'nx-primary'},
          {text: NX.I18n.get('Add_Cancel_Button'), action: 'back'}
        ]
      }
    });

    me.callParent();

    me.down('#name').setReadOnly(false);
  }

});
