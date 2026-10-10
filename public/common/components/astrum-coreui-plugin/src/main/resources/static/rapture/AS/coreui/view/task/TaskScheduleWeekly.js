
/*global Ext, AS*/

/**
 * Task Schedule Weekly field set.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.task.TaskScheduleWeekly', {
  extend: 'AS.coreui.view.task.TaskScheduleFields',
  alias: 'widget.as-coreui-task-schedule-weekly',
  requires: [
    'AS.util.DateFormat',
    'AS.I18n'
  ],

  /**
   * @override
   */
  initComponent: function () {
    var me = this,
        weekDays = ['sunday', 'monday', 'tuesday', 'wednesday', 'thursday', 'friday', 'saturday'],
        days = [];

    Ext.Array.each(weekDays, function (day)
    {
      days.push({
        xtype: 'checkbox',
        name: 'recurringDay-' + (weekDays.indexOf(day) + 1),
        boxLabel: Ext.String.capitalize(day),
        recurringDayValue: weekDays.indexOf(day) + 1,
        submitValue: false
      });
    });

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
      },
      {
        xtype: 'checkboxgroup',
        fieldLabel: AS.I18n.get('Task_TaskScheduleMonthly_Days_FieldLabel'),
        allowBlank: false,
        blankText: AS.I18n.get('Task_TaskScheduleMonthly_Days_BlankText'),
        vertical: true,
        columns: 1,
        items: days
      }
    ];

    me.callParent();
  }

});
