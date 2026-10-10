
/*global Ext, AS*/

/**
 * Repository Formats store.
 *
 * @since 3.14
 */
Ext.define('AS.coreui.store.RepositoryFormat', {
  extend: 'Ext.data.Store',
  model: 'AS.coreui.model.Format',

  proxy: {
    type: 'direct',
    paramsAsHash: false,

    api: {
      read: 'AS.direct.coreui_Repository.readFormats'
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
      me.insert(0, Ext.create('AS.coreui.model.Format', {
        value: AS.I18n.get('Repository_Formats_All')
      }));
    }
  }
});
