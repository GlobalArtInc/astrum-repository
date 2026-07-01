
/*global Ext, NX*/

/**
 * Alpine repository search contribution.
 *
 * @since 3.95
 */
Ext.define('NX.coreui.controller.SearchAlpine', {
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
      id: 'alpine',
      name: 'Alpine',
      text: NX.I18n.get('SearchAlpine_Text'),
      description: NX.I18n.get('SearchAlpine_Description'),
      readOnly: true,
      criterias: [
        { id: 'format', value: 'alpine', hidden: true },
        { id: 'name.raw' },
        { id: 'version' }
      ]
    }, me);
  }
});
