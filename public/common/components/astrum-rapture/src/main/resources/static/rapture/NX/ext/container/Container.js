
/*global Ext*/

/**
 * Override ExtJS Container to protect against null items during component destruction.
 *
 */
Ext.define('NX.ext.container.Container', {
  override: 'Ext.container.Container',

  /**
   * @override
   * Protect against null items collection during destruction lifecycle.
   * This is called by query(), enable(), disable() and other traversal methods.
   *
   * Original ExtJS implementation:
   *   var me = this, items = me.items.items, len = items.length, i = 0, item, result = [];
   *   for (; i < len; i++) { item = items[i]; result[result.length] = item; ... }
   *   if (me.floatingItems) { items = me.floatingItems.items; ... }
   *   return result;
   */
  getRefItems: function(deep) {
    var me = this,
        items,
        len,
        i = 0,
        item,
        result = [];

    // Guard against component being destroyed or items collection being null
    if (me.isDestroying || me.destroying || !me.items) {
      return result;
    }

    // Safe access to items.items - we've already verified me.items exists
    items = me.items.items;
    if (items) {
      len = items.length;
      for (; i < len; i++) {
        item = items[i];
        result[result.length] = item;
        if (deep && item && item.getRefItems) {
          result.push.apply(result, item.getRefItems(true));
        }
      }
    }

    if (me.floatingItems && me.floatingItems.items) {
      items = me.floatingItems.items;
      len = items.length;
      for (i = 0; i < len; i++) {
        item = items[i];
        result[result.length] = item;
        if (deep && item && item.getRefItems) {
          result.push.apply(result, item.getRefItems(true));
        }
      }
    }

    return result;
  }
});
