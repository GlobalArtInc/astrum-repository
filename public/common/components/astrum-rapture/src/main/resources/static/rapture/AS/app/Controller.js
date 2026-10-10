
/*global Ext, AS*/

/**
 * Astrum managed controller.
 *
 * @since 3.0
 * @see AS.app.Application#syncManagedControllers
 */
Ext.define('AS.app.Controller', {
  extend: 'Ext.app.Controller',
  mixins: {
    logAware: 'AS.LogAware'
  },

  /**
   * Event fired when a controller is being destroyed.
   *
   * @event destroy
   * @param {AS.app.Controller} controller  The controller being destroyed.
   */

  /**
   * Optional callback to invoke when a controller is being destroyed.
   *
   * @protected
   * @property {Function}
   */
  onDestroy: undefined,

  /**
   * Optional callback to invoke when a controller has fully destroyed.
   *
   * @protected
   * @property {Function}
   */
  destroy: undefined
});
