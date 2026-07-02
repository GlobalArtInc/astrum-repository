
/*global Ext, NX*/

/**
 * Repository Formats store.
 *
 * @since 3.14
 */
Ext.define('NX.coreui.store.RepositoryFormat', {
  extend: 'Ext.data.Store',
  model: 'NX.coreui.model.Format',

  proxy: {
    type: 'direct',
    paramsAsHash: false,

    api: {
      read: 'NX.direct.coreui_Repository.readFormats'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      idProperty: 'value',
      successProperty: 'success'
    }
  },

  sorters: [{property: 'value', direction: 'ASC'}],

  listeners: {
    load: function() {
      var me = this;
      me.insert(0, Ext.create('NX.coreui.model.Format', {
        value: NX.I18n.get('Repository_Formats_All')
      }));
    }
  }
});
