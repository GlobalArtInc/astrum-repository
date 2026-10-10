
/*global Ext, AS*/

/**
 * Task "Summary" panel.
 *
 * @since 3.1
 */
Ext.define('AS.coreui.view.task.TaskSummary', {
  extend: 'Ext.Panel',
  alias: 'widget.as-coreui-task-summary',
  requires: [
    'AS.Conditions',
    'AS.I18n'
  ],
  cls: 'as-hr',

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.title = AS.I18n.get('TaskFeature_Summary_Title');
    me.autoScroll = true;

    me.layout = {
      type: 'vbox',
      align: 'stretch'
    };

    me.items = {
      xtype: 'panel',
      ui: 'as-inset',
      items: [
        {
          xtype: 'panel',
          ui: 'as-subsection',
          itemId: 'as-coreui-task-summary-subsection',
          frame: true,
          layout: 'column',
          weight: 10,
          items: [
            {
              xtype: 'as-info',
              columnWidth: 1
            }
          ]
        }
      ]
    };

    me.callParent();
  },

  showInfo: function (info) {
    this.down('as-info').showInfo(info);
  }
});
