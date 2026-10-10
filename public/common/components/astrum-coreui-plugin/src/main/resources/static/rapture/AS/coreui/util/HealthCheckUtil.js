
/**
 * Utility functions to be shared between different implementations of SystemStatus.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.util.HealthCheckUtil', {
  singleton: true,

  /**
   * Helper method to generate consistent html for embedding iconography.
   * @param icon name of fa icon to display
   * @param style (optional) styles to include
   * @returns {string}
   */
  iconSpan: function (icon, style) {
    return '<span class="fa ' + icon + '"' + (style ? (' style="' + style + '"') : '') + '/>';
  },

  /*
   * Shortens a number, e.g. 234 -> 0.2k
   * @param a number to be formatted
   * @return {string}
   */
  simplifyNumber: function(count) {
    if (count < 100) {
      return count;
    }
    else if (count < 100000) {
      return Math.floor(count / 100) / 10 + 'k';
    }
    else if (count < 100000000) {
      return Math.floor(count / 100000) / 10 + 'm';
    }
    else {
      return Math.floor(count / 100000000) / 10 + 'b';
    }
  },

});
