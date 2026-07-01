
/*global Ext, NX*/

/**
 * Bundle model.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.model.Bundle', {
  extend: 'Ext.data.Model',
  fields: [
    { name: 'id', type: 'int' },
    { name: 'state', type: 'string' },
    { name: 'name', type: 'string' },
    { name: 'symbolicName', type: 'string' },
    { name: 'version', type: 'string' },
    { name: 'location', type: 'string' },
    { name: 'startLevel', type: 'int' },
    { name: 'lastModified', type: 'int'},
    { name: 'fragment', type: 'bool' },
    { name: 'fragments', type: 'auto' /*array*/ },
    { name: 'fragmentHosts', type: 'auto' /*array*/ },
    { name: 'headers', type: 'auto' /*object*/}
  ]
});
