
/*global Ext, AS*/

/**
 * Component/Asset tree panel
 *
 * @since 3.6
 */
Ext.define('AS.coreui.view.browse.ComponentAssetTree', {
  extend: 'Ext.panel.Panel',
  alias: 'widget.as-coreui-component-asset-tree',
  requires: [
    'AS.I18n',
    'AS.State',
    'AS.coreui.view.component.ComponentInfo',
    'AS.coreui.view.component.ComponentAssetInfo',
    'AS.coreui.view.component.ComponentFolderInfo'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;
    var store = me.store || 'ComponentAssetTree';
    Ext.apply(me, {
      layout: {
        type: 'hbox',
        align: 'stretch'
      },
      items: [{
        xtype: 'treepanel',
        ariaRole: 'grid',
        flex: 3,
        store: store,
        rootVisible: false,
        // Prevent the store from automatically loading
        loadStore: Ext.emptyFn,
        animate: Ext.enableFx && !!AS.State.getValue('animateDuration'),
        viewConfig: {
          emptyText: AS.I18n.get('Component_Asset_Tree_EmptyText_View'),
          deferEmptyText: false
        }
      },
      {
        xtype: 'as-coreui-component-componentinfo',
        iconCls: 'as-icon-tree-component-x16',
        flex: 2,
        visible: false
      },
      {
        xtype: 'as-coreui-component-componentassetinfo',
        flex: 2,
        iconCls: 'as-icon-tree-asset-x16',
        visible: false
      },
      {
        xtype: 'as-coreui-component-componentfolderinfo',
        iconCls: 'as-icon-tree-folder-x16',
        flex: 2,
        visible: false
      }],
      dockedItems: [{
        xtype: 'panel',
        items: [{
          xtype: 'panel',
          itemId: 'info',
          ui: 'as-drilldown-message',
          cls: 'as-drilldown-info',
          iconCls: AS.Icons.cls('drilldown-info', 'x16'),
          hidden: true
        },
        {
          xtype: 'panel',
          itemId: 'warning',
          ui: 'as-drilldown-message',
          cls: 'as-drilldown-warning',
          iconCls: AS.Icons.cls('drilldown-warning', 'x16'),
          hidden: true
        },
        {
          xtype: 'as-actions',
          dock: 'top',
          items: [{
            xtype: 'button',
            text: AS.I18n.get('Component_Asset_Tree_Upload_Component'),
            iconCls: 'x-fa fa-upload',
            action: 'upload',
            hidden: true
          },
          {
            xtype: 'label',
            itemId: 'as-coreui-component-asset-tree-html-view',
            html: AS.util.Url.asLink("", AS.I18n.get('Component_Asset_Tree_Html_View'), '_blank')
          },
          '->',
          {
            xtype: 'label',
            itemId: 'as-coreui-component-asset-tree-advanced-search',
            html: '<a href="#browse/search/generic">Advanced search...</a>'
          }]
        }]
      }]
    });

    me.callParent();
  }

});
