
/**
 * @since 3.28
 */
Ext.define('AS.coreui.controller.SearchHelm', {
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
      id: 'helm',
      name: 'helm',
      text: AS.I18n.get('SearchHelm_Text'),
      description: AS.I18n.get('SearchHelm_Description'),
      readOnly: true,
      criterias: [
        {id: 'format', value: 'helm', hidden: true},
        {id: 'name.raw'},
        {id: 'version'}
      ]
    }, me);
  }
});
