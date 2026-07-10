
/*global Ext, NX*/

/**
 * Repository grid.
 *
 * @since 3.8
 */
Ext.define('NX.coreui.view.upload.UploadRepositoryList', {
  extend: 'NX.coreui.view.repository.RepositoryListTemplate',
  alias: 'widget.nx-coreui-upload-repository-list',
  requires: [
    'NX.I18n'
  ],

  stateful: true,
  stateId: 'nx-coreui-upload-repository-list',

  store: 'RepositoryReference',

  tbar: {
    xtype: 'nx-actions'
  },

  listeners: {
    beforedestroy: function(element) {
      element.getStore().clearFilter();
    }
  },

  cls: 'as-browse-upload-repository-list'
});
