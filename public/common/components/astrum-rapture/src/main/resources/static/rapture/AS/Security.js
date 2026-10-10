
/*global Ext, AS*/

/**
 * Helpers to interact with **{@link AS.controller.User}** controller.
 *
 * @since 3.0
 */
Ext.define('AS.Security', {
  singleton: true,
  requires: [
    'AS.controller.User'
  ],

  /**
   * @private
   * @returns {AS.controller.User}
   */
  controller: function () {
    return AS.getApplication().getController('User');
  },

  /**
   * @see AS.controller.User#hasUser
   */
  hasUser: function () {
    var me = this;
    if (me.controller()) {
      return me.controller().hasUser();
    }
  },

  /**
   * @see AS.controller.User#doWithAuthenticationToken
   */
  doWithAuthenticationToken: function (message, options) {
    var me = this;
    if (me.controller()) {
      me.controller().doWithAuthenticationToken(message, options);
    }
  },

  /**
   * @see AS.controller.User#signOut
   */
  signOut: function () {
    var me = this;
    if (me.controller()) {
      me.controller().signOut();
    }
  },

  /**
   * Shared helper for creating a server-side session using base64 credentials.
   *
   * NOTE: This function does NOT perform the base64 encoding. It expects
   *       `b64username` and `b64password` to be already encoded before calling.
   * @param {String} b64username
   * @param {String} b64password
   * @returns {Ext.promise.Promise}
   */
  requestSession: function (b64username, b64password) {
    var deferred = new Ext.Deferred();

    Ext.Ajax.request({
      url: AS.util.Url.urlOf('service/rapture/session'),
      method: 'POST',
      params: {
        username: b64username,
        password: b64password
      },
      suppressStatus: true,
      success: function (response) {
        deferred.resolve(response);
      },
      failure: function (response) {
        deferred.reject(response);
      }
    });

    return deferred.promise;
  }

});
