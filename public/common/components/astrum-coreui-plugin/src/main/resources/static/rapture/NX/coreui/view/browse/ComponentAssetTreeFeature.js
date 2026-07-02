
/*global Ext, NX*/

/**
 * Component/Asset tree feature.
 *
 * @since 3.6
 */
Ext.define('NX.coreui.view.browse.ComponentAssetTreeFeature', {
  extend: 'NX.view.drilldown.Drilldown',
  alias: 'widget.nx-coreui-componentassettreefeature',

  iconName: 'browse-asset-default',

  masters: [
    {xtype: 'nx-coreui-browse-repository-list'},
    {xtype: 'nx-coreui-component-asset-tree'}
  ]
});
