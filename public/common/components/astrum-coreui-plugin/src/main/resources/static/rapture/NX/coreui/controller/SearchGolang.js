
/*global Ext, NX*/

/**
 * Go repository search contribution.
 *
 * @since 3.28
 */
Ext.define('NX.coreui.controller.SearchGolang', {
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
      id: 'go',
      name: 'Go',
      text: NX.I18n.get('SearchGolang_Text'),
      description: NX.I18n.get('SearchGolang_Description'),
      readOnly: true,
      criterias: [
        { id: 'format', value: 'go', hidden: true },
        { id: 'name.raw' },
        { id: 'version' },
        { id: 'keyword' }
      ]
    }, me);
  }
});
