
/*global Ext, NX*/

/**
 * Task feature panel.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.task.TaskFeature', {
  extend: 'NX.view.drilldown.Drilldown',
  alias: 'widget.nx-coreui-task-feature',
  requires: [
    'NX.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      iconName: 'task-default',

      masters: [
        { xtype: 'nx-coreui-task-list' }
      ],

      tabs: [
        {
          xtype: 'nx-coreui-task-summary',
          weight: 10
        }
      ],

      nxActions: [
        {
          xtype: 'button',
          text: NX.I18n.get('Task_TaskFeature_Delete_Button'),
          iconCls: 'x-fa fa-trash',
          action: 'delete',disabled: true
        },
        {
          id: NX.I18n.get('Task_TaskFeature_Run_Button_Id'),
          xtype: 'button',
          text: NX.I18n.get('Task_TaskFeature_Run_Button'),
          iconCls: 'x-fa fa-play',
          action: 'run',
          handler: function(button) {
            button.fireEvent('runaction');
          },
          disabled: true
        },
        {
          xtype: 'button',
          text: NX.I18n.get('Task_TaskFeature_Stop_Button'),
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
