
/*global Ext, AS*/

/**
 * Repository Recipe store.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.store.RepositoryRecipe', {
  extend: 'Ext.data.Store',
  model: 'AS.coreui.model.Reference',

  proxy: {
    type: 'direct',
    paramsAsHash: false,

    api: {
      read: 'AS.direct.coreui_Repository.readRecipes'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sorters: { property: 'name', direction: 'ASC' }
});
