
/*global Ext, NX*/

/**
 * Component details panel.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.component.ComponentDetails', {
  // FIXME: change to container
  extend: 'Ext.panel.Panel',
  alias: 'widget.nx-coreui-component-details',

  cls: 'nx-coreui-component-details',

  requires: [
    'NX.ext.button.Button'
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
        xtype: 'nx-actions',
        dock: 'bottom',
        items: [
          {
            xtype: 'nx-button',
            text: NX.I18n.get('ComponentDetails_View_Vulnerabilities_Button'),
            iconCls: 'x-fa fa-bug',
            action: 'viewVulnerabilities',
            hidden: true
          },
          {
            xtype: 'nx-button',
            text: ''+NX.I18n.get('ComponentDetails_Delete_Button'),
            iconCls: 'x-fa fa-trash',
            action: 'deleteComponent',
            hidden: true
          },
          {
            xtype: 'nx-button',
            text: NX.I18n.get('ComponentDetails_Browse_Snapshots_Button'),
            iconCls: 'x-fa fa-database',
            action: 'browseComponent',
            hidden: true
          }
        ]
      },

      items: [
        {xtype: 'nx-info', itemId: 'repositoryInfo'},
        {xtype: 'nx-info', itemId: 'componentInfo'},
        {xtype: 'nx-info', itemId: 'extraInfo'}
      ]
    });

    this.callParent();
  },

  /**
   * Sets component.
   *
   * @public
   * @param {NX.coreui.model.Component} componentModel
   */
  setComponentModel: function(componentModel) {
    var me = this;

    me.componentModel = componentModel;
    me.fireEvent('updated', me, me.componentModel);
  }
});
