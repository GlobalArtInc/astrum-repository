
/*global Ext, NX*/

/**
 * Add capability window.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.capability.CapabilityAdd', {
  extend: 'NX.view.AddPanel',
  alias: 'widget.nx-coreui-capability-add',
  requires: [
    'NX.I18n'
  ],

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.settingsForm = {
      xtype: 'nx-coreui-capability-settings-form',
      editableCondition: NX.Conditions.isPermitted('astrum:capabilities:create'),
      editableMarker: NX.I18n.get('Capability_CapabilitySettingsForm_Update_Error'),

      buttons: [
        { text: NX.I18n.get('Capability_CapabilityList_New_Button'), action: 'add', formBind: true, ui: 'nx-primary' },
        { text: NX.I18n.get('Add_Cancel_Button'), action: 'back' }
      ]
    };

    me.callParent();
  }

});
