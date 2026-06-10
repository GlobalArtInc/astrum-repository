
/*global Ext*/

/**
 * Feature menu tree model.
 *
 * @since 3.0
 */
Ext.define('NX.model.FeatureMenu', {
  extend: 'Ext.data.TreeModel',

  // FIXME: define types so its clear what this data is!  Also consider comments for further clarity.

  fields: [
    { name: 'id' },
    { name: 'path' },
    {
      /**
       * Mode name.
       */
      name: 'mode',
      type: 'string'
    },
    { name: 'text' },
    { name: 'weight', defaultValue: 100 },
    { name: 'grouped', defaultValue: false },
    { name: 'view' },
    { name: 'bookmark' },
    { name: 'iconName' },
    {
      name: 'separator',
      type: 'boolean'
    }
  ]
});
