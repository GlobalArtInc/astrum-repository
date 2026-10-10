
/*global Ext, AS*/

/**
 * Raw repository search contribution.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.controller.SearchRaw', {
  extend: 'AS.app.Controller',
  requires: [
    'AS.I18n'
  ],

  /**
   * @override
   */
  init: function() {
    var me = this,
        search = me.getController('AS.coreui.controller.Search');

    search.registerFilter({
      id: 'raw',
      name: 'Raw',
      text: AS.I18n.get('SearchRaw_Text'),
      description: AS.I18n.get('SearchRaw_Description'),
      readOnly: true,
      criterias: [
        { id: 'format', value: 'raw', hidden: true },
        { id: 'name.raw' }
      ]
    }, me);
  }

});
