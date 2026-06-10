
/*global Ext*/

/**
 * Store containing {@link NX.model.Feature} records for selected feature group (children of feature node).
 *
 * @since 3.0
 */
Ext.define('NX.store.FeatureGroup', {
  extend: 'Ext.data.ArrayStore',
  model: 'NX.model.Feature',

  sorters: { property: 'path', direction: 'ASC' }
});
