
/*global Ext, AS*/

/**
 * Component/Asset tree feature.
 *
 * @since 3.6
 */
Ext.define('AS.coreui.view.browse.ComponentAssetTreeFeature', {
  extend: 'AS.view.drilldown.Drilldown',
  alias: 'widget.as-coreui-componentassettreefeature',

  iconName: 'browse-asset-default',

  masters: [
    {xtype: 'as-coreui-browse-repository-list'},
    {xtype: 'as-coreui-component-asset-tree'}
  ]
});
