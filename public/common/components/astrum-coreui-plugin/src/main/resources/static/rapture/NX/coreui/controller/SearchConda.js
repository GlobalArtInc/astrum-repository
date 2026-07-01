
/**
 * global Ext, NX
 * Conda repository search contribution.
 *
 * @since 3.19
 */
Ext.define('NX.coreui.controller.SearchConda', {
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
      id: 'conda',
      name: 'Conda',
      text: NX.I18n.get('SearchConda_Text'),
      description: NX.I18n.get('SearchConda_Description'),
      readOnly: true,
      criterias: [
        {id: 'format', value: 'conda', hidden: true},
        {id: 'name.raw'},
        {id: 'version'}
      ]
    }, me);
  }
});
