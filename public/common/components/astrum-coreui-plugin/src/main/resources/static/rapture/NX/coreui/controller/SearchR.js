
/**
 * @since 3.28
 */
Ext.define('NX.coreui.controller.SearchR', {
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
      id: 'r',
      name: 'r',
      text: NX.I18n.get('SearchR_Text'),
      description: NX.I18n.get('SearchR_Description'),
      readOnly: true,
      criterias: [
        {id: 'format', value: 'r', hidden: true},
        {id: 'name.raw'},
        {id: 'version'}
      ]
    }, me);
  }
});
