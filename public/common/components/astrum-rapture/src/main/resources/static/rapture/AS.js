
/*global Ext*/

// NOTE: This is not a real class, but exists for jsdoc purposes ONLY
// NOTE: Namespace is created via bootstrap.js and app.js

/**
 * AS singleton.
 *
 * @class AS
 * @singleton
 * @since 3.0
 */
(function () {
  /**
   * Global reference.
   *
   * @public
   * @property {Object} global
   * @readonly
   */

  /**
   * Application reference.
   *
   * @internal
   * @property {AS.app.Application} application
   * @readonly
   */

  /**
   * Container for bootstrap cruft.
   *
   * @internal
   * @property {Object} app
   * @property {Boolean} app.debug  Flag if debug is enabled.
   * @property {String} app.baseUrl Initial application base-url.
   * @property {Object} app.state   Initial application state.
   * @readonly
   */
});
