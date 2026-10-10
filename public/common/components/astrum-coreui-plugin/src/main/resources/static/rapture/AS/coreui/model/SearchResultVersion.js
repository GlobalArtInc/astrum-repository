
/*global Ext, AS*/

/**
 * Versions / search result version model.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.model.SearchResultVersion', {
  extend: 'Ext.data.Model',
  idProperty: 'version',
  fields: [
    'groupingKey',
    'group',
    'name',
    'version',
    'versionOrder',
    'repositoryId',
    'repositoryName',
    'path',
    'type'
  ]
});
