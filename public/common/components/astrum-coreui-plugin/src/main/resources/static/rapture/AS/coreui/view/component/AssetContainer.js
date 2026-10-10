
/*global Ext, AS*/

/**
 * Asset tabs container.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.component.AssetContainer', {
  extend: 'Ext.Panel',
  alias: 'widget.as-coreui-component-assetcontainer',
  requires: [
    'AS.Icons',
    'AS.ext.button.Button'
  ],

  /**
   * Currently shown asset model.
   *
   * @private
   */
  assetModel: undefined,

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      layout: 'fit',

      dockedItems: {
        xtype: 'as-actions',
        items: [
          {
            xtype: 'as-button',
            text: AS.I18n.get('AssetInfo_Delete_Button'),
            iconCls: 'x-fa fa-trash',
            action: 'deleteAsset',
            hidden: true
          }
        ]
      },

      items: [
        {
          xtype: 'as-sorted-tabpanel',
          ui: 'as-light',
          cls: 'as-hr',
          itemId: 'assetInfoTabs',
          autoScroll: true,
          layout: 'vbox',
          activeTab: 0
        }
      ]
    });

    this.callParent();
  },

  /**
   * Shows an asset in container.
   *
   * @public
   * @param {AS.coreui.model.Asset} assetModel asset to be shown
   */
  refreshInfo: function (assetModel) {
    var me = this,
        iconName = 'asset-type-default',
        contentType;

    me.assetModel = assetModel;

    if (me.assetModel) {
      if (me.hidden) {
        me.show();
      }
      contentType = me.assetModel.get('contentType').replace('/', '-');
      if (AS.getApplication().getIconController().findIcon('asset-type-' + contentType, 'x16')) {
        iconName = 'asset-type-' + contentType;
      }
      me.setIconCls(AS.Icons.cls(iconName, 'x16'));
      me.setTitle(me.assetModel.get('name'));

      me.fireEvent('updated', me, me.assetModel);
    }
    else {
      me.hide();
    }
  },

  /**
   * Contribute a content tab related to the visible Asset. If the tabItemId already exists, it will be replaced.
   * @public
   */
  addTab: function (item) {
    var me = this, tabs = me.down('#assetInfoTabs');
    var existingTab = tabs.down('#' + item.itemId);
    if (existingTab) {
      tabs.remove(existingTab);
    }
    tabs.add(item);
    tabs.setActiveTab(0);
  },

  /**
   * Show a tab
   * @public
   */
  showTab: function (itemId) {
    var tabs = this.down('#assetInfoTabs'),
        existingTab = tabs.child('#' + itemId);

    if (existingTab) {
      existingTab.tab.show();
    }
  },

  /**
   * Hide a tab
   * @public
   */
  hideTab: function (itemId) {
    var tabs = this.down('#assetInfoTabs'),
        existingTab = tabs.child('#' + itemId),
        wasActive = tabs.getActiveTab() === existingTab;

    if (existingTab) {
      existingTab.tab.hide();

      if (wasActive) {
        tabs.setActiveTab(0);
      }
    }
  }

});
