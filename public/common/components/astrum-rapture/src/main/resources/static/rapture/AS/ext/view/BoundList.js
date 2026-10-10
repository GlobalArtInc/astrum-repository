
/*global Ext*/

/**
 * **{@link Ext.view.BoundList}** override, that uses HTML encoding for the inner template
 *
 * @since 3.12
 */
Ext.define('AS.ext.view.BoundList', {
  override: 'Ext.view.BoundList',

  getInnerTpl : function(displayField) {
    return '{' + displayField + ':htmlEncode}';
  }
});
