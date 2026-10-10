
/*global Ext, AS*/

/**
 * Add capability window.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.capability.CapabilityAdd', {
  extend: 'AS.view.AddPanel',
  alias: 'widget.as-coreui-capability-add',
  requires: [
    'AS.I18n'
  ],

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.settingsForm = {
      xtype: 'as-coreui-capability-settings-form',
      editableCondition: AS.Conditions.isPermitted('astrum:capabilities:create'),
      editableMarker: AS.I18n.get('Capability_CapabilitySettingsForm_Update_Error'),

      buttons: [
        { text: AS.I18n.get('Capability_CapabilityList_New_Button'), action: 'add', formBind: true, ui: 'as-primary' },
        { text: AS.I18n.get('Add_Cancel_Button'), action: 'back' }
      ]
    };

    me.callParent();
  }

});
