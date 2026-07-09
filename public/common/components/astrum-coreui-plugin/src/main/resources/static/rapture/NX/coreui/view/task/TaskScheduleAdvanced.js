
/*global Ext, NX*/

/**
 * Task Schedule Advanced field set.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.task.TaskScheduleAdvanced', {
  extend: 'NX.coreui.view.task.TaskScheduleFields',
  alias: 'widget.nx-coreui-task-schedule-advanced',
  requires: [
    'NX.I18n'
  ],

  initComponent: function () {
    var me = this;

    me.items = [
      {
        xtype: 'textfield',
        name: 'cronExpression',
        fieldLabel: NX.I18n.get('Task_TaskScheduleAdvanced_Cron_FieldLabel'),
        helpText: NX.I18n.get('Task_TaskScheduleAdvanced_Cron_HelpText'),
        afterBodyEl: NX.I18n.get('Task_TaskScheduleAdvanced_Cron_AfterBodyEl'),
        emptyText: NX.I18n.get('Task_TaskScheduleAdvanced_Cron_EmptyText'),
        allowBlank: false
      }
    ];

    me.callParent();
  }

});
