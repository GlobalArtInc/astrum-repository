
/*global Ext, AS*/

/**
 * Add task window.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.task.TaskAdd', {
  extend: 'AS.view.AddPanel',
  alias: 'widget.as-coreui-task-add',
  requires: [
    'AS.Conditions',
    'AS.I18n'
  ],

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.settingsForm = {
      xtype: 'as-coreui-task-settings-form',
      editableCondition: AS.Conditions.isPermitted('astrum:tasks:create'),
      editableMarker: AS.I18n.get('Task_TaskAdd_Create_Error'),

      buttons: [
        {text: AS.I18n.get('Task_TaskList_New_Button'), action: 'add', formBind: true, ui: 'as-primary'},
        {text: AS.I18n.get('Add_Cancel_Button'), action: 'back'}
      ]
    };

    me.callParent();
  }
});
