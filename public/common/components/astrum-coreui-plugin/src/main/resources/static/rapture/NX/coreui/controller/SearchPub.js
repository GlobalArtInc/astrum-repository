
/*global Ext, NX*/

/**
 * Pub (pub.dev) repository search contribution.
 */
Ext.define('NX.coreui.controller.SearchPub', {
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
      id: 'pub',
      name: 'Pub',
      text: NX.I18n.get('SearchPub_Text'),
      description: NX.I18n.get('SearchPub_Description'),
      readOnly: true,
      criterias: [
        { id: 'format', value: 'pub', hidden: true },
        { id: 'name.raw' },
        { id: 'version' }
      ]
    }, me);
  }

});
