
/*global Ext*/

/**
 * Store containing {@link AS.model.Feature} records for selected feature group (children of feature node).
 *
 * @since 3.0
 */
Ext.define('AS.store.FeatureGroup', {
  extend: 'Ext.data.ArrayStore',
  model: 'AS.model.Feature',

  sorters: { property: 'path', direction: 'ASC' }
});
