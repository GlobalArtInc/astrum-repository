
/*global Ext, NX*/

/**
 * Menu group controller.
 *
 * @since 3.0
 */
Ext.define('NX.controller.MenuGroup', {
  extend: 'NX.app.Controller',
  requires: [
    'NX.Bookmarks'
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
        'nx-feature-group dataview': {
          selectionchange: me.onSelection
        }
      }
    });
  },

  /**
   * Invoked when {@link NX.view.feature.Group} item is selected.
   *
   * @private
   * @param {NX.view.feature.Group} view
   * @param {NX.model.Feature[]} records
   */
  onSelection: function (view, records) {
    var feature;

    if (records.length > 0) {
      feature = records[0];
      NX.Bookmarks.navigateTo(NX.Bookmarks.fromToken(feature.get('bookmark')), this);
    }
  }

});
