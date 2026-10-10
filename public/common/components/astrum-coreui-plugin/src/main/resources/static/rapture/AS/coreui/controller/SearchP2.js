
Ext.define('AS.coreui.controller.SearchP2', {
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

    search.registerCriteria([
      {
        id: 'attributes.p2.pluginName',
        group: AS.I18n.get('SearchP2_Group'),
        config: {
          format: 'p2',
          fieldLabel: AS.I18n.get('SearchP2_PluginName_FieldLabel'),
          width: 250
        }
      }
    ], me);

    search.registerFilter({
      id: 'p2',
      name: 'p2',
      text: AS.I18n.get('SearchP2_Text'),
      description: AS.I18n.get('SearchP2_Description'),
      readOnly: true,
      criterias: [
        {id: 'format', value: 'p2', hidden: true},
        {id: 'attributes.p2.pluginName'},
        {id: 'name.raw'}
      ]
    }, me);
  }
});
