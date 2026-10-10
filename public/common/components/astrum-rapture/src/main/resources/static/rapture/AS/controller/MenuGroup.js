
/*global Ext, AS*/

/**
 * Menu group controller.
 *
 * @since 3.0
 */
Ext.define('AS.controller.MenuGroup', {
  extend: 'AS.app.Controller',
  requires: [
    'AS.Bookmarks'
  ],

  views: [
    'feature.Group'
  ],

  /**
   * @override
   */
  init: function () {
    var me = this;

    me.listen({
      component: {
        'as-feature-group dataview': {
          selectionchange: me.onSelection
        }
      }
    });
  },

  /**
   * Invoked when {@link AS.view.feature.Group} item is selected.
   *
   * @private
   * @param {AS.view.feature.Group} view
   * @param {AS.model.Feature[]} records
   */
  onSelection: function (view, records) {
    var feature;

    if (records.length > 0) {
      feature = records[0];
      AS.Bookmarks.navigateTo(AS.Bookmarks.fromToken(feature.get('bookmark')), this);
    }
  }

});
