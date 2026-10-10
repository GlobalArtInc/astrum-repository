
/*global Ext, AS*/

/**
 * Master/Detail tabs.
 *
 * @since 3.0
 */
Ext.define('AS.view.drilldown.Details', {
  extend: 'Ext.panel.Panel',
  alias: 'widget.as-drilldown-details',
  requires: [
    'AS.Icons',
    'AS.Bookmarks',
    'AS.ext.tab.SortedPanel',
    'AS.view.drilldown.Actions'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {
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
        items: me.asActions
      },
      {
        xtype: 'as-sorted-tabpanel',
        itemId: 'tab',
        ui: 'as-light',
        cls: 'as-hr',
        activeTab: 0,
        layoutOnTabChange: true,
        flex: 1,
        items: me.tabs
      }
    ];

    me.callParent();

    me.on('afterrender', me.calculateBookmarks, me);
  },

  showInfo: function(message, tooltipText) {
    var infoPanel = this.down('>#info');

    infoPanel.setTitle(message);
    Ext.tip.QuickTipManager.unregister(infoPanel.getId());
    if (tooltipText) {
      Ext.tip.QuickTipManager.register({
        showDelay: 50,
        target: infoPanel.getId(),
        text  : tooltipText,
        trackMouse: true
      });
    }
    infoPanel.show();
  },

  clearInfo: function() {
    var infoPanel = this.down('>#info');

    infoPanel.hide();
  },

  showWarning: function(message) {
    var warningPanel = this.down('>#warning');

    warningPanel.setTitle(message);
    warningPanel.show();
  },

  clearWarning: function() {
    var warningPanel = this.down('>#warning');

    warningPanel.hide();
  },

  addTab: function(tab) {
    var me = this,
        tabPanel = me.down('>#tab');

    tabPanel.add(tab);
    me.calculateBookmarks();
  },

  removeTab: function(tab) {
    var me = this,
        tabPanel = me.down('>#tab');

    tabPanel.remove(tab);
    me.calculateBookmarks();
  },

  /**
   * @public
   * @returns {String} bookmark token of selected tab
   */
  getBookmarkOfSelectedTab: function() {
    var tabPanel = this.down('>#tab');

    return tabPanel.getActiveTab().bookmark;
  },

  /**
   * @public
   * Finds a tab by bookmark & sets it active (if found).
   * @param {String} bookmark of tab to be activated
   */
  setActiveTabByBookmark: function(bookmark) {
    var me = this,
        tabPanel = me.down('>#tab'),
        tab = me.down('> tabpanel > panel[bookmark=' + bookmark + ']');

    if (tabPanel && tab) {
      tabPanel.setActiveTab(tab);
    }
  },

  /**
   * @private
   * Calculates bookmarks of all tabs based on tab title.
   */
  calculateBookmarks: function() {
    var tabPanel = this.down('>#tab');

    tabPanel.items.each(function(tab) {
      if (tab.title) {
        tab.bookmark = AS.Bookmarks.encode(tab.title).toLowerCase();
      }
    });
  }

});
