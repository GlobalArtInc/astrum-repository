
/*global Ext, AS*/

/**
 * Cocoapods repository search contribution.
 *
 * @since 3.19
 */
Ext.define('AS.coreui.controller.SearchCocoapods', {
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
      id: 'cocoapods',
      name: 'Cocoapods',
      text: AS.I18n.get('SearchCocoapods_Text'),
      description: AS.I18n.get('SearchCocoapods_Description'),
      readOnly: true,
      criterias: [
        {id: 'format', value: 'cocoapods', hidden: true},
        {id: 'name.raw'},
        {id: 'version'}
      ]
    }, me);
  }
});
