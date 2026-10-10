
/*global Ext, AS*/

/**
 * Apt repository search contribution.
 *
 * @since 3.17
 */
Ext.define('AS.coreui.controller.SearchApt', {
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
      id: 'apt',
      name: 'Apt',
      text: AS.I18n.get('SearchApt_Text'),
      description: AS.I18n.get('SearchApt_Description'),
      readOnly: true,
      criterias: [
        {id: 'format', value: 'apt', hidden: true},
        {id: 'name.raw'},
        {id: 'version'}
      ]
    }, me);
  }
});
