
/*global Ext, AS*/

/**
 * Cargo repository search contribution.
 *
 *
 */
Ext.define('AS.coreui.controller.SearchCargo', {
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
      id: 'cargo',
      name: 'Cargo',
      text: AS.I18n.get('SearchCargo_Text'),
      description: AS.I18n.get('SearchCargo_Description'),
      readOnly: true,
      criterias: [
        { id: 'format', value: 'cargo', hidden: true },
        { id: 'name.raw' },
        { id: 'version' }
      ]
    }, me);
  }

});
