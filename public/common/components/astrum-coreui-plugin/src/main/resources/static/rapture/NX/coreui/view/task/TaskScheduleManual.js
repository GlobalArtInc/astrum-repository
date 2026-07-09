
/*global Ext, NX*/

/**
 * Task Schedule Manual field set.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.task.TaskScheduleManual', {
  extend: 'NX.coreui.view.task.TaskScheduleFields',
  alias: 'widget.nx-coreui-task-schedule-manual',
  requires: [
    'NX.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      items: {
        xtype: 'label',
        text: NX.I18n.get('Task_TaskScheduleManual_HelpText')
      }
    });

    this.callParent();
  }

});
