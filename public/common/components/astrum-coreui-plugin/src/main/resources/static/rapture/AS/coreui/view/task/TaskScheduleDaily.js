
/*global Ext, AS*/

/**
 * Task Schedule Daily field set.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.task.TaskScheduleDaily', {
  extend: 'AS.coreui.view.task.TaskScheduleFields',
  alias: 'widget.as-coreui-task-schedule-daily',
  requires: [
    'AS.util.DateFormat',
    'AS.I18n'
  ],

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.items = [
      {
        xtype: 'datefield',
        name: 'startDate',
        itemId: 'startDate',
        fieldLabel: AS.I18n.get('Task_TaskScheduleDaily_StartDate_FieldLabel'),
        allowBlank: false,
        format: 'm/d/Y',
        value: new Date(),
        submitValue: false
      },
      {
        xtype: 'timefield',
        name: 'startTime',
        itemId: 'startTime',
        fieldLabel: AS.I18n.get('Task_TaskScheduleDaily_Recurring_FieldLabel'),
        helpText: AS.I18n.format('Task_TaskSchedule_StartTime_Days_HelpText', AS.util.DateFormat.getTimeZone()),
        allowBlank: false,
        format: 'H:i',
        submitValue: false
      }
    ];

    me.callParent();
  }
});
