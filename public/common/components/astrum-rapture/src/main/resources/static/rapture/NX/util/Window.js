
/*global Ext, NX*/

/**
 * Window related utils.
 *
 * @since 3.10
 */
Ext.define('NX.util.Window', {
  singleton: true,
  requires: [
    'Ext.ComponentQuery'
  ],

  closeWindows: function () {
    var windows = Ext.ComponentQuery.query('window');
    Ext.each(windows, function (win) {
      win.close && win.rendered && win.close();
    });
  }
});
