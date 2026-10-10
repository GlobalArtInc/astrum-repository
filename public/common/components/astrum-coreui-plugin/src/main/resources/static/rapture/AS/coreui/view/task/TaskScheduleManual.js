
/*global Ext, AS*/

/**
 * Task Schedule Manual field set.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.task.TaskScheduleManual', {
  extend: 'AS.coreui.view.task.TaskScheduleFields',
  alias: 'widget.as-coreui-task-schedule-manual',
  requires: [
    'AS.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      items: {
        xtype: 'label',
        text: AS.I18n.get('Task_TaskScheduleManual_HelpText')
      }
    });

    this.callParent();
  }

});
