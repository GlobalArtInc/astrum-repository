
/*global Ext, NX*/

/**
 * Helpers to open browser windows.
 *
 * @since 3.0
 */
Ext.define('NX.Windows', {
  singleton: true,
  requires: [
    'NX.Messages',
    'NX.I18n'
  ],
  mixins: {
    logAware: 'NX.LogAware'
  },

  /**
   * Open a new browser window.
   *
   * @public
   * @return Browser window object or {@code null} if unable to open.
   */
  open: function(url, name, specs, replace) {
    var win;

    // apply default window specs if omitted, helps keep windows user-controllable on most browsers
    if (specs === undefined) {
      specs = 'menubar=yes,location=yes,resizable=yes,scrollbars=yes,status=yes';
    }

    //<if debug>
    this.logDebug('Opening window: url=' + url + ', name=' + name + ', specs=' + specs + ', replace=' + replace);
    //</if>

    win = NX.global.open(url, name, specs, replace);
    if (win === null) {
      NX.Messages.error(NX.I18n.get('Windows_Popup_Message'));
    }
    return win;
  }
});
