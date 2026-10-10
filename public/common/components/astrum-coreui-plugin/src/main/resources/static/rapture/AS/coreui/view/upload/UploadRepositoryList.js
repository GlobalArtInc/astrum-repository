
/*global Ext, AS*/

/**
 * Repository grid.
 *
 * @since 3.8
 */
Ext.define('AS.coreui.view.upload.UploadRepositoryList', {
  extend: 'AS.coreui.view.repository.RepositoryListTemplate',
  alias: 'widget.as-coreui-upload-repository-list',
  requires: [
    'AS.I18n'
  ],

  stateful: true,
  stateId: 'as-coreui-upload-repository-list',

  store: 'RepositoryReference',

  tbar: {
    xtype: 'as-actions'
  },

  listeners: {
    beforedestroy: function(element) {
      element.getStore().clearFilter();
    }
  },

  cls: 'as-browse-upload-repository-list'
});
