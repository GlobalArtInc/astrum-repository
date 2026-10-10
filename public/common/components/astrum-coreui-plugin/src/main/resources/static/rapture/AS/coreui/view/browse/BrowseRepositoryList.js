
/*global Ext, AS*/

/**
 * Repository grid.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.browse.BrowseRepositoryList', {
  extend: 'AS.coreui.view.repository.RepositoryListTemplate',
  alias: 'widget.as-coreui-browse-repository-list',
  requires: [
    'AS.I18n'
  ],

  stateful: true,
  stateId: 'as-coreui-browse-repository-list',

  hideBlobStore: true,

  store: 'RepositoryReference',

  tbar: {
    xtype: 'as-actions'
  },

  cls: 'as-browse-browse-repository-list'
});
