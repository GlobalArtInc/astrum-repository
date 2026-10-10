
/*global Ext, AS*/

/**
 * Task Schedule Monthly field set.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.task.TaskScheduleMonthly', {
  extend: 'AS.coreui.view.task.TaskScheduleFields',
  alias: 'widget.as-coreui-task-schedule-monthly',
  requires: [
    'AS.util.DateFormat',
    'AS.I18n'
  ],

  /**
   * @override
   */
  initComponent: function () {
    var me = this,
        columns = [];

    for (var c = 0; c < 7; c++) {
      columns.push({
        xtype: 'panel',
        items: [],
        width: 50
      });
    }

    for (var day = 1; day <= 32; day++) {
      var columnNumber = day % 7 - 1,
          recurringDayValue = day === 32 ? '999' : day;

      if (columnNumber === -1) {
        columnNumber = 6;
      }
      columns[columnNumber].items.push({
        xtype: 'checkbox',
        name: 'recurringDay-' + recurringDayValue,
        boxLabel: day === 32 ? 'Last' : day,
        submitValue: false,
        recurringDayValue: recurringDayValue
      });
    }

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
        items: {
          xtype: 'panel',
          layout: 'column',
          items: columns
        }
      }
    ];

    me.callParent();
  }

});
