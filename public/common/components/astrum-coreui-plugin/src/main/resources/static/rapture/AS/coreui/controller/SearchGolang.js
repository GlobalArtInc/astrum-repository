
/*global Ext, AS*/

/**
 * Go repository search contribution.
 *
 * @since 3.28
 */
Ext.define('AS.coreui.controller.SearchGolang', {
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
      id: 'go',
      name: 'Go',
      text: AS.I18n.get('SearchGolang_Text'),
      description: AS.I18n.get('SearchGolang_Description'),
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
