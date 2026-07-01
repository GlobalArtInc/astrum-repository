
/*global Ext, NX*/

/**
 * Apt repository search contribution.
 *
 * @since 3.17
 */
Ext.define('NX.coreui.controller.SearchApt', {
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
      id: 'apt',
      name: 'Apt',
      text: NX.I18n.get('SearchApt_Text'),
      description: NX.I18n.get('SearchApt_Description'),
      readOnly: true,
      criterias: [
        {id: 'format', value: 'apt', hidden: true},
        {id: 'name.raw'},
        {id: 'version'}
      ]
    }, me);
  }
});
