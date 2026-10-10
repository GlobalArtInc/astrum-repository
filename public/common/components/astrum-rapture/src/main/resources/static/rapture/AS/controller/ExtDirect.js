
/*global Ext, AS*/

/**
 * Watches over Ext.Direct communication.
 *
 * @since 3.0
 */
Ext.define('AS.controller.ExtDirect', {
  extend: 'AS.app.Controller',
  requires: [
    'AS.Security',
    'AS.Messages',
    'AS.I18n'
  ],

  /**
   * @override
   */
  init: function() {
    var me = this;

    me.listen({
      direct: {
        '*': {
          beforecallback: me.checkResponse
        }
      }
    });
  },

  /**
   * Checks Ext.Direct response and automatically show warning messages if an error occurred.
   * If response specifies that authentication is required, will show the sign-in window.
   *
   * @private
   */
  checkResponse: function(provider, transaction, options) {
    var result = transaction.result,
        message;

    // FIXME: Anything that does logging here can cause Ext.Direct log event remoting to spin out of control

    if (options && options.callbackOptions && options.callbackOptions.skipResultCheck) {
      return;
    }

    if (Ext.isDefined(result)) {
      if (Ext.isDefined(result.success) && result.success === false) {

        if (Ext.isDefined(result.message)) {
          message = result.message;
        }
        else if (Ext.isDefined(result.messages)) {
          message = Ext.Array.from(result.messages).join('<br/>');
        }
      }

      if (Ext.isDefined(transaction.serverException)) {
        message = transaction.serverException.exception.message;
      }
    }
    else {
      message = AS.I18n.get('User_ConnectFailure_Message');
    }

    if (message) {
      if ((result && result.success)) {
        AS.Messages.warning(message);
      }
      else {
        AS.Messages.error(message);
      }
    }

    // HACK: disabled for now as this causes problems remoting LogEvents
    ////<if debug>
    //var logMsg = transaction.action + ':' + transaction.method + " -> " + (message ? 'Failed: ' + message : 'OK');
    //if (Ext.isDefined(result) && result.errors) {
    //  logMsg += (' Errors: ' + Ext.encode(result.errors));
    //}
    //this.logDebug(logMsg);
    ////</if>
  }

});
