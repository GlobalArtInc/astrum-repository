
/*global Ext, AS*/

/**
 * Alpine repository search contribution.
 *
 * @since 3.95
 */
Ext.define('AS.coreui.controller.SearchAlpine', {
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
      id: 'alpine',
      name: 'Alpine',
      text: AS.I18n.get('SearchAlpine_Text'),
      description: AS.I18n.get('SearchAlpine_Description'),
      readOnly: true,
      criterias: [
        { id: 'format', value: 'alpine', hidden: true },
        { id: 'name.raw' },
        { id: 'version' }
      ]
    }, me);
  }
});
