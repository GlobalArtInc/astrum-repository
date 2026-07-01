
/*global Ext, NX*/

/**
 * Git LFS repository search contribution.
 *
 * @since 3.3
 */
Ext.define('NX.coreui.controller.SearchGitLfs', {
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
      id: 'gitlfs',
      name: 'Git LFS',
      text: NX.I18n.get('SearchGitLfs_Text'),
      description: NX.I18n.get('SearchGitLfs_Description'),
      readOnly: true,
      criterias: [
        { id: 'format', value: 'gitlfs', hidden: true },
        { id: 'name.raw' }
      ]
    }, me);
  }
});
