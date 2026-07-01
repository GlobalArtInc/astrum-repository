
/*global Ext, NX*/

/**
 * Health Check repository status model.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.model.HealthCheckRepositoryStatus', {
  extend: 'Ext.data.Model',
  idProperty: 'repositoryName',
  fields: [
    {name:'repositoryName', type: 'string', sortType: 'asUCText'},
    {name:'enabled', type: 'boolean'},
    {name:'eulaAccepted', type: 'boolean'},
    {name:'analyzing', type: 'boolean'},
    {name:'detailedReportSupported', type: 'boolean'},
    {name:'iframeHeight', type: 'int'},
    {name:'iframeWidth', type: 'int'},
    {name:'securityIssueCount', type: 'int'},
    {name:'licenseIssueCount', type: 'int'},
    {name:'summaryUrl', type: 'string', sortType: 'asUCText'},
    {name:'detailUrl', type: 'string', sortType: 'asUCText'},
    {name:'totalCounts', type: 'auto' /* array */},
    {name:'vulnerableCounts', type: 'auto' /* array */}
  ]
});
