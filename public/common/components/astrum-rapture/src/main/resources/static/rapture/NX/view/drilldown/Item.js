
/*global Ext*/

/**
 * A single panel in a drilldown series
 *
 * @since 3.0
 */

Ext.define('NX.view.drilldown.Item', {
  extend: 'Ext.panel.Panel',
  alias: 'widget.nx-drilldown-item',

  itemName: null,
  itemClass: null,
  itemBookmark: null,
  cardIndex: 0,

  layout: 'card',

  /**
   * @public
   * Set the name of this drilldown item (appears in the breadcrumb)
   */
  setItemName: function(text) {
    this.itemName = text;
  },

  /**
   * @public
   * Set the icon class of this drilldown item (appears in the breadcrumb)
   */
  setItemClass: function(cls) {
    this.itemClass = cls;
  },

  /**
   * @public
   * Set the page to load when the breadcrumb segment associated with this drilldown item is clicked
   */
  setItemBookmark: function(bookmark, scope) {
    this.itemBookmark = (bookmark ? { obj: bookmark, scope: scope } : null);
  },

  /**
   * @public
   * Set the currently selected card (will not change the active index by itself)
   */
  setCardIndex: function(index) {
    this.cardIndex = index;
  }
});
