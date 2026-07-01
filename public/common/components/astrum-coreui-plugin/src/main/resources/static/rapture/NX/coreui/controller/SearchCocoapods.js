
/*global Ext, NX*/

/**
 * Cocoapods repository search contribution.
 *
 * @since 3.19
 */
Ext.define('NX.coreui.controller.SearchCocoapods', {
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
      id: 'cocoapods',
      name: 'Cocoapods',
      text: NX.I18n.get('SearchCocoapods_Text'),
      description: NX.I18n.get('SearchCocoapods_Description'),
      readOnly: true,
      criterias: [
        {id: 'format', value: 'cocoapods', hidden: true},
        {id: 'name.raw'},
        {id: 'version'}
      ]
    }, me);
  }
});
