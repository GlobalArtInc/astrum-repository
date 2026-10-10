
/*global Ext, AS*/

/**
 * Git LFS repository search contribution.
 *
 * @since 3.3
 */
Ext.define('AS.coreui.controller.SearchGitLfs', {
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
      id: 'gitlfs',
      name: 'Git LFS',
      text: AS.I18n.get('SearchGitLfs_Text'),
      description: AS.I18n.get('SearchGitLfs_Description'),
      readOnly: true,
      criterias: [
        { id: 'format', value: 'gitlfs', hidden: true },
        { id: 'name.raw' }
      ]
    }, me);
  }
});
