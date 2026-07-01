
/*global Ext, NX*/

/**
 * Cargo repository search contribution.
 *
 *
 */
Ext.define('NX.coreui.controller.SearchCargo', {
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
      id: 'cargo',
      name: 'Cargo',
      text: NX.I18n.get('SearchCargo_Text'),
      description: NX.I18n.get('SearchCargo_Description'),
      readOnly: true,
      criterias: [
        { id: 'format', value: 'cargo', hidden: true },
        { id: 'name.raw' },
        { id: 'version' }
      ]
    }, me);
  }

});
