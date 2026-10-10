
/*global Ext, AS*/

/**
 * Refresh controller.
 *
 * @since 3.0
 */
Ext.define('AS.controller.Refresh', {
  extend: 'AS.app.Controller',
  requires: [
    'AS.Messages',
    'AS.I18n'
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
        'as-header-refresh': {
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
      AS.Messages.info(AS.I18n.get('Refresh_Message'));
    }
  }

});
