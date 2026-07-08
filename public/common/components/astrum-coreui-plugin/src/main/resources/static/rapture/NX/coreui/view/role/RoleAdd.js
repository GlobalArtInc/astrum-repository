
/*global Ext, NX*/

/**
 * Add role window.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.role.RoleAdd', {
  extend: 'NX.view.AddPanel',
  alias: 'widget.nx-coreui-role-add',
  requires: [
    'NX.Conditions',
    'NX.I18n'
  ],

  defaultFocus: 'id',

  initComponent: function() {
    var me = this;

    me.settingsForm = {
      xtype: 'nx-coreui-role-settings-form',
      api: {
        submit: 'NX.direct.coreui_Role.create'
      },
      settingsFormSuccessMessage: function(data) {
        return NX.I18n.get('Role_RoleAdd_Create_Success') + Ext.htmlEncode(data['name']);
      },
      editableCondition: NX.Conditions.isPermitted('astrum:roles:create'),
      editableMarker: NX.I18n.get('Role_RoleAdd_Create_Error'),
      source: me.source,

      buttons: [
        { text: NX.I18n.get('Role_RoleList_New_Button'), action: 'add', formBind: true, ui: 'nx-primary' },
        { text: NX.I18n.get('Add_Cancel_Button'), action: 'back' }
      ]
    };

    me.callParent();

    me.down('#id').setReadOnly(false);
  }

});
