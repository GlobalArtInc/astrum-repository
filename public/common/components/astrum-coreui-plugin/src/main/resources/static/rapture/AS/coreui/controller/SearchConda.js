
/**
 * global Ext, AS
 * Conda repository search contribution.
 *
 * @since 3.19
 */
Ext.define('AS.coreui.controller.SearchConda', {
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
      id: 'conda',
      name: 'Conda',
      text: AS.I18n.get('SearchConda_Text'),
      description: AS.I18n.get('SearchConda_Description'),
      readOnly: true,
      criterias: [
        {id: 'format', value: 'conda', hidden: true},
        {id: 'name.raw'},
        {id: 'version'}
      ]
    }, me);
  }
});
