
/*global Ext, NX*/

/**
 * Task Schedule Once field set.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.task.TaskScheduleOnce', {
  extend: 'NX.coreui.view.task.TaskScheduleFields',
  alias: 'widget.nx-coreui-task-schedule-once',
  requires: [
    'NX.util.DateFormat',
    'NX.I18n'
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
        fieldLabel: NX.I18n.get('Task_TaskScheduleDaily_StartDate_FieldLabel'),
        allowBlank: false,
        format: 'm/d/Y',
        value: new Date(),
        submitValue: false
      },
      {
        xtype: 'timefield',
        name: 'startTime',
        itemId: 'startTime',
        fieldLabel: NX.I18n.get('Task_TaskScheduleHourly_EndDate_FieldLabel'),
        helpText: 'The time this task should start running in your time zone ' + NX.util.DateFormat.getTimeZone() + '.',
        allowBlank: false,
        format: 'H:i',
        submitValue: false
      }
    ];

    me.callParent();
  }

});
