
/*global Ext, NX*/

/**
 * Raw repository search contribution.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.controller.SearchRaw', {
  extend: 'NX.app.Controller',
  requires: [
    'NX.I18n'
  ],

  /**
   * @override
   */
  init: function() {
    var me = this,
        search = me.getController('NX.coreui.controller.Search');

    search.registerFilter({
      id: 'raw',
      name: 'Raw',
      text: NX.I18n.get('SearchRaw_Text'),
      description: NX.I18n.get('SearchRaw_Description'),
      readOnly: true,
      criterias: [
        { id: 'format', value: 'raw', hidden: true },
        { id: 'name.raw' }
      ]
    }, me);
  }

});
