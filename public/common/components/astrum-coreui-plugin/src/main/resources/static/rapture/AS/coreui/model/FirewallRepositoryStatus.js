
/*global Ext, AS*/

/**
 * Firewall repository status model.
 *
 * @since 3.2
 */
Ext.define('AS.coreui.model.FirewallRepositoryStatus', {
  extend: 'Ext.data.Model',
  idProperty: 'repositoryName',
  fields: [
    {name:'repositoryName', type: 'string', sortType: 'asUCText'},
    {name:'affectedComponentCount', type: 'int'},
    {name:'criticalComponentCount', type: 'int'},
    {name:'moderateComponentCount', type: 'int'},
    {name:'quarantinedComponentCount', type: 'int'},
    {name:'severeComponentCount', type: 'int'},
    {name:'reportUrl', type: 'string', sortType: 'asUCText'},
    {name:'message', type: 'string', sortType: 'asUCText'},
    {name:'errorMessage', type: 'string', sortType: 'asUCText'},
    // Epoch-milliseconds — Java side is Long, which overflows ExtJS 'int' (32-bit). Use 'number'.
    // ASTRUM-52787: this is the audit-enabled watermark, NOT a per-run "last success" timestamp.
    // See FirewallRepositoryStatusXO#lastCrawlDate JavaDoc before labelling the column.
    {name:'lastCrawlDate', type: 'number'}
  ]
});
