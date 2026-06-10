
/*global Ext, NX*/

/**
 * Array related utils.
 *
 * @since 3.0
 */
Ext.define('NX.util.Array', {
  singleton: true,
  requires: [
    'Ext.Array'
  ],

  /**
   * Check if one array contains all elements from another.
   *
   * @public
   * @param {Array} array1
   * @param {Array} array2
   * @return {boolean} true if array1 contains all elements from array2.
   */
  containsAll: function(array1, array2) {
    var i;
    for (i=0; i<array2.length; i++) {
      if (!Ext.Array.contains(array1, array2[i])) {
        return false;
      }
    }
    return true;
  }
});
