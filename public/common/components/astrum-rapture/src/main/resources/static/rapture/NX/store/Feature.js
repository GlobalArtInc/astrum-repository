
/*global Ext*/

/**
 * Feature store.
 *
 * @since 3.0
 */
Ext.define('NX.store.Feature', {
  extend: 'Ext.data.ArrayStore',
  model: 'NX.model.Feature',

  sorters: { property: 'path', direction: 'ASC' }
});
