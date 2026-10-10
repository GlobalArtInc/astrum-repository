
/*global Ext*/

/**
 * Helpers to show dialog boxes.
 *
 * @since 3.0
 */
Ext.define('AS.Dialogs', {
  singleton: true,
  requires: [
    'AS.I18n'
  ],

  /**
   * Show information dialog.
   *
   * @public
   */
  showInfo: function (title, message, options, htmlEncoded) {
    options = options || {};

    if (!htmlEncoded) {
      message = Ext.htmlEncode(message);
      title = title ? Ext.htmlEncode(title) : title;
    }

    // set default configuration
    Ext.applyIf(options, {
      title: title || AS.I18n.get('Dialogs_Info_Title'),
      msg: message,
      buttons: Ext.Msg.OK,
      icon: Ext.MessageBox.INFO,
      closable: true
    });

    Ext.Msg.show(options);
  },

  /**
   * Show error dialog.
   *
   * @public
   */
  showError: function (title, message, options, htmlEncoded) {
    options = options || {};

    if (!htmlEncoded) {
      message = Ext.htmlEncode(message);
      title = title ? Ext.htmlEncode(title) : title;
    }

    // set default configuration
    Ext.applyIf(options, {
      title: title || AS.I18n.get('Dialogs_Error_Title'),
      msg: message || AS.I18n.get('Dialogs_Error_Message'),
      buttons: Ext.Msg.OK,
      icon: Ext.MessageBox.ERROR,
      closable: true
    });

    Ext.Msg.show(options);
  },

  /**
   * Show confirmation dialog.
   *
   * @public
   */
  askConfirmation: function (title, message, onYesFn, options, htmlEncoded) {
    options = options || {};

    if (!htmlEncoded) {
      message = Ext.htmlEncode(message);
      title = title ? Ext.htmlEncode(title) : title;
    }

    Ext.Msg.show({
      title: title,
      msg: message,
      buttons: Ext.Msg.YESNO,
      icon: Ext.MessageBox.QUESTION,
      closable: false,
      animEl: options.animEl,
      buttonText: options.buttonText,
      fn: function (buttonName) {
        if (buttonName === 'yes' || buttonName === 'ok') {
          if (Ext.isDefined(onYesFn)) {
            onYesFn.call(options.scope);
          }
        }
        else {
          if (Ext.isDefined(options.onNoFn)) {
            options.onNoFn.call(options.scope);
          }
        }
      }
    });
  }

});
