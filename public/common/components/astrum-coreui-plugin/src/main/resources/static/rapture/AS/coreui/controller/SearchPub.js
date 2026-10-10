
/*global Ext, AS*/

/**
 * Pub (pub.dev) repository search contribution.
 */
Ext.define('AS.coreui.controller.SearchPub', {
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
      id: 'pub',
      name: 'Pub',
      text: AS.I18n.get('SearchPub_Text'),
      description: AS.I18n.get('SearchPub_Description'),
      readOnly: true,
      criterias: [
        { id: 'format', value: 'pub', hidden: true },
        { id: 'name.raw' },
        { id: 'version' }
      ]
    }, me);
  }

});
