
/*global Ext, AS*/

/**
 * Task feature panel.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.task.TaskFeature', {
  extend: 'AS.view.drilldown.Drilldown',
  alias: 'widget.as-coreui-task-feature',
  requires: [
    'AS.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      iconName: 'task-default',

      masters: [
        { xtype: 'as-coreui-task-list' }
      ],

      tabs: [
        {
          xtype: 'as-coreui-task-summary',
          weight: 10
        }
      ],

      asActions: [
        {
          xtype: 'button',
          text: AS.I18n.get('Task_TaskFeature_Delete_Button'),
          iconCls: 'x-fa fa-trash',
          action: 'delete',disabled: true
        },
        {
          id: 'task_run_button_id',
          xtype: 'button',
          text: AS.I18n.get('Task_TaskFeature_Run_Button'),
          iconCls: 'x-fa fa-play',
          action: 'run',
          handler: function(button) {
            button.fireEvent('runaction');
          },
          disabled: true
        },
        {
          xtype: 'button',
          text: AS.I18n.get('Task_TaskFeature_Stop_Button'),
          iconCls: 'x-fa fa-stop',
          action: 'stop',
          handler: function(button) {
            button.fireEvent('runaction');
          },
          disabled: true
        }
      ]
    });

    this.callParent();
  }
});
