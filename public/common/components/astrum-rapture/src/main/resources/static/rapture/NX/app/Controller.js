
/*global Ext, NX*/

/**
 * Astrum managed controller.
 *
 * @since 3.0
 * @see NX.app.Application#syncManagedControllers
 */
Ext.define('NX.app.Controller', {
  extend: 'Ext.app.Controller',
  mixins: {
    logAware: 'NX.LogAware'
  },

  /**
   * Event fired when a controller is being destroyed.
   *
   * @event destroy
   * @param {NX.app.Controller} controller  The controller being destroyed.
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
