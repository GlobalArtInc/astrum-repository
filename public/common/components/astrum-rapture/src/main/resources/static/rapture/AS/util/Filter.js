
/*global Ext*/

/**
 * Filter
 *
 * @since 3.3
 */
Ext.define('AS.util.Filter', {
  singleton: true,

  /**
   * Util to build a div for empty search results.
   *
   * @param searchString
   * @param emptyTemplate
   * @returns {string}
   */
  buildEmptyResult: function(searchString, emptyTemplate) {
    var encoded = Ext.util.Format.htmlEncode(searchString);
    return '<div class="x-grid-empty">' + emptyTemplate.replace(/\$filter/, encoded) + '</div>';
  }
});
