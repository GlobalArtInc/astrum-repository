
/*global Ext, AS*/

/**
 * Node model.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.model.Node', {
  extend: 'Ext.data.Model',
  idProperty: 'nodeIdentity',
  fields: [
    { name: 'nodeIdentity', type: 'string' },
    { name: 'local', type: 'bool' },
    { name: 'socketAddress', type: 'string' },
    { name: 'friendlyName', type: 'string' },
    { name: 'attributes', type: 'auto' /*object*/}
  ]
});
