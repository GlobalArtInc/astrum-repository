
/*global Ext, AS*/

/**
 * Helpers to open browser windows.
 *
 * @since 3.0
 */
Ext.define('AS.Windows', {
  singleton: true,
  requires: [
    'AS.Messages',
    'AS.I18n'
  ],
  mixins: {
    logAware: 'AS.LogAware'
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

    win = AS.global.open(url, name, specs, replace);
    if (win === null) {
      AS.Messages.error(AS.I18n.get('Windows_Popup_Message'));
    }
    return win;
  }
});
