
/*global Ext, NX*/

/**
 * Add task window.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.task.TaskAdd', {
  extend: 'NX.view.AddPanel',
  alias: 'widget.nx-coreui-task-add',
  requires: [
    'NX.Conditions',
    'NX.I18n'
  ],

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.settingsForm = {
      xtype: 'nx-coreui-task-settings-form',
      editableCondition: NX.Conditions.isPermitted('astrum:tasks:create'),
      editableMarker: NX.I18n.get('Task_TaskAdd_Create_Error'),

      buttons: [
        {text: NX.I18n.get('Task_TaskList_New_Button'), action: 'add', formBind: true, ui: 'nx-primary'},
        {text: NX.I18n.get('Add_Cancel_Button'), action: 'back'}
      ]
    };

    me.callParent();
  }
});
