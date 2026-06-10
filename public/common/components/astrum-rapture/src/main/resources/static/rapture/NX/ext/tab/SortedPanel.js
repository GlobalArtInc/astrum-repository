
/*global Ext, NX*/

/**
 * A tab panel that sorts tabs based on weight and title and not show the tab bar if only one tab.
 *
 * @since 3.0
 */
Ext.define('NX.ext.tab.SortedPanel', {
  extend: 'Ext.tab.Panel',
  alias: 'widget.nx-sorted-tabpanel',

  add: function(component) {
    var componentWeight = component.weight || 1000,
        componentTitle = component.title || '';

    var index = this.items.findIndexBy(function(item, key) {
      var itemWeight = item.weight || 1000,
          itemTitle = item.title || '';
      return componentWeight < itemWeight || (componentWeight === itemWeight && componentTitle < itemTitle);
    });

    this.callParent([index, component]);
  },

  // FIXME: This doesn't belong here, this is styling treatment for master/detail tabs only
  /**
   * @override
   */
  onAdd: function(item, index) {
    item.tabConfig = item.tabConfig || {};
    Ext.applyIf(item.tabConfig, {
      // HACK: force tabs to follow scss style for borders
      border: null,
      title: item.title || item.titled
    });

    this.callParent([item, index]);
  }
});
