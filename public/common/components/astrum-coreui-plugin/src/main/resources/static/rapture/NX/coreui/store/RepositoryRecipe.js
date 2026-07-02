
/*global Ext, NX*/

/**
 * Repository Recipe store.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.store.RepositoryRecipe', {
  extend: 'Ext.data.Store',
  model: 'NX.coreui.model.Reference',

  proxy: {
    type: 'direct',
    paramsAsHash: false,

    api: {
      read: 'NX.direct.coreui_Repository.readRecipes'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sorters: { property: 'name', direction: 'ASC' }
});
