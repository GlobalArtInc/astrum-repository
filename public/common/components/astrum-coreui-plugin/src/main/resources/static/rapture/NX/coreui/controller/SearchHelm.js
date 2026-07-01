
/**
 * @since 3.28
 */
Ext.define('NX.coreui.controller.SearchHelm', {
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
      id: 'helm',
      name: 'helm',
      text: NX.I18n.get('SearchHelm_Text'),
      description: NX.I18n.get('SearchHelm_Description'),
      readOnly: true,
      criterias: [
        {id: 'format', value: 'helm', hidden: true},
        {id: 'name.raw'},
        {id: 'version'}
      ]
    }, me);
  }
});
