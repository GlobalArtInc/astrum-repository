
/*global Ext, AS*/

/**
 * Task Schedule Advanced field set.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.task.TaskScheduleAdvanced', {
  extend: 'AS.coreui.view.task.TaskScheduleFields',
  alias: 'widget.as-coreui-task-schedule-advanced',
  requires: [
    'AS.I18n'
  ],

  initComponent: function () {
    var me = this;

    me.items = [
      {
        xtype: 'textfield',
        name: 'cronExpression',
        fieldLabel: AS.I18n.get('Task_TaskScheduleAdvanced_Cron_FieldLabel'),
        helpText: AS.I18n.get('Task_TaskScheduleAdvanced_Cron_HelpText'),
        afterBodyEl: AS.I18n.get('Task_TaskScheduleAdvanced_Cron_AfterBodyEl'),
        emptyText: AS.I18n.get('Task_TaskScheduleAdvanced_Cron_EmptyText'),
        allowBlank: false
      }
    ];

    me.callParent();
  }

});
