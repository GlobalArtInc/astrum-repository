
/*global Ext, AS*/

/**
 * Component details panel.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.component.ComponentDetails', {
  // FIXME: change to container
  extend: 'Ext.panel.Panel',
  alias: 'widget.as-coreui-component-details',

  cls: 'as-coreui-component-details',

  requires: [
    'AS.ext.button.Button'
  ],

  /**
   * Currently shown component model.
   */
  componentModel: undefined,

  layout: {
    type: 'hbox',
    align: 'stretch'
  },

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      dockedItems: {
        xtype: 'as-actions',
        dock: 'bottom',
        items: [
          {
            xtype: 'as-button',
            text: AS.I18n.get('ComponentDetails_View_Vulnerabilities_Button'),
            iconCls: 'x-fa fa-bug',
            action: 'viewVulnerabilities',
            hidden: true
          },
          {
            xtype: 'as-button',
            text: ''+NX.I18n.get('ComponentDetails_Delete_Button'),
            iconCls: 'x-fa fa-trash',
            action: 'deleteComponent',
            hidden: true
          },
          {
            xtype: 'as-button',
            text: AS.I18n.get('ComponentDetails_Browse_Snapshots_Button'),
            iconCls: 'x-fa fa-database',
            action: 'browseComponent',
            hidden: true
          }
        ]
      },

      items: [
        {xtype: 'as-info', itemId: 'repositoryInfo'},
        {xtype: 'as-info', itemId: 'componentInfo'},
        {xtype: 'as-info', itemId: 'extraInfo'}
      ]
    });

    this.callParent();
  },

  /**
   * Sets component.
   *
   * @public
   * @param {AS.coreui.model.Component} componentModel
   */
  setComponentModel: function(componentModel) {
    var me = this;

    me.componentModel = componentModel;
    me.fireEvent('updated', me, me.componentModel);
  }
});
