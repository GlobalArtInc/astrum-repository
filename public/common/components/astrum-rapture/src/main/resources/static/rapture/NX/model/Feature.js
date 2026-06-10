
/*global Ext*/

/**
 * Feature model.
 *
 * @since 3.0
 */
Ext.define('NX.model.Feature', {
  extend: 'Ext.data.Model',

  idProperty: 'id',

  // FIXME: define types so its clear what this data is!  Also consider comments for further clarity.

  fields: [
    { name: 'id' },
    { name: 'path' },
    { name: 'text' },
    { name: 'textComplement', defaultValue: '' },
    {
      /**
       * Mode name.
       */
      name: 'mode',
      type: 'string',

      // FIXME: why is this defaulting to 'admin'?
      defaultValue: 'admin'
    },
    { name: 'weight', defaultValue: 100 },
    { name: 'group', defaultValue: false },
    { name: 'view', defaultValue: undefined },
    { name: 'visible', defaultValue: true },
    { name: 'expanded', defaultValue: true },
    { name: 'bookmark', defaultValue: undefined },
    { name: 'iconName', defaultValue: undefined },
    { name: 'description', defaultValue: undefined },
    { name: 'authenticationRequired', defaultValue: true }
  ]
});
