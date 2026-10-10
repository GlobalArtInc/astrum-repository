
/**
 * @since 3.28
 */
Ext.define('AS.coreui.controller.SearchR', {
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
      id: 'r',
      name: 'r',
      text: AS.I18n.get('SearchR_Text'),
      description: AS.I18n.get('SearchR_Description'),
      readOnly: true,
      criterias: [
        {id: 'format', value: 'r', hidden: true},
        {id: 'name.raw'},
        {id: 'version'}
      ]
    }, me);
  }
});
