
/*global Ext*/

/**
 * Dependency snippet model.
 *
 * @since 3.15
 */
Ext.define('NX.model.DependencySnippet', {
  extend: 'Ext.data.Model',

  fields: [
    { name: 'format', type: 'string' },
    { name: 'snippetGenerator', type: 'auto' }
  ]
});
