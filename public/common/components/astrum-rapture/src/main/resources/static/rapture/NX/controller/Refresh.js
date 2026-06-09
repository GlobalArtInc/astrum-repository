
/*global Ext, NX*/

/**
 * Refresh controller.
 *
 * @since 3.0
 */
Ext.define('NX.controller.Refresh', {
  extend: 'NX.app.Controller',
  requires: [
    'NX.Messages',
    'NX.I18n'
  ],

  views: [
    'header.Refresh'
  ],

  /**
   * @override
   */
  init: function () {
    var me = this;

    me.listen({
      component: {
        'nx-header-refresh': {
          click: me.refresh
        }
      }
    });
  },

  /**
   * Fire refresh event.
   *
   * @public
   */
  refresh: function () {
    var me = this;

    if (me.fireEvent('beforerefresh')) {
      me.fireEvent('refresh');

      // Show a message here, so that if the current view doesn't actually support
      // request that users don't think the feature is broken and spam-click the refresh button
      NX.Messages.info(NX.I18n.get('Refresh_Message'));
    }
  }

});
