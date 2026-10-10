
/*global Ext*/

/**
 * Feature store.
 *
 * @since 3.0
 */
Ext.define('AS.store.Feature', {
  extend: 'Ext.data.ArrayStore',
  model: 'AS.model.Feature',

  sorters: { property: 'path', direction: 'ASC' }
});
