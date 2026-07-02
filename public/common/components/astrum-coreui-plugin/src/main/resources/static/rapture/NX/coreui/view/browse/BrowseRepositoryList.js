
/*global Ext, NX*/

/**
 * Repository grid.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.browse.BrowseRepositoryList', {
  extend: 'NX.coreui.view.repository.RepositoryListTemplate',
  alias: 'widget.nx-coreui-browse-repository-list',
  requires: [
    'NX.I18n'
  ],

  stateful: true,
  stateId: 'nx-coreui-browse-repository-list',

  hideBlobStore: true,

  store: 'RepositoryReference',

  tbar: {
    xtype: 'nx-actions'
  },

  cls: 'as-browse-browse-repository-list'
});
